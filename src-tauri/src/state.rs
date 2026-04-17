use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperMode {
    Normal,
    Custom,
}

impl Default for WallpaperMode {
    fn default() -> Self {
        Self::Normal
    }
}

pub struct AppStateInner {
    // 当前单词
    pub current_word: Option<String>,
    // 休息提醒
    pub is_rest_enabled: bool,
    pub last_activity_time: Instant,
    pub last_rest_time: Instant,
    pub is_overlay_showing: bool,
    pub rest_interval_seconds: u64,
    pub idle_reset_seconds: u64,
    pub rest_lock_seconds: u64,
    pub overlay_color: String,
    // 壁纸模式
    pub wallpaper_mode: WallpaperMode,
    pub custom_image_path: Option<String>,
}

impl Default for AppStateInner {
    fn default() -> Self {
        Self {
            current_word: None,
            is_rest_enabled: false,
            last_activity_time: Instant::now(),
            last_rest_time: Instant::now(),
            is_overlay_showing: false,
            rest_interval_seconds: 2700,
            idle_reset_seconds: 300,
            rest_lock_seconds: 30,
            overlay_color: "#2C3E50".to_string(),
            wallpaper_mode: WallpaperMode::default(),
            custom_image_path: None,
        }
    }
}

pub type AppState = Arc<Mutex<AppStateInner>>;
