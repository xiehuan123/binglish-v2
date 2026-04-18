use crate::state::{AppState, WallpaperMode};
use crate::wallpaper_setter;
use crate::word_db::WordDb;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn get_screen_size() -> (u32, u32) {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Graphics::Gdi::{
            GetDC, GetDeviceCaps, ReleaseDC, DESKTOPHORZRES, DESKTOPVERTRES,
        };
        unsafe {
            let dc = GetDC(None);
            let w = GetDeviceCaps(dc, DESKTOPHORZRES);
            let h = GetDeviceCaps(dc, DESKTOPVERTRES);
            let _ = ReleaseDC(None, dc);
            (w as u32, h as u32)
        }
    }
    #[cfg(target_os = "macos")]
    {
        use std::ffi::c_void;
        extern "C" {
            fn CGGetActiveDisplayList(max: u32, displays: *mut u32, count: *mut u32) -> i32;
            fn CGDisplayCopyDisplayMode(display: u32) -> *const c_void;
            fn CGDisplayModeGetPixelWidth(mode: *const c_void) -> usize;
            fn CGDisplayModeGetPixelHeight(mode: *const c_void) -> usize;
            fn CGDisplayModeRelease(mode: *const c_void);
        }
        unsafe {
            let mut ids = [0u32; 16];
            let mut count = 0u32;
            CGGetActiveDisplayList(16, ids.as_mut_ptr(), &mut count);
            let mut max_w = 0u32;
            let mut max_h = 0u32;
            for i in 0..count as usize {
                let mode = CGDisplayCopyDisplayMode(ids[i]);
                if !mode.is_null() {
                    let w = CGDisplayModeGetPixelWidth(mode) as u32;
                    let h = CGDisplayModeGetPixelHeight(mode) as u32;
                    CGDisplayModeRelease(mode);
                    max_w = max_w.max(w);
                    max_h = max_h.max(h);
                }
            }
            log::info!("Screen size (max of {} displays): {max_w}x{max_h}", count);
            if max_w > 0 && max_h > 0 { return (max_w, max_h); }
        }
        (2560, 1600)
    }
}

fn remove_files_with_prefix(dir: &std::path::Path, prefix: &str) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        if entry.file_name().to_string_lossy().starts_with(prefix) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

async fn download_bing_wallpaper(w: u32, h: u32) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let url = format!("https://picsum.photos/1920/1080");
    log::info!("Downloading wallpaper: {url}");
    download_image(&client, &url).await
}

async fn try_bing_official(client: &reqwest::Client, api_url: &str, w: u32, h: u32) -> Result<String, String> {
    let resp = client.get(api_url).send().await.map_err(|e| e.to_string())?;
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let base = json.pointer("/images/0/url")
        .and_then(|v| v.as_str())
        .ok_or("No image URL in Bing API response")?;
    let url = if base.starts_with("http") {
        base.to_string()
    } else {
        format!("https://www.bing.com{base}")
    };
    let url = url.replace("1920x1080", &format!("{w}x{h}"));
    Ok(url)
}

async fn download_image(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    let resp = client.get(url).send().await.map_err(|e| format!("Download failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let bytes = resp.bytes().await.map_err(|e| format!("Read failed: {e}"))?;
    if bytes.len() < 10000 {
        return Err("Image too small, likely not a valid image".to_string());
    }
    Ok(bytes.to_vec())
}

#[tauri::command]
pub async fn update_wallpaper(app: AppHandle) -> Result<String, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    let save_path = data_dir.join("wallpaper.jpg");
    let (screen_w, screen_h) = get_screen_size();

    // 先从词库选词（在 await 之前完成，避免生命周期问题）
    let word_db: tauri::State<'_, WordDb> = app.state();
    let entry = word_db.random_word().ok_or("Word database is empty")?.clone();
    drop(word_db);

    let state: AppState = app.state::<AppState>().inner().clone();
    let (mode, custom_path) = {
        let s = state.lock();
        (s.wallpaper_mode.clone(), s.custom_image_path.clone())
    };

    let base_path = match mode {
        WallpaperMode::Custom => {
            match custom_path {
                Some(p) if std::path::Path::new(&p).exists() => PathBuf::from(p),
                _ => return Err("Custom image not found".to_string()),
            }
        }
        WallpaperMode::Normal => {
            let bing_path = data_dir.join("bing_base.jpg");
            let bytes = download_bing_wallpaper(screen_w, screen_h).await?;
            std::fs::write(&bing_path, &bytes).map_err(|e| format!("Save failed: {e}"))?;
            log::info!("Wallpaper downloaded: {}KB", bytes.len() / 1024);
            bing_path
        }
    };

    log::info!("Selected word: {} [{}]", entry.word, entry.phonetic);

    let desc = if entry.phonetic.is_empty() {
        entry.trans.clone()
    } else {
        format!("/{}/ {}", entry.phonetic, entry.trans)
    };

    let card = crate::text_renderer::WordCard {
        word: entry.word.clone(),
        desc: if desc.is_empty() { None } else { Some(desc) },
        sentence_en: if entry.sentence_en.is_empty() { None } else { Some(entry.sentence_en.clone()) },
        sentence_cn: if entry.sentence_cn.is_empty() { None } else { Some(entry.sentence_cn.clone()) },
    };

    crate::text_renderer::render_word_on_image(&base_path, &card, &save_path, screen_w, screen_h)?;

    wallpaper_setter::set_wallpaper(&save_path)?;
    log::info!("Wallpaper set: {}", entry.word);

    {
        let mut s = state.lock();
        s.current_word = Some(entry.word.clone());
    }

    let _ = crate::tray::rebuild_tray_menu(&app);
    Ok(entry.word.clone())
}

#[tauri::command]
pub fn copy_wallpaper(app: AppHandle, dest: String) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let src = data_dir.join("wallpaper.jpg");
    if !src.exists() {
        return Err("Wallpaper file not found".to_string());
    }
    std::fs::copy(&src, &dest).map_err(|e| format!("Copy failed: {e}"))?;
    Ok(())
}

pub fn save_wallpaper_config(app: &AppHandle) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    let state: AppState = app.state::<AppState>().inner().clone();
    let (mode, path) = {
        let s = state.lock();
        (s.wallpaper_mode.clone(), s.custom_image_path.clone())
    };
    let store = app.store("config.json").map_err(|e| e.to_string())?;
    store.set("wallpaper_mode", serde_json::to_value(&mode).unwrap());
    store.set("custom_image_path", serde_json::to_value(&path).unwrap());
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn set_custom_wallpaper(app: AppHandle, image_path: String) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let src = std::path::Path::new(&image_path);
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
    let custom_base = data_dir.join(format!("custom_base.{ext}"));

    remove_files_with_prefix(&data_dir, "custom_base.");

    std::fs::copy(&image_path, &custom_base).map_err(|e| format!("Copy image failed: {e}"))?;
    log::info!("Custom wallpaper copied from: {image_path}");

    {
        let state: AppState = app.state::<AppState>().inner().clone();
        let mut s = state.lock();
        s.wallpaper_mode = WallpaperMode::Custom;
        s.custom_image_path = Some(custom_base.to_string_lossy().to_string());
    }

    save_wallpaper_config(&app)?;
    update_wallpaper(app).await?;
    Ok(())
}

#[tauri::command]
pub async fn clear_custom_wallpaper(app: AppHandle) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    {
        let state: AppState = app.state::<AppState>().inner().clone();
        let mut s = state.lock();
        s.wallpaper_mode = WallpaperMode::Normal;
        s.custom_image_path = None;
    }

    remove_files_with_prefix(&data_dir, "custom_base.");
    save_wallpaper_config(&app)?;
    update_wallpaper(app).await?;
    Ok(())
}

#[tauri::command]
pub fn get_current_word(state: tauri::State<'_, AppState>) -> Option<String> {
    state.inner().lock().current_word.clone()
}

#[derive(serde::Serialize)]
pub struct WordPage {
    pub words: Vec<WordPageItem>,
    pub current_page: usize,
    pub total_pages: usize,
}

#[derive(serde::Serialize)]
pub struct WordPageItem {
    pub word: String,
    pub phonetic: String,
    pub trans: String,
}

#[tauri::command]
pub fn get_word_page(word_db: tauri::State<'_, WordDb>, page: usize, page_size: usize) -> WordPage {
    let size = if page_size == 0 { 5 } else { page_size };
    let total = word_db.total_words();
    let total_pages = (total + size - 1) / size;
    let p = page.min(total_pages.saturating_sub(1));
    let entries = word_db.get_page(p, size);
    let words = entries.into_iter().map(|e| {
        let trans_short: String = e.trans
            .split(';')
            .take(2)
            .map(|s| {
                // 去掉括号内的详细解释
                let mut result = String::new();
                let mut depth = 0i32;
                for ch in s.trim().chars() {
                    match ch {
                        '（' | '(' => depth += 1,
                        '）' | ')' => { depth -= 1; continue; }
                        _ if depth > 0 => continue,
                        _ => result.push(ch),
                    }
                }
                // 中文逗号分隔的子义项只取前2个
                let parts: Vec<&str> = result.split('，').collect();
                if parts.len() > 2 {
                    parts[..2].join("，")
                } else {
                    result
                }
            })
            .collect::<Vec<_>>()
            .join("; ");
        WordPageItem {
            word: e.word,
            phonetic: e.phonetic,
            trans: trans_short,
        }
    }).collect();
    WordPage { words, current_page: p, total_pages }
}
