use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use parking_lot::Mutex;
use std::sync::Arc;

const WORDS_CET4_CET6: &[u8] = include_bytes!("../resources/words.json");
const WORDS_TOEFL: &[u8] = include_bytes!("../resources/words_toefl.json");
const WORDS_IELTS: &[u8] = include_bytes!("../resources/words_ielts.json");
const WORDS_GRE: &[u8] = include_bytes!("../resources/words_gre.json");
const WORDS_KAOYAN: &[u8] = include_bytes!("../resources/words_kaoyan.json");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WordEntry {
    #[serde(alias = "w")]
    pub word: String,
    #[serde(alias = "ph")]
    pub phonetic: String,
    #[serde(alias = "tr")]
    pub trans: String,
    #[serde(alias = "en")]
    pub sentence_en: String,
    #[serde(alias = "cn")]
    pub sentence_cn: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WordBook {
    Cet4Cet6,
    Toefl,
    Ielts,
    Gre,
    Kaoyan,
}

impl WordBook {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Cet4Cet6 => "四六级",
            Self::Toefl => "托福",
            Self::Ielts => "雅思",
            Self::Gre => "GRE",
            Self::Kaoyan => "考研",
        }
    }

    pub fn all() -> &'static [WordBook] {
        &[Self::Cet4Cet6, Self::Toefl, Self::Ielts, Self::Gre, Self::Kaoyan]
    }

    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Cet4Cet6,
            1 => Self::Toefl,
            2 => Self::Ielts,
            3 => Self::Gre,
            4 => Self::Kaoyan,
            _ => Self::Cet4Cet6,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Self::Cet4Cet6 => 0,
            Self::Toefl => 1,
            Self::Ielts => 2,
            Self::Gre => 3,
            Self::Kaoyan => 4,
        }
    }
}

pub struct WordDbInner {
    books: Vec<Vec<WordEntry>>,
    active_book: usize,
}

pub type WordDb = Arc<Mutex<WordDbInner>>;

fn parse_book(data: &[u8]) -> Vec<WordEntry> {
    serde_json::from_slice(data).unwrap_or_default()
}

pub fn load_word_db() -> WordDb {
    let books = vec![
        parse_book(WORDS_CET4_CET6),
        parse_book(WORDS_TOEFL),
        parse_book(WORDS_IELTS),
        parse_book(WORDS_GRE),
        parse_book(WORDS_KAOYAN),
    ];
    log::info!("WordDb loaded: {} books, sizes: {:?}",
        books.len(),
        books.iter().map(|b| b.len()).collect::<Vec<_>>()
    );
    Arc::new(Mutex::new(WordDbInner { books, active_book: 0 }))
}

impl WordDbInner {
    pub fn set_active_book(&mut self, index: usize) {
        if index < self.books.len() {
            self.active_book = index;
        }
    }

    pub fn active_book_index(&self) -> usize {
        self.active_book
    }

    fn active_words(&self) -> &[WordEntry] {
        &self.books[self.active_book]
    }

    pub fn random_word(&self) -> Option<&WordEntry> {
        let mut rng = rand::thread_rng();
        self.active_words().choose(&mut rng)
    }

    pub fn get_page(&self, page: usize, page_size: usize) -> Vec<WordEntry> {
        let words = self.active_words();
        let start = page * page_size;
        if start >= words.len() {
            return Vec::new();
        }
        let end = (start + page_size).min(words.len());
        words[start..end].to_vec()
    }

    pub fn total_words(&self) -> usize {
        self.active_words().len()
    }

    pub fn get_sequential_words(&self, book_index: usize, start: usize, count: usize) -> Vec<WordEntry> {
        if book_index >= self.books.len() {
            return Vec::new();
        }
        let words = &self.books[book_index];
        if start >= words.len() {
            return Vec::new();
        }
        let end = (start + count).min(words.len());
        words[start..end].to_vec()
    }

    pub fn book_total_words(&self, book_index: usize) -> usize {
        if book_index >= self.books.len() {
            return 0;
        }
        self.books[book_index].len()
    }

    pub fn find_word(&self, word: &str) -> Option<&WordEntry> {
        for book in &self.books {
            if let Some(entry) = book.iter().find(|e| e.word == word) {
                return Some(entry);
            }
        }
        None
    }
}
