use std::io::BufReader;

pub fn play_word_audio(url: &str) {
    let url = url.to_string();
    std::thread::spawn(move || {
        log::info!("Playing word audio from: {url}");
        let resp = match reqwest::blocking::get(&url) {
            Ok(r) => r,
            Err(e) => { log::warn!("Word audio download failed: {e}"); return; }
        };
        let bytes = match resp.bytes() {
            Ok(b) => b,
            Err(e) => { log::warn!("Word audio read failed: {e}"); return; }
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
            Err(e) => { log::warn!("Word audio decode failed: {e}"); return; }
        }

        sink.sleep_until_end();
        log::info!("Word audio finished");
    });
}
