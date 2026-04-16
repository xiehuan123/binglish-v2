use serde::Serialize;

const HISTORY_URL_BASE: &str = "https://ss.blueforge.org/getHistory";

#[derive(Debug, Serialize, Clone)]
pub struct HistoryEvent {
    pub year: String,
    pub en: String,
    pub cn: String,
}

#[tauri::command]
pub async fn get_history_today() -> Result<Vec<HistoryEvent>, String> {
    let today = chrono::Local::now().format("%m/%d").to_string();
    let url = format!("{HISTORY_URL_BASE}?date={today}");

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let data: Vec<serde_json::Value> = resp.json().await.map_err(|e| format!("Parse failed: {e}"))?;

    let events = data
        .iter()
        .filter_map(|item| {
            Some(HistoryEvent {
                year: item.get("year")?.as_str()?.to_string(),
                en: item.get("en")?.as_str()?.to_string(),
                cn: item.get("cn")?.as_str()?.to_string(),
            })
        })
        .collect();

    Ok(events)
}
