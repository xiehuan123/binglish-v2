use crate::state::{AppState, WallpaperMode};
use std::path::PathBuf;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager, Wry,
};

const PROJECT_URL: &str = "https://github.com/xiehuan123/binglish-v2";

pub fn create_tray(app: &AppHandle) -> Result<TrayIcon, tauri::Error> {
    let menu = build_menu(app)?;
    let icon = app.default_window_icon().cloned().expect("no app icon found");
    TrayIconBuilder::with_id("main")
        .icon(icon)
        .menu(&menu)
        .tooltip("Binglish")
        .on_menu_event(move |app, event| {
            handle_menu_event(app, event.id().as_ref());
        })
        .build(app)
}

pub fn rebuild_tray_menu(app: &AppHandle) -> Result<(), String> {
    let menu = build_menu(app).map_err(|e| e.to_string())?;
    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn get_state(app: &AppHandle) -> AppState {
    app.state::<AppState>().inner().clone()
}

fn build_menu(app: &AppHandle) -> Result<Menu<Wry>, tauri::Error> {
    let state = get_state(app);
    let s = state.lock();

    let menu = Menu::new(app)?;

    if let Some(ref word) = s.current_word {
        menu.append(&MenuItem::with_id(
            app, "current_word",
            format!("当前单词: {word}"), false, None::<&str>,
        )?)?;
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }

    menu.append(&MenuItem::with_id(app, "next_word", "换个单词", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "copy_save", "复制保存", true, None::<&str>)?)?;

    let custom_label = if s.wallpaper_mode == WallpaperMode::Custom {
        "取消自定义壁纸"
    } else {
        "自定义壁纸"
    };
    menu.append(&MenuItem::with_id(app, "custom_wallpaper", custom_label, true, None::<&str>)?)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    let rest_label = if s.is_rest_enabled {
        let elapsed = s.last_rest_time.elapsed().as_secs();
        let remaining = s.rest_interval_seconds.saturating_sub(elapsed);
        format!("提醒休息 (剩余{}分)", remaining / 60)
    } else {
        "提醒休息".to_string()
    };
    menu.append(&CheckMenuItem::with_id(
        app, "toggle_rest", &rest_label, true, s.is_rest_enabled, None::<&str>,
    )?)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "games", "英语小游戏", true, None::<&str>)?)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&CheckMenuItem::with_id(app, "autostart", "开机运行", true, false, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "about", "关于", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?)?;

    Ok(menu)
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    let state = get_state(app);
    match id {
        "next_word" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = crate::commands::wallpaper::update_wallpaper(app).await {
                    log::error!("Update wallpaper failed: {e}");
                }
            });
        }
        "copy_save" => {
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                use tauri_plugin_dialog::DialogExt;
                if let Some(path) = app_clone.dialog().file().blocking_save_file() {
                    let _ = crate::commands::wallpaper::copy_wallpaper(
                        app_clone,
                        path.to_string(),
                    );
                }
            });
        }
        "toggle_rest" => {
            {
                let mut s = state.lock();
                s.is_rest_enabled = !s.is_rest_enabled;
                if s.is_rest_enabled {
                    s.last_rest_time = std::time::Instant::now();
                    s.last_activity_time = std::time::Instant::now();
                }
            }
            let _ = rebuild_tray_menu(app);
        }
        "games" => {
            open_overlay_window(app, "game-overlay", "src/game-overlay.html", "英语小游戏");
        }
        "custom_wallpaper" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let state = get_state(&app);
                let mode = state.lock().wallpaper_mode.clone();
                if mode == WallpaperMode::Custom {
                    if let Err(e) = crate::commands::wallpaper::clear_custom_wallpaper(app.clone()).await {
                        log::error!("Clear custom wallpaper failed: {e}");
                    }
                } else {
                    use tauri_plugin_dialog::DialogExt;
                    let file = app
                        .dialog()
                        .file()
                        .add_filter("Images", &["jpg", "jpeg", "png", "bmp"])
                        .blocking_pick_file();
                    if let Some(f) = file {
                        match PathBuf::try_from(f) {
                            Ok(path) => {
                                let path_str = path.to_string_lossy().to_string();
                                if let Err(e) = crate::commands::wallpaper::set_custom_wallpaper(app.clone(), path_str).await {
                                    log::error!("Set custom wallpaper failed: {e}");
                                }
                            }
                            Err(e) => log::error!("Invalid file path: {e}"),
                        }
                    }
                }
                let _ = rebuild_tray_menu(&app);
            });
        }
        "about" => {
            use tauri_plugin_dialog::DialogExt;
            app.dialog()
                .message(&format!("Binglish桌面英语 2.0.0\n{PROJECT_URL}"))
                .title("关于 Binglish")
                .blocking_show();
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

fn open_overlay_window(app: &AppHandle, label: &str, url: &str, title: &str) {
    if let Some(win) = app.get_webview_window(label) {
        let _ = win.set_focus();
        return;
    }
    let _ = tauri::WebviewWindowBuilder::new(app, label, tauri::WebviewUrl::App(url.into()))
        .title(title)
        .fullscreen(true)
        .decorations(false)
        .always_on_top(true)
        .build();
}
