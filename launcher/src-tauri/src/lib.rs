mod addons;
mod connect;
mod deep_link;
mod gameinfo;
mod ping;

fn patch_gameinfo_on_startup() {
    let game_dir_buf;
    let game_dir: &std::path::Path = if let Some(override_dir) = connect::get_game_dir_override() {
        game_dir_buf = override_dir;
        &game_dir_buf
    } else {
        game_dir_buf = match connect::find_deadlock_game_dir() {
            Ok(d) => d,
            Err(_) => return,
        };
        &game_dir_buf
    };
    match gameinfo::ensure_addonroot(game_dir) {
        Ok(true) => println!("[startup] Patched gameinfo.gi with addonroot"),
        Ok(false) => {}
        Err(e) => println!("[startup] Could not patch gameinfo.gi (game may be running): {}", e),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            for arg in &args {
                if arg.starts_with("partylock://") {
                    deep_link::handle_incoming_url(app, arg);
                }
            }
            deep_link::surface_main_window(app);
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(deep_link::DeepLinkStateContainer::new())
        .invoke_handler(tauri::generate_handler![
            connect::launch_deadlock,
            connect::get_detected_game_dir,
            connect::get_game_dir,
            connect::set_game_dir,
            connect::reset_game_dir,
            addons::prepare_and_connect,
            addons::prepare_and_connect_match,
            ping::ping_server,
            deep_link::auth_callback_ready,
        ])
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
            use tauri::Manager;
            use tauri_plugin_deep_link::DeepLinkExt;

            #[cfg(any(target_os = "linux", target_os = "windows"))]
            let _ = app.deep_link().register("partylock");

            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    deep_link::handle_incoming_url(&handle, url.as_str());
                }
            });

            if let Ok(store) =
                tauri_plugin_store::StoreBuilder::new(app.handle(), "settings.json").build()
            {
                if let Some(path) = store
                    .get("game_dir_override")
                    .and_then(|v| v.as_str().map(String::from))
                {
                    connect::set_game_dir_override(Some(std::path::PathBuf::from(path)));
                }
            }
            patch_gameinfo_on_startup();

            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let launch = MenuItem::with_id(app, "launch", "Launch Deadlock", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &launch, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("PartyLock")
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "launch" => {
                        let _ = open::that(format!("steam://run/{}", "1422450"));
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
