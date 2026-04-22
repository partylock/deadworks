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

pub struct DeepLinkState {
    ready: bool,
    pending: Vec<DeepLinkPayload>,
}

pub struct DeepLinkStateContainer(pub Mutex<DeepLinkState>);

impl DeepLinkStateContainer {
    pub fn new() -> Self {
        Self(Mutex::new(DeepLinkState {
            ready: false,
            pending: Vec::new(),
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

pub fn parse_url(url_str: &str) -> DeepLinkPayload {
    let url = match url::Url::parse(url_str) {
        Ok(u) => u,
        Err(_) => return DeepLinkPayload::Error(format!("invalid URL: {}", url_str)),
    };
    if url.scheme() != "deadworks" {
        return DeepLinkPayload::Error(format!("unsupported scheme: {}", url.scheme()));
    }
    let action = url.host_str().unwrap_or("");
    let value = url.path().trim_start_matches('/').to_string();
    match action {
        "connect" => {
            if value.is_empty() {
                DeepLinkPayload::Error("missing server id".into())
            } else {
                DeepLinkPayload::Id(value)
            }
        }
        "connectip" => {
            if is_valid_ip_port(&value) {
                DeepLinkPayload::Ip(value)
            } else {
                DeepLinkPayload::Error("malformed address, expected ip:port".into())
            }
        }
        other => DeepLinkPayload::Error(format!("unknown action: {}", other)),
    }
}

/// Emit payload to the frontend if it's listening, else buffer until it signals readiness.
pub fn dispatch(app: &AppHandle, payload: DeepLinkPayload) {
    let state = app.state::<DeepLinkStateContainer>();
    let mut s = state.0.lock().unwrap();
    if s.ready {
        drop(s);
        let _ = app.emit("deep-link://connect", payload);
    } else {
        s.pending.push(payload);
    }
}

pub fn surface_main_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Frontend calls this once its listener is mounted. Marks ready and replays any
/// URLs that arrived before the listener existed (cold-start case) via the same
/// `deep-link://connect` event the listener already subscribes to. Idempotent.
#[tauri::command]
pub fn deep_link_ready(state: tauri::State<DeepLinkStateContainer>, app: AppHandle) {
    let mut s = state.0.lock().unwrap();
    s.ready = true;
    let pending = std::mem::take(&mut s.pending);
    drop(s);
    for p in pending {
        let _ = app.emit("deep-link://connect", p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_connect_with_id() {
        match parse_url("deadworks://connect/abc123DEF456") {
            DeepLinkPayload::Id(v) => assert_eq!(v, "abc123DEF456"),
            other => panic!("expected Id, got {:?}", other),
        }
    }

    #[test]
    fn parses_connectip_with_port() {
        match parse_url("deadworks://connectip/1.2.3.4:27015") {
            DeepLinkPayload::Ip(v) => assert_eq!(v, "1.2.3.4:27015"),
            other => panic!("expected Ip, got {:?}", other),
        }
    }

    #[test]
    fn rejects_connectip_without_port() {
        match parse_url("deadworks://connectip/1.2.3.4") {
            DeepLinkPayload::Error(_) => {}
            other => panic!("expected Error, got {:?}", other),
        }
    }

    #[test]
    fn rejects_bad_octet() {
        match parse_url("deadworks://connectip/999.0.0.1:27015") {
            DeepLinkPayload::Error(_) => {}
            other => panic!("expected Error, got {:?}", other),
        }
    }

    #[test]
    fn rejects_unknown_action() {
        match parse_url("deadworks://foo/bar") {
            DeepLinkPayload::Error(_) => {}
            other => panic!("expected Error, got {:?}", other),
        }
    }
}
