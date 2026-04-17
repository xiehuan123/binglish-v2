mod commands;
mod idle_detector;
mod scheduler;
mod state;
mod text_renderer;
mod tray;
mod wallpaper_setter;
mod word_db;

use state::{AppState, AppStateInner, WallpaperMode};
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let app_state: AppState = Arc::new(parking_lot::Mutex::new(AppStateInner::default()));
    let word_db = word_db::WordDb::load();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(app_state)
        .manage(word_db)
        .invoke_handler(tauri::generate_handler![
            commands::wallpaper::update_wallpaper,
            commands::wallpaper::copy_wallpaper,
            commands::wallpaper::set_custom_wallpaper,
            commands::wallpaper::clear_custom_wallpaper,
            commands::wallpaper::get_current_word,
            commands::wallpaper::get_word_page,
            commands::games::get_game_data,
            commands::system::is_fullscreen,
            scheduler::rest_completed,
        ])
        .setup(|app| {
            let _tray = tray::create_tray(&app.handle())?;
            load_config(app.handle());
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
        if let Some(val) = store.get("wallpaper_mode") {
            if let Ok(mode) = serde_json::from_value::<WallpaperMode>(val.clone()) {
                state.lock().wallpaper_mode = mode;
            }
        }
        if let Some(val) = store.get("custom_image_path") {
            if let Some(s) = val.as_str() {
                state.lock().custom_image_path = Some(s.to_string());
            }
        }
    }
}
