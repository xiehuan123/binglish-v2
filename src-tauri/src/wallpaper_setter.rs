use std::path::Path;

#[cfg(target_os = "windows")]
pub fn set_wallpaper(path: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
    };

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wide.as_ptr() as *mut _),
            SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
        )
        .map_err(|e| format!("Failed to set wallpaper: {e}"))?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn set_wallpaper(path: &Path) -> Result<(), String> {
    // macOS 需要复制文件以强制刷新壁纸缓存
    let cache_path = path.with_extension("_display.jpg");
    std::fs::copy(path, &cache_path).map_err(|e| format!("Copy failed: {e}"))?;

    let script = format!(
        r#"tell application "System Events" to tell every desktop to set picture to POSIX file "{}""#,
        cache_path.display()
    );
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("osascript failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "osascript error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}
