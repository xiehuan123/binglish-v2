mod commands;
mod idle_detector;
mod review_store;
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
    let word_db = word_db::load_word_db();

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
            commands::review::add_word_to_learning,
            commands::review::add_words_batch,
            commands::review::get_due_words,
            commands::review::get_due_count,
            commands::review::submit_review,
            commands::review::get_learning_stats,
            commands::review::get_learning_words,
            commands::review::remove_word_from_learning,
            commands::review::is_word_in_learning,
            commands::review::get_new_words,
            commands::review::commit_new_words,
            commands::review::get_today_new_count,
            commands::review::set_daily_limit,
            commands::review::set_learning_book,
            commands::review::get_learning_config,
            commands::review::get_book_info,
            scheduler::rest_completed,
        ])
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("no app data dir");
            std::fs::create_dir_all(&app_data_dir).ok();
            let review_store = review_store::create_review_store(app_data_dir);
            app.manage(review_store);

            let _tray = tray::create_tray(&app.handle())?;
            load_config(app.handle());
            scheduler::spawn_wallpaper_scheduler(app.handle().clone());
            scheduler::spawn_rest_monitor(app.handle().clone());
            scheduler::spawn_review_reminder(app.handle().clone());
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
