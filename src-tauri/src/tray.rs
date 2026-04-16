use crate::state::AppState;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Emitter, Manager, Wry,
};

const PROJECT_URL: &str = "https://github.com/klemperer/binglish";

pub fn create_tray(app: &AppHandle) -> Result<TrayIcon, tauri::Error> {
    let menu = build_menu(app)?;
    let icon = app.default_window_icon().cloned().expect("no app icon found");
    TrayIconBuilder::new()
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

    if let Some(ref word) = s.bing_word {
        if s.bing_url.is_some() {
            menu.append(&MenuItem::with_id(
                app, "lookup_word",
                format!("查单词 {word}"), true, None::<&str>,
            )?)?;
        }
        if s.bing_mp3.is_some() {
            menu.append(&MenuItem::with_id(
                app, "listen_word",
                format!("听单词 {word}"), true, None::<&str>,
            )?)?;
        }
        menu.append(&MenuItem::with_id(
            app, "watch_word",
            format!("看单词 {word}"), true, None::<&str>,
        )?)?;
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }

    menu.append(&MenuItem::with_id(app, "random_review", "随机复习", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "copy_save", "复制保存", true, None::<&str>)?)?;

    if s.bing_copyright.is_some() {
        menu.append(&MenuItem::with_id(app, "wallpaper_info", "壁纸信息", true, None::<&str>)?)?;
    }
    if s.bing_id.is_some() {
        menu.append(&MenuItem::with_id(app, "share_wallpaper", "分享壁纸", true, None::<&str>)?)?;
    }

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
    menu.append(&MenuItem::with_id(app, "history", "Today in History", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "games", "Binglish Games", true, None::<&str>)?)?;

    if let Some(ref name) = s.music_name {
        if s.music_url.is_some() {
            menu.append(&MenuItem::with_id(app, "music_header", "==Song of the Day==", false, None::<&str>)?)?;
            menu.append(&MenuItem::with_id(app, "music_name", format!("  {name}"), false, None::<&str>)?)?;
            let play_text = if s.is_music_playing { "  停止播放" } else { "  播放歌曲" };
            menu.append(&MenuItem::with_id(app, "toggle_music", play_text, true, None::<&str>)?)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
        }
    }

    menu.append(&CheckMenuItem::with_id(app, "autostart", "开机运行", true, false, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "about", "关于", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?)?;

    Ok(menu)
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    let state = get_state(app);
    match id {
        "lookup_word" => {
            if let Some(ref url) = state.lock().bing_url {
                let _ = tauri_plugin_shell::ShellExt::shell(app).open(url, None);
            }
        }
        "listen_word" => {
            let mp3 = state.lock().bing_mp3.clone();
            if let Some(mp3_url) = mp3 {
                let _ = app.emit("play-word-audio", mp3_url);
            }
        }
        "watch_word" => {
            let word = state.lock().bing_word.clone();
            if let Some(word) = word {
                let url = format!("https://www.playphrase.me/#/search?q={word}&language=en");
                let _ = tauri_plugin_shell::ShellExt::shell(app).open(&url, None);
            }
        }
        "random_review" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = crate::commands::wallpaper::update_wallpaper(app, true).await;
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
        "wallpaper_info" => {
            let (copyright, copyright_url) = {
                let s = state.lock();
                (s.bing_copyright.clone().unwrap_or_default(), s.bing_copyright_url.clone())
            };
            use tauri_plugin_dialog::DialogExt;
            if let Some(url) = copyright_url {
                let app_clone = app.clone();
                app.dialog()
                    .message(&format!("{copyright}\n\n查看相关信息？"))
                    .title("壁纸信息")
                    .blocking_show();
                let _ = tauri_plugin_shell::ShellExt::shell(&app_clone).open(&url, None);
            } else {
                app.dialog()
                    .message(&copyright)
                    .title("壁纸信息")
                    .blocking_show();
            }
        }
        "share_wallpaper" => {
            open_overlay_window(app, "game-overlay", "src/game-overlay.html", "分享壁纸");
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
        "history" => {
            open_overlay_window(app, "history-overlay", "src/history-overlay.html", "Today in History");
        }
        "games" => {
            open_overlay_window(app, "game-overlay", "src/game-overlay.html", "Binglish Games");
        }
        "toggle_music" => {
            // 直接操作 state 而不是通过 tauri command
            let _ = crate::commands::audio::toggle_music_inner(&state);
            let _ = rebuild_tray_menu(app);
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
