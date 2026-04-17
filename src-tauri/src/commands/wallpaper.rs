use crate::state::AppState;
use crate::wallpaper_setter;
use serde::Serialize;
use std::io::Cursor;
use tauri::{AppHandle, Manager};

const IMAGE_URL: &str = "https://ss.blueforge.org/bing";
const MUSIC_JSON_URL: &str = "https://ss.blueforge.org/bing/songoftheday.json";
const VERSION: &str = "2.0.1";

#[derive(Debug, Serialize, Clone)]
pub struct WallpaperInfo {
    pub word: Option<String>,
    pub url: Option<String>,
    pub mp3: Option<String>,
    pub copyright: Option<String>,
    pub copyright_url: Option<String>,
    pub id: Option<String>,
    pub music_name: Option<String>,
    pub music_url: Option<String>,
}

fn parse_exif(data: &[u8]) -> (Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>) {
    let cursor = Cursor::new(data);
    let exif = match exif::Reader::new().read_from_container(&mut std::io::BufReader::new(cursor)) {
        Ok(e) => e,
        Err(e) => {
            log::warn!("EXIF parse failed: {e}");
            return (None, None, None, None, None, None);
        }
    };

    let get_field = |tag: exif::Tag| -> Option<String> {
        exif.get_field(tag, exif::In::PRIMARY).and_then(|f| match &f.value {
            exif::Value::Ascii(v) if !v.is_empty() => {
                String::from_utf8(v[0].clone()).ok().map(|s| s.trim().to_string())
            }
            _ => None,
        }).filter(|s| !s.is_empty())
    };

    let word = get_field(exif::Tag::Artist);
    let url = get_field(exif::Tag::ImageDescription);
    let mp3 = get_field(exif::Tag(exif::Context::Tiff, 269));
    let id = get_field(exif::Tag::Software);

    let copyright_raw = get_field(exif::Tag::Copyright);
    let (copyright, copyright_url) = match copyright_raw {
        Some(raw) if raw.contains("||") => {
            let mut parts = raw.splitn(2, "||");
            let c = parts.next().map(|s| s.trim().to_string());
            let u = parts.next().map(|s| s.trim().to_string());
            (c, u)
        }
        other => (other, None),
    };

    (word, url, mp3, copyright, copyright_url, id)
}

#[tauri::command]
pub async fn update_wallpaper(app: AppHandle, is_random: bool) -> Result<WallpaperInfo, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    let save_path = data_dir.join("wallpaper.jpg");

    // 构建下载 URL
    let mut image_url = format!("{IMAGE_URL}?v={VERSION}");

    // 获取屏幕尺寸（通过前端传入或使用默认值）
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
            image_url.push_str(&format!("&w={w}&h={h}"));
        }
    }

    if is_random {
        image_url.push_str("&random");
    }

    log::info!("Downloading wallpaper from: {image_url}");

    // 下载壁纸
    let client = reqwest::Client::new();
    let img_bytes = client
        .get(&image_url)
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("Read body failed: {e}"))?;

    std::fs::write(&save_path, &img_bytes).map_err(|e| format!("Save failed: {e}"))?;
    log::info!("Wallpaper saved to: {}", save_path.display());

    // 解析 EXIF
    let (word, url, mp3, copyright, copyright_url, id) = parse_exif(&img_bytes);

    // 下载音乐信息
    let (music_name, music_url) = match client
        .get(MUSIC_JSON_URL)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => {
                let name = json.get("name").and_then(|v| v.as_str()).map(String::from);
                let url = json.get("url").and_then(|v| v.as_str()).map(String::from);
                log::info!("Music info: name={:?}, url={:?}", name, url);
                (name, url)
            }
            Err(e) => {
                log::warn!("Music JSON parse failed: {e}");
                (None, None)
            }
        },
        Err(e) => {
            log::warn!("Music JSON fetch failed: {e}");
            (None, None)
        }
    };

    // 设置壁纸
    wallpaper_setter::set_wallpaper(&save_path)?;
    log::info!("Wallpaper set successfully");

    // 更新共享状态
    let state: AppState = app.state::<AppState>().inner().clone();
    {
        let mut s = state.lock();
        s.bing_word = word.clone();
        s.bing_url = url.clone();
        s.bing_mp3 = mp3.clone();
        s.bing_copyright = copyright.clone();
        s.bing_copyright_url = copyright_url.clone();
        s.bing_id = id.clone();
        s.music_name = music_name.clone();
        s.music_url = music_url.clone();
    }

    // 触发托盘菜单重建
    let _ = crate::tray::rebuild_tray_menu(&app);

    Ok(WallpaperInfo {
        word,
        url,
        mp3,
        copyright,
        copyright_url,
        id,
        music_name,
        music_url,
    })
}

#[tauri::command]
pub fn get_wallpaper_info(state: tauri::State<'_, AppState>) -> WallpaperInfo {
    let s = state.inner().lock();
    WallpaperInfo {
        word: s.bing_word.clone(),
        url: s.bing_url.clone(),
        mp3: s.bing_mp3.clone(),
        copyright: s.bing_copyright.clone(),
        copyright_url: s.bing_copyright_url.clone(),
        id: s.bing_id.clone(),
        music_name: s.music_name.clone(),
        music_url: s.music_url.clone(),
    }
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
