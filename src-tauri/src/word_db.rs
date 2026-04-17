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

    pub fn get_page(&self, page: usize, page_size: usize) -> Vec<WordEntry> {
        let start = page * page_size;
        if start >= self.words.len() {
            return Vec::new();
        }
        let end = (start + page_size).min(self.words.len());
        self.words[start..end].to_vec()
    }

    pub fn total_words(&self) -> usize {
        self.words.len()
    }
}
