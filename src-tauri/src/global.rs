use once_cell::sync::OnceCell;
use serde_json::Value;
use std::{
    collections::HashMap,
    path::PathBuf,
    process::Child,
    sync::{atomic::AtomicBool, Arc, Mutex, RwLock},
};

pub const SHOW_TRANSLATOR: &str = "show_translator";
pub const SCREENSHOT_RECOGNIZER: &str = "screenshot_recognizer";

pub const TRANSLATOR_LABEL: &str = "translator";
pub const SETTING_LABEL: &str = "setting";
pub const SCREEN_CAPTURE_LABEL: &str = "screen-capture";

pub static APP: OnceCell<tauri::AppHandle> = OnceCell::new();
pub static BASE_PATH: OnceCell<PathBuf> = OnceCell::new();

lazy_static::lazy_static! {
    // 截图后的base64
    pub static ref OCR_BASE64: Mutex<String> = Mutex::new(String::new());

    pub static ref WORKER: Mutex<Option<Child>> = Mutex::new(None);
    pub static ref AHK_STATE: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    pub static ref STORE: RwLock<HashMap<String, Value>> = RwLock::new(HashMap::new());
}
