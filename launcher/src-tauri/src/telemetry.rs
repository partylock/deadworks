use serde::Serialize;
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_store::StoreBuilder;

#[derive(Serialize)]
struct InstallPayload {
    install_id: String,
    version: String,
    os: String,
    arch: String,
}

#[derive(Serialize)]
struct HeartbeatPayload {
    install_id: String,
    version: String,
    os: String,
}

fn api_base_url(endpoint: &str) -> &'static str {
    if cfg!(debug_assertions) && endpoint == "local" {
        "http://localhost:8787"
    } else {
        "https://api.deadworks.net"
    }
}

pub fn maybe_send_install(app: &AppHandle) {
    let Ok(store) = StoreBuilder::new(app, "settings.json").build() else {
        return;
    };
    if !store
        .get("telemetry_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
    {
        return;
    }

    let existing: Option<String> = store
        .get("install_id")
        .and_then(|v| v.as_str().map(String::from));
    if existing.is_some() {
        return;
    }

    let id = uuid::Uuid::new_v4().to_string();
    store.set("install_id", serde_json::Value::String(id.clone()));
    if store.save().is_err() {
        return;
    }

    let endpoint = store
        .get("api_endpoint")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "prod".to_string());
    let url = format!("{}/v1/install", api_base_url(&endpoint));

    let payload = InstallPayload {
        install_id: id,
        version: app.package_info().version.to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    };
    if let Ok(body) = serde_json::to_value(&payload) {
        send(url, body);
    }
}

pub fn maybe_send_heartbeat(app: &AppHandle) {
    let Ok(store) = StoreBuilder::new(app, "settings.json").build() else {
        return;
    };
    if !store
        .get("telemetry_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
    {
        return;
    }

    let id = match store
        .get("install_id")
        .and_then(|v| v.as_str().map(String::from))
    {
        Some(id) => id,
        None => {
            let new_id = uuid::Uuid::new_v4().to_string();
            store.set("install_id", serde_json::Value::String(new_id.clone()));
            if store.save().is_err() {
                return;
            }
            new_id
        }
    };

    let endpoint = store
        .get("api_endpoint")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "prod".to_string());
    let url = format!("{}/v1/heartbeat", api_base_url(&endpoint));

    let payload = HeartbeatPayload {
        install_id: id,
        version: app.package_info().version.to_string(),
        os: std::env::consts::OS.to_string(),
    };
    if let Ok(body) = serde_json::to_value(&payload) {
        send(url, body);
    }
}

fn send(url: String, payload: serde_json::Value) {
    tauri::async_runtime::spawn(async move {
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                println!("[telemetry] client build failed: {}", e);
                return;
            }
        };
        if let Err(e) = client.post(&url).json(&payload).send().await {
            println!("[telemetry] post to {} failed: {}", url, e);
        }
    });
}
