mod commands;
mod idle_detector;
mod scheduler;
mod state;
mod tray;
mod wallpaper_setter;

use state::{AppState, AppStateInner};
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let app_state: AppState = Arc::new(parking_lot::Mutex::new(AppStateInner::default()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::wallpaper::update_wallpaper,
            commands::wallpaper::get_wallpaper_info,
            commands::wallpaper::copy_wallpaper,
            commands::audio::toggle_music,
            commands::audio::is_music_playing,
            commands::history::get_history_today,
            commands::games::get_game_data,
            commands::system::is_fullscreen,
            commands::system::get_useless_fact,
            scheduler::rest_completed,
        ])
        .setup(|app| {
            // 创建系统托盘
            let _tray = tray::create_tray(&app.handle())?;

            // 加载持久化配置
            load_config(app.handle());

            // 启动调度器
            scheduler::spawn_wallpaper_scheduler(app.handle().clone());
            scheduler::spawn_rest_monitor(app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Binglish");
}

fn load_config(app: &tauri::AppHandle) {
    use tauri_plugin_store::StoreExt;

    let state: AppState = app.state::<AppState>().inner().clone();
    if let Ok(store) = app.store("config.json") {
        if let Some(val) = store.get("rest_enabled") {
            if let Some(b) = val.as_bool() {
                state.lock().is_rest_enabled = b;
            }
        }
        if let Some(val) = store.get("rest_interval") {
            if let Some(n) = val.as_u64() {
                state.lock().rest_interval_seconds = n;
            }
        }
        if let Some(val) = store.get("overlay_color") {
            if let Some(s) = val.as_str() {
                state.lock().overlay_color = s.to_string();
            }
        }
    }
}
