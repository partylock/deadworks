use std::sync::Mutex;
use std::time::Duration;

use tauri::AppHandle;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::deep_link::{dispatch_auth, surface_main_window, AuthCallbackPayload};

fn steam_log(message: &str) {
    println!("[steam-auth] {message}");
}

static ACTIVE_LISTENER: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

fn parse_loopback_callback(request: &str) -> Option<AuthCallbackPayload> {
    let first = request.lines().next()?;
    let mut parts = first.split_whitespace();
    if parts.next()? != "GET" {
        return None;
    }
    let path = parts.next()?;
    let query = path.split('?').nth(1)?;

    let mut access_token = None;
    let mut error = None;
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        match key.as_ref() {
            "access_token" if !value.is_empty() => access_token = Some(value.into_owned()),
            "error" if !value.is_empty() => error = Some(value.into_owned()),
            _ => {}
        }
    }

    Some(AuthCallbackPayload {
        access_token,
        error,
    })
}

async fn write_success_response(stream: &mut tokio::net::TcpStream) {
    let body = "<!DOCTYPE html><html lang=\"pt-BR\"><body><p>Login concluído. Pode fechar esta aba e voltar ao PartyLock.</p></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
}

#[tauri::command]
pub async fn stop_steam_auth_listener() {
    if let Some(handle) = ACTIVE_LISTENER.lock().unwrap().take() {
        steam_log("loopback listener stopped");
        handle.abort();
    }
}

#[tauri::command]
pub async fn start_steam_auth_listener(app: AppHandle) -> Result<u16, String> {
    stop_steam_auth_listener().await;

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind loopback port: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to read loopback port: {e}"))?
        .port();

    let app_handle = app.clone();
    steam_log(&format!("loopback listener started on 127.0.0.1:{port}"));
    let handle = tokio::spawn(async move {
        let accept_result = tokio::time::timeout(Duration::from_secs(300), listener.accept()).await;
        let Ok(Ok((mut stream, _))) = accept_result else {
            steam_log("loopback timeout (300s) — browser never called back; check backend redirect & endpoint");
            return;
        };

        steam_log("loopback connection received");
        let mut buf = vec![0u8; 32_768];
        let n = stream.read(&mut buf).await.unwrap_or(0);
        if n == 0 {
            steam_log("loopback empty request");
            return;
        }

        let request = String::from_utf8_lossy(&buf[..n]);
        let first_line = request.lines().next().unwrap_or("<empty>");
        if let Some(payload) = parse_loopback_callback(&request) {
            let has_token = payload.access_token.is_some();
            let has_error = payload.error.is_some();
            steam_log(&format!(
                "loopback parsed ok (token={has_token}, error={has_error})"
            ));
            dispatch_auth(&app_handle, payload);
            surface_main_window(&app_handle);
            write_success_response(&mut stream).await;
        } else {
            steam_log(&format!("loopback parse failed — first line: {first_line}"));
        }
    });

    *ACTIVE_LISTENER.lock().unwrap() = Some(handle);
    Ok(port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_loopback_success() {
        let request = "GET /callback?access_token=abc.def HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        let payload = parse_loopback_callback(request).unwrap();
        assert_eq!(payload.access_token.as_deref(), Some("abc.def"));
        assert!(payload.error.is_none());
    }

    #[test]
    fn parses_loopback_error() {
        let request = "GET /callback?error=steam_auth_failed HTTP/1.1\r\n\r\n";
        let payload = parse_loopback_callback(request).unwrap();
        assert_eq!(payload.error.as_deref(), Some("steam_auth_failed"));
    }
}
