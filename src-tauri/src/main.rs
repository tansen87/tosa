#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod event_handle;
mod global;
mod hotkey;
mod tray;
mod window;

use log::info;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_context_menu::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_single_instance::init(|app, _, _cwd| {
            let win = app.get_window(global::TRANSLATOR_LABEL).unwrap();
            win.show().unwrap();
            win.set_focus().unwrap();
        }))
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([
                    tauri_plugin_log::LogTarget::LogDir,
                    tauri_plugin_log::LogTarget::Stdout,
                ])
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .system_tray(tauri::SystemTray::new())
        .setup(|app| {
            global::APP.get_or_init(|| app.handle());
            let base_path = dirs::config_dir()
                .unwrap()
                .join(app.config().tauri.bundle.identifier.clone());
            global::BASE_PATH.get_or_init(|| base_path);

            info!("init config store");
            config::init_config();

            tray::generate_tray(app.app_handle());
            hotkey::init_hotkey();

            std::thread::spawn(move || {
                window::create_trans_window();
                window::create_screenshot_window();
                //window::create_mini_trans_window();
                //window::show_mini_trans_window();

                if config::is_first_run() {
                    info!("First Run, opening config window");
                    window::create_setting_window();
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::get_config,
            config::set_config_by_key,
            window::show_trans_win,
            window::show_setting_window,
            event_handle::screenps,
        ])
        .on_system_tray_event(event_handle::tray_event_handler)
        .build(tauri::generate_context!())
        .expect("error while running application")
        .run(|app, event| match event {
            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } => {
                if label != global::TRANSLATOR_LABEL {
                    return;
                }
                #[cfg(target_os = "macos")]
                {
                    tauri::AppHandle::hide(&app.app_handle()).unwrap();
                }
                #[cfg(not(target_os = "macos"))]
                {
                    let window = app.get_window(label.as_str()).unwrap();
                    window.hide().unwrap();
                }
                api.prevent_close();
            }
            _ => {}
        })
}
