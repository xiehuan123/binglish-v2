use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

pub struct AppStateInner {
    // 壁纸数据
    pub bing_word: Option<String>,
    pub bing_url: Option<String>,
    pub bing_mp3: Option<String>,
    pub bing_copyright: Option<String>,
    pub bing_copyright_url: Option<String>,
    pub bing_id: Option<String>,
    // 音乐
    pub music_name: Option<String>,
    pub music_url: Option<String>,
    pub is_music_playing: bool,
    /// 发送 () 到此 channel 可停止当前播放
    pub music_stop_tx: Option<std::sync::mpsc::Sender<()>>,
    // 休息提醒
    pub is_rest_enabled: bool,
    pub last_activity_time: Instant,
    pub last_rest_time: Instant,
    pub is_overlay_showing: bool,
    pub rest_interval_seconds: u64,
    pub idle_reset_seconds: u64,
    pub rest_lock_seconds: u64,
    pub overlay_color: String,
}

impl Default for AppStateInner {
    fn default() -> Self {
        Self {
            bing_word: None,
            bing_url: None,
            bing_mp3: None,
            bing_copyright: None,
            bing_copyright_url: None,
            bing_id: None,
            music_name: None,
            music_url: None,
            is_music_playing: false,
            music_stop_tx: None,
            is_rest_enabled: false,
            last_activity_time: Instant::now(),
            last_rest_time: Instant::now(),
            is_overlay_showing: false,
            rest_interval_seconds: 2700,
            idle_reset_seconds: 300,
            rest_lock_seconds: 30,
            overlay_color: "#2C3E50".to_string(),
        }
    }
}

pub type AppState = Arc<Mutex<AppStateInner>>;
