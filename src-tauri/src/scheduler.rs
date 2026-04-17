use crate::state::AppState;
use crate::idle_detector;
use crate::commands::system::is_fullscreen;
use tauri::{AppHandle, Manager};
use std::time::Duration;

const IDLE_POLL_SECS: u64 = 5;

fn seconds_until_next_10am() -> u64 {
    let now = chrono::Local::now();
    let today_10am = now.date_naive().and_hms_opt(10, 0, 0).unwrap();
    let target = if now.naive_local() < today_10am {
        today_10am
    } else {
        today_10am + chrono::Duration::days(1)
    };
    let diff = target - now.naive_local();
    diff.num_seconds().max(60) as u64
}

pub fn spawn_wallpaper_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // 首次更新
        if let Err(e) = crate::commands::wallpaper::update_wallpaper(app.clone()).await {
            log::warn!("First wallpaper update failed: {e}");
        } else {
            log::info!("First wallpaper update succeeded");
        }

        // 每天 10 点更新
        loop {
            let wait = seconds_until_next_10am();
            log::info!("Next wallpaper update in {}h {}m", wait / 3600, (wait % 3600) / 60);
            tokio::time::sleep(Duration::from_secs(wait)).await;
            log::info!("Scheduled daily wallpaper update");
            if let Err(e) = crate::commands::wallpaper::update_wallpaper(app.clone()).await {
                log::warn!("Scheduled update failed: {e}");
            }
        }
    });
}

pub fn spawn_rest_monitor(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(IDLE_POLL_SECS)).await;

            let state: AppState = app.state::<AppState>().inner().clone();
            let (enabled, interval, idle_reset, is_showing) = {
                let s = state.lock();
                (
                    s.is_rest_enabled,
                    s.rest_interval_seconds,
                    s.idle_reset_seconds,
                    s.is_overlay_showing,
                )
            };

            if !enabled || is_showing {
                continue;
            }

            let idle_secs = match idle_detector::get_idle_seconds() {
                Ok(s) => s,
                Err(_) => continue,
            };

            if idle_secs >= idle_reset {
                let mut s = state.lock();
                s.last_activity_time = std::time::Instant::now();
                s.last_rest_time = std::time::Instant::now();
                continue;
            }

            let work_duration = {
                let s = state.lock();
                s.last_rest_time.elapsed().as_secs()
            };

            if work_duration >= interval {
                if is_fullscreen() {
                    continue;
                }

                log::info!("Rest reminder triggered after {work_duration}s of work");

                {
                    let mut s = state.lock();
                    s.is_overlay_showing = true;
                }

                if let Some(win) = app.get_webview_window("rest-overlay") {
                    let _ = win.set_focus();
                } else {
                    let _ = tauri::WebviewWindowBuilder::new(
                        &app,
                        "rest-overlay",
                        tauri::WebviewUrl::App("src/rest-overlay.html".into()),
                    )
                    .title("休息提醒")
                    .fullscreen(true)
                    .decorations(false)
                    .always_on_top(true)
                    .build();
                }

                let _ = crate::tray::rebuild_tray_menu(&app);
            }
        }
    });
}

#[tauri::command]
pub fn rest_completed(state: tauri::State<'_, AppState>) {
    let mut s = state.inner().lock();
    s.is_overlay_showing = false;
    s.last_rest_time = std::time::Instant::now();
    s.last_activity_time = std::time::Instant::now();
}
