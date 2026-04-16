use crate::state::AppState;
use crate::idle_detector;
use crate::commands::system::is_fullscreen;
use tauri::{AppHandle, Manager};
use std::time::Duration;

const UPDATE_INTERVAL_SECS: u64 = 3 * 60 * 60; // 3 hours
const RETRY_INTERVAL_SECS: u64 = 30;
const IDLE_POLL_SECS: u64 = 5;

/// 壁纸定时刷新调度器
pub fn spawn_wallpaper_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // 等待网络就绪
        loop {
            match reqwest::get("https://www.bing.com").await {
                Ok(_) => break,
                Err(_) => {
                    log::info!("No network, retrying in 60s...");
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            }
        }

        log::info!("Network ready, starting first wallpaper update");

        // 首次更新（失败重试）
        loop {
            match crate::commands::wallpaper::update_wallpaper(app.clone(), false).await {
                Ok(_) => {
                    log::info!("First wallpaper update succeeded");
                    break;
                }
                Err(e) => {
                    log::warn!("Wallpaper update failed: {e}, retrying in {RETRY_INTERVAL_SECS}s");
                    tokio::time::sleep(Duration::from_secs(RETRY_INTERVAL_SECS)).await;
                }
            }
        }

        // 每 3 小时循环
        loop {
            tokio::time::sleep(Duration::from_secs(UPDATE_INTERVAL_SECS)).await;
            log::info!("Scheduled wallpaper update");
            if let Err(e) = crate::commands::wallpaper::update_wallpaper(app.clone(), false).await {
                log::warn!("Scheduled update failed: {e}");
            }
        }
    });
}

/// 休息提醒监控器
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

            // 检测空闲时间
            let idle_secs = match idle_detector::get_idle_seconds() {
                Ok(s) => s,
                Err(_) => continue,
            };

            // 空闲超过阈值，重置计时
            if idle_secs >= idle_reset {
                let mut s = state.lock();
                s.last_activity_time = std::time::Instant::now();
                s.last_rest_time = std::time::Instant::now();
                continue;
            }

            // 检查连续工作时间
            let work_duration = {
                let s = state.lock();
                s.last_rest_time.elapsed().as_secs()
            };

            if work_duration >= interval {
                // 全屏时不弹出（仅 Windows）
                if is_fullscreen() {
                    continue;
                }

                log::info!("Rest reminder triggered after {work_duration}s of work");

                // 标记遮罩正在显示
                {
                    let mut s = state.lock();
                    s.is_overlay_showing = true;
                }

                // 打开休息覆盖层窗口
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

                // 更新托盘菜单
                let _ = crate::tray::rebuild_tray_menu(&app);
            }
        }
    });
}

/// 休息结束后调用，重置计时
#[tauri::command]
pub fn rest_completed(state: tauri::State<'_, AppState>) {
    let mut s = state.inner().lock();
    s.is_overlay_showing = false;
    s.last_rest_time = std::time::Instant::now();
    s.last_activity_time = std::time::Instant::now();
}
