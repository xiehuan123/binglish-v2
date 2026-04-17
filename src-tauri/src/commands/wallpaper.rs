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
    #[cfg(not(target_os = "windows"))]
    {
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
    use rand::Rng;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let idx = rand::thread_rng().gen_range(0..8);

    // 源1: picsum.photos
    let source1 = format!("https://picsum.photos/{w}/{h}");
    log::info!("Trying picsum.photos: {source1}");
    if let Ok(bytes) = download_image(&client, &source1).await {
        return Ok(bytes);
    }

    // 源2: Bing 官方 API（随机最近 8 天）
    let bing_api = format!(
        "https://www.bing.com/HPImageArchive.aspx?format=js&idx={idx}&n=1&mkt=zh-CN"
    );
    if let Ok(url) = try_bing_official(&client, &bing_api, w, h).await {
        log::info!("Trying Bing official (idx={idx}): {url}");
        if let Ok(bytes) = download_image(&client, &url).await {
            return Ok(bytes);
        }
    }

    // 源3: bingw.jasonzeng.dev
    let source3 = format!("https://bingw.jasonzeng.dev/?index=random&w={w}&h={h}");
    log::info!("Trying bingw.jasonzeng.dev");
    if let Ok(bytes) = download_image(&client, &source3).await {
        return Ok(bytes);
    }

    // 源4: bing.img.run
    log::info!("Trying bing.img.run");
    if let Ok(bytes) = download_image(&client, "https://bing.img.run/1920x1080.php").await {
        return Ok(bytes);
    }

    // 源5: Bing 直链
    log::info!("Trying Bing direct OHR");
    if let Ok(bytes) = download_image(&client, "https://www.bing.com/th?id=OHR.POTD_zhCN&w=1920&h=1080&c=7&rs=1&qlt=80").await {
        return Ok(bytes);
    }

    Err("All wallpaper sources failed".to_string())
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

    let word_db: tauri::State<'_, WordDb> = app.state();
    let entry = word_db.random_word().ok_or("Word database is empty")?;
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
