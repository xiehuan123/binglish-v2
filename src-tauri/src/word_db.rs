use rand::seq::SliceRandom;
use serde::Deserialize;

const WORDS_JSON: &[u8] = include_bytes!("../resources/words.json");

#[derive(Debug, Clone, Deserialize)]
pub struct WordEntry {
    #[serde(rename = "w")]
    pub word: String,
    #[serde(rename = "ph")]
    pub phonetic: String,
    #[serde(rename = "tr")]
    pub trans: String,
    #[serde(rename = "en")]
    pub sentence_en: String,
    #[serde(rename = "cn")]
    pub sentence_cn: String,
}

pub struct WordDb {
    words: Vec<WordEntry>,
}

impl WordDb {
    pub fn load() -> Self {
        let words: Vec<WordEntry> = serde_json::from_slice(WORDS_JSON)
            .expect("Failed to parse words.json");
        log::info!("WordDb loaded: {} words", words.len());
        Self { words }
    }

    pub fn random_word(&self) -> Option<&WordEntry> {
        let mut rng = rand::thread_rng();
        self.words.choose(&mut rng)
    }
}
