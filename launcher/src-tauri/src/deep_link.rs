use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "kind", content = "value")]
pub enum DeepLinkPayload {
    #[serde(rename = "id")]
    Id(String),
    #[serde(rename = "ip")]
    Ip(String),
    #[serde(rename = "error")]
    Error(String),
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct AuthCallbackPayload {
    pub access_token: Option<String>,
    pub error: Option<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct MatchConnectDeepLink {
    pub match_id: String,
    pub host: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_host: Option<String>,
}

pub struct DeepLinkState {
    auth_ready: bool,
    pending_auth: Vec<AuthCallbackPayload>,
}

pub struct DeepLinkStateContainer(pub Mutex<DeepLinkState>);

impl DeepLinkStateContainer {
    pub fn new() -> Self {
        Self(Mutex::new(DeepLinkState {
            auth_ready: false,
            pending_auth: Vec::new(),
        }))
    }
}

pub fn is_valid_ip_port(value: &str) -> bool {
    let (ip, port) = match value.split_once(':') {
        Some(p) => p,
        None => return false,
    };
    let octets: Vec<&str> = ip.split('.').collect();
    if octets.len() != 4 || octets.iter().any(|o| o.parse::<u8>().is_err()) {
        return false;
    }
    match port.parse::<u16>() {
        Ok(p) => p > 0,
        Err(_) => false,
    }
}

pub fn parse_auth_url(url_str: &str) -> Option<AuthCallbackPayload> {
    let url = url::Url::parse(url_str).ok()?;
    if url.scheme() != "partylock" {
        return None;
    }
    if url.host_str()? != "auth" {
        return None;
    }
    if url.path() != "/callback" {
        return None;
    }

    let mut access_token = None;
    let mut error = None;
    for (key, value) in url.query_pairs() {
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

pub fn parse_match_connect_url(url_str: &str) -> Option<MatchConnectDeepLink> {
    let url = url::Url::parse(url_str).ok()?;
    if url.scheme() != "partylock" {
        return None;
    }
    if url.host_str()? != "match" || url.path() != "/connect" {
        return None;
    }

    let mut match_id = None;
    let mut host = None;
    let mut port = None;
    let mut local_host = None;
    for (key, value) in url.query_pairs() {
        if value.is_empty() {
            continue;
        }
        match key.as_ref() {
            "matchId" => match_id = Some(value.into_owned()),
            "host" => host = Some(value.into_owned()),
            "port" => port = value.parse::<u16>().ok(),
            "localHost" => local_host = Some(value.into_owned()),
            _ => {}
        }
    }

    let match_id = match_id?;
    let host = host?;
    let port = port?;
    if !is_valid_ip_port(&format!("{host}:{port}")) {
        return None;
    }

    Some(MatchConnectDeepLink {
        match_id,
        host,
        port,
        local_host,
    })
}

pub fn dispatch_auth(app: &AppHandle, payload: AuthCallbackPayload) {
    let state = app.state::<DeepLinkStateContainer>();
    let mut s = state.0.lock().unwrap();
    if s.auth_ready {
        drop(s);
        println!(
            "[steam-auth] emitting auth-callback (token={}, error={})",
            payload.access_token.is_some(),
            payload.error.as_deref().unwrap_or("-")
        );
        let _ = app.emit("auth-callback", payload);
    } else {
        println!(
            "[steam-auth] auth listener not ready — queued callback (pending={})",
            s.pending_auth.len() + 1
        );
        s.pending_auth.push(payload);
    }
}

pub fn handle_incoming_url(app: &AppHandle, url_str: &str) {
    if let Some(payload) = parse_auth_url(url_str) {
        dispatch_auth(app, payload);
        surface_main_window(app);
        return;
    }
    if let Some(payload) = parse_match_connect_url(url_str) {
        let _ = app.emit("match-connect", payload);
        surface_main_window(app);
    }
}

pub fn surface_main_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Frontend calls once its auth listener is mounted.
#[tauri::command]
pub fn auth_callback_ready(state: tauri::State<DeepLinkStateContainer>, app: AppHandle) {
    let mut s = state.0.lock().unwrap();
    s.auth_ready = true;
    let pending = std::mem::take(&mut s.pending_auth);
    let pending_count = pending.len();
    drop(s);
    println!("[steam-auth] frontend auth listener ready (flushing {pending_count} queued)");
    for payload in pending {
        let _ = app.emit("auth-callback", payload);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_auth_success() {
        let payload = parse_auth_url("partylock://auth/callback?access_token=abc123").unwrap();
        assert_eq!(payload.access_token.as_deref(), Some("abc123"));
        assert!(payload.error.is_none());
    }

    #[test]
    fn parses_auth_error() {
        let payload = parse_auth_url("partylock://auth/callback?error=vac_ban").unwrap();
        assert!(payload.access_token.is_none());
        assert_eq!(payload.error.as_deref(), Some("vac_ban"));
    }

    #[test]
    fn ignores_non_partylock_scheme() {
        assert!(parse_auth_url("deadworks://connect/foo").is_none());
    }

    #[test]
    fn parses_match_connect() {
        let payload = parse_match_connect_url(
            "partylock://match/connect?matchId=abc&host=10.0.0.5&port=27015&localHost=127.0.0.1",
        )
        .unwrap();
        assert_eq!(payload.match_id, "abc");
        assert_eq!(payload.host, "10.0.0.5");
        assert_eq!(payload.port, 27015);
        assert_eq!(payload.local_host.as_deref(), Some("127.0.0.1"));
    }

    #[test]
    fn rejects_invalid_match_connect() {
        assert!(parse_match_connect_url("partylock://auth/callback").is_none());
        assert!(parse_match_connect_url(
            "partylock://match/connect?matchId=a&host=bad&port=27015"
        )
        .is_none());
    }
}
