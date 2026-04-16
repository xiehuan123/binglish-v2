use serde::Serialize;

const USELESS_FACT_URL: &str = "https://ss.blueforge.org/bing/uselessfact.json";

#[derive(Debug, Serialize)]
pub struct UselessFact {
    pub en: String,
    pub cn: String,
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn is_fullscreen() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowRect, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
    };
    use windows::Win32::Foundation::RECT;

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return false;
        }
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let sw = GetSystemMetrics(SM_CXSCREEN);
            let sh = GetSystemMetrics(SM_CYSCREEN);
            return rect.left <= 0
                && rect.top <= 0
                && rect.right >= sw
                && rect.bottom >= sh;
        }
        false
    }
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn is_fullscreen() -> bool {
    // macOS 不做全屏检测，始终允许休息提醒
    false
}

#[tauri::command]
pub async fn get_useless_fact() -> Result<UselessFact, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(USELESS_FACT_URL)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse failed: {e}"))?;

    Ok(UselessFact {
        en: json.get("en").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        cn: json.get("cn").and_then(|v| v.as_str()).unwrap_or("").to_string(),
    })
}
