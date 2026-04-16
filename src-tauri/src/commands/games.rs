use serde::Serialize;

const GAME_DATA_URL: &str = "https://ss.blueforge.org/bing/games.json";

#[derive(Debug, Serialize, Clone)]
pub struct GameData {
    pub shuffle: Option<ShuffleData>,
    pub wordle: Option<WordleData>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ShuffleData {
    pub en: String,
    pub cn: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct WordleData {
    pub word: String,
    pub hint: Option<String>,
}

#[tauri::command]
pub async fn get_game_data() -> Result<GameData, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(GAME_DATA_URL)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse failed: {e}"))?;

    let shuffle = json.get("shuffle").map(|s| ShuffleData {
        en: s.get("en").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        cn: s.get("cn").and_then(|v| v.as_str()).unwrap_or("").to_string(),
    });

    let wordle = json.get("wordle").map(|w| WordleData {
        word: w.get("word").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        hint: w.get("hint").and_then(|v| v.as_str()).map(String::from),
    });

    Ok(GameData { shuffle, wordle })
}
