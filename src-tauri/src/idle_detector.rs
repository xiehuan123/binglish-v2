/// 返回用户空闲时间（秒）
#[cfg(target_os = "windows")]
pub fn get_idle_seconds() -> Result<u64, String> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    use windows::Win32::System::SystemInformation::GetTickCount;

    let mut info = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    unsafe {
        if !GetLastInputInfo(&mut info).as_bool() {
            return Err("GetLastInputInfo failed".to_string());
        }
        let tick = GetTickCount();
        let idle_ms = tick.wrapping_sub(info.dwTime);
        Ok((idle_ms / 1000) as u64)
    }
}

#[cfg(target_os = "macos")]
pub fn get_idle_seconds() -> Result<u64, String> {
    let output = std::process::Command::new("ioreg")
        .args(["-c", "IOHIDSystem", "-d", "4"])
        .output()
        .map_err(|e| format!("ioreg failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("HIDIdleTime") {
            if let Some(val) = line.split('=').last() {
                let val = val.trim();
                if let Ok(ns) = val.parse::<u64>() {
                    return Ok(ns / 1_000_000_000);
                }
            }
        }
    }
    Err("Could not parse HIDIdleTime".to_string())
}
