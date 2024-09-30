use crate::{
    config::get_or_bool, global::*, hotkey, window::*, PP_OCR,
};
use base64::{engine::general_purpose, Engine as _};
use log::debug;
use mouse_position::mouse_position::Mouse;
use screenshots::{Compression, Screen};
use tauri::{AppHandle, SystemTrayEvent};

fn handlle_hotkey_name_event(name: &str) {
    debug!("hotkey event: {}", name);
    match name {
        SHOW_TRANSLATOR => {
            show_trans_win(true);
        }
        SCREENSHOT_RECOGNIZER => {
            debug!("{}", name);
            screenshot(name);
        }
        _ => {}
    }
}

pub fn screenshot(name: &str) {
    debug!("screenshot: {}", name);
    if let Some(w) = get_window(TRANSLATOR_LABEL) {
        w.hide().unwrap();
    }
    if let Some(w) = get_window(SCREEN_CAPTURE_LABEL) {
        if w.is_visible().unwrap() {
            return;
        }
    }
    let start = std::time::Instant::now();
    let (x, y) = match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => (x, y),
        Mouse::Error => (0, 0),
    };
    let ocr_err_tip = get_or_bool("ocr_err_tip", false);
    let screen = match Screen::from_point(x, y) {
        Ok(s) => s,
        Err(_) => {
            debug!("无法获取鼠标所在屏幕");
            if ocr_err_tip {
                message_box("错误", "无法获取鼠标所在屏幕");
            }
            return;
        }
    };
    let image = match screen.capture() {
        Ok(i) => i,
        Err(e) => {
            debug!("截图失败: {}", e);
            if ocr_err_tip {
                message_box("错误", format!("截图失败: {}", e).as_str());
            }
            return;
        }
    };
    let bytes = image.to_png(Compression::Fast).unwrap();
    let base64 = general_purpose::STANDARD.encode(&bytes);
    debug!("screenshot: {:?}", start.elapsed());
    emit_to(
        SCREEN_CAPTURE_LABEL,
        "screenshot://capture",
        serde_json::json!({
            "name": name,
            "base64": base64,
        }),
    );
}

#[tauri::command]
pub async fn screenps(clip: String) -> String {
    let json_data = match tauri::async_runtime::spawn_blocking(move || {
        let mut pp_ocr = PP_OCR.lock().unwrap();
        pp_ocr.ocr(paddleocr::ImageData::from_base64(clip))
    }).await.unwrap() {
        Ok(res) => res,
        Err(err) => {
            eprintln!("OCR error: {err}");
            return err.to_string();
        }
    };

    let mut concatenated_text = String::new();
    let v: serde_json::Value = serde_json::from_str(&json_data).unwrap();
    if v["code"] == 100 {
        let data = v["data"].as_array().unwrap();
        for (index, item) in data.iter().enumerate() {
            if let Some(text) = item["text"].as_str() {
                concatenated_text.push_str(text);
                if index != data.len() - 1 {
                    concatenated_text.push('\n');
                }
            }
        }
    } else {
        if let Some(data) = v["data"].as_str() {
            concatenated_text.push_str(data);
        }
    };

    concatenated_text
}

pub fn tray_event_handler(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::DoubleClick {
            position: _,
            size: _,
            ..
        } => {
            show_trans_win(true);
        }
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "setting" => {
                show_setting_window();
            }
            "relaunch" => {
                app.restart();
            }
            "quit" => {
                std::process::exit(0);
            }
            name => {
                handlle_hotkey_name_event(name);
            }
        },
        _ => {}
    }
}

pub fn handle_hotkey(name: String, key: String) {
    debug!("hotkey {} callback", key.clone());
    handlle_hotkey_name_event(name.as_str());
}

// pub fn handle_ahk_event(line: String) {
//     if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line.clone()) {
//         if let Some(action) = v.get("action") {
//             let action = action.as_str().unwrap();
//             if action == "not_running" {
//                 AHK_STATE.store(false, std::sync::atomic::Ordering::SeqCst);
//                 return;
//             }
//             if action == "running" {
//                 AHK_STATE.store(true, std::sync::atomic::Ordering::SeqCst);
//                 return;
//             }
//             handlle_hotkey_name_event(action);
//         }
//     }
//     debug!("ahk event: {}", line.clone());
//     emit_to(SETTING_LABEL, "ahk://event", line);
// }

pub fn handle_config_change(key: String, value: serde_json::Value) {
    match key.as_str() {
        SHOW_TRANSLATOR | SCREENSHOT_RECOGNIZER => {
            hotkey::init_hotkey();
        }
        _ => {}
    }

    debug!("config change: {} {:?}", key, value);
}
