use crate::state::AppState;
use std::io::BufReader;

/// 核心逻辑：可从 tray 和 tauri command 调用
pub fn toggle_music_inner(state: &AppState) -> Result<bool, String> {
    let mut s = state.lock();

    if s.is_music_playing {
        if let Some(tx) = s.music_stop_tx.take() {
            let _ = tx.send(());
        }
        s.is_music_playing = false;
        log::info!("Music stopped");
        return Ok(false);
    }

    let url = s.music_url.clone().ok_or("No music URL available")?;

    let (tx, rx) = std::sync::mpsc::channel::<()>();
    s.music_stop_tx = Some(tx);
    s.is_music_playing = true;
    drop(s);

    std::thread::spawn(move || {
        log::info!("Downloading music from: {url}");
        let resp = match reqwest::blocking::get(&url) {
            Ok(r) => r,
            Err(e) => { log::warn!("Music download failed: {e}"); return; }
        };
        let bytes = match resp.bytes() {
            Ok(b) => b,
            Err(e) => { log::warn!("Music read failed: {e}"); return; }
        };

        let (_stream, stream_handle) = match rodio::OutputStream::try_default() {
            Ok(s) => s,
            Err(e) => { log::warn!("Audio output failed: {e}"); return; }
        };
        let sink = match rodio::Sink::try_new(&stream_handle) {
            Ok(s) => s,
            Err(e) => { log::warn!("Sink failed: {e}"); return; }
        };

        let cursor = std::io::Cursor::new(bytes.to_vec());
        match rodio::Decoder::new(BufReader::new(cursor)) {
            Ok(source) => sink.append(source),
            Err(e) => { log::warn!("Decode failed: {e}"); return; }
        }

        log::info!("Music playing");
        loop {
            if rx.try_recv().is_ok() { sink.stop(); break; }
            if sink.empty() { break; }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        log::info!("Music thread exiting");
    });

    Ok(true)
}

#[tauri::command]
pub fn toggle_music(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    toggle_music_inner(state.inner())
}

#[tauri::command]
pub fn is_music_playing(state: tauri::State<'_, AppState>) -> bool {
    state.lock().is_music_playing
}
