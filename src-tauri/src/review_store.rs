use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;

// 艾宾浩斯遗忘曲线复习间隔
const INTERVALS_MS: [i64; 9] = [
    300_000,         // 5 min
    1_800_000,       // 30 min
    43_200_000,      // 12 hours
    86_400_000,      // 1 day
    172_800_000,     // 2 days
    345_600_000,     // 4 days
    604_800_000,     // 7 days
    1_296_000_000,   // 15 days
    2_592_000_000,   // 30 days → mastered
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ReviewResult {
    Remembered,
    Fuzzy,
    Forgot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRecord {
    pub word: String,
    pub stage: u8,
    pub next_review_at: i64,
    pub last_reviewed_at: i64,
    pub added_at: i64,
    pub review_count: u32,
    pub correct_count: u32,
    pub is_mastered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub reviews_done: u32,
    pub correct: u32,
    pub words_added: u32,
    pub words_mastered: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    pub daily_new_limit: u32,
    pub active_book: usize,
    pub book_progress: HashMap<usize, usize>,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            daily_new_limit: 20,
            active_book: 0,
            book_progress: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearningData {
    pub records: HashMap<String, LearningRecord>,
    pub daily_stats: Vec<DailyStats>,
    pub streak_last_date: Option<String>,
    pub streak_count: u32,
    #[serde(default)]
    pub config: LearningConfig,
}

pub type ReviewStore = Arc<Mutex<ReviewStoreInner>>;

pub struct ReviewStoreInner {
    data: LearningData,
    path: PathBuf,
}

impl ReviewStoreInner {
    pub fn load(app_data_dir: PathBuf) -> Self {
        let path = app_data_dir.join("learning.json");
        let data = if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => LearningData::default(),
            }
        } else {
            LearningData::default()
        };
        Self { data, path }
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            let _ = std::fs::write(&self.path, json);
        }
    }

    pub fn data_mut(&mut self) -> &mut LearningData {
        &mut self.data
    }

    fn today_str() -> String {
        Local::now().format("%Y-%m-%d").to_string()
    }

    fn today_stats_mut(&mut self) -> &mut DailyStats {
        let today = Self::today_str();
        if self.data.daily_stats.last().map(|s| &s.date) != Some(&today) {
            self.data.daily_stats.push(DailyStats {
                date: today,
                reviews_done: 0,
                correct: 0,
                words_added: 0,
                words_mastered: 0,
            });
        }
        // keep only 30 days
        if self.data.daily_stats.len() > 30 {
            self.data.daily_stats = self.data.daily_stats.split_off(self.data.daily_stats.len() - 30);
        }
        self.data.daily_stats.last_mut().unwrap()
    }

    fn update_streak(&mut self) {
        let today = Self::today_str();
        let today_date = NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap();

        match &self.data.streak_last_date {
            Some(last) => {
                if let Ok(last_date) = NaiveDate::parse_from_str(last, "%Y-%m-%d") {
                    let diff = (today_date - last_date).num_days();
                    if diff == 1 {
                        self.data.streak_count += 1;
                    } else if diff > 1 {
                        self.data.streak_count = 1;
                    }
                }
            }
            None => {
                self.data.streak_count = 1;
            }
        }
        self.data.streak_last_date = Some(today);
    }

    pub fn add_word(&mut self, word: String) -> bool {
        if self.data.records.contains_key(&word) {
            return false;
        }
        let now = chrono::Utc::now().timestamp_millis();
        self.data.records.insert(word.clone(), LearningRecord {
            word,
            stage: 0,
            next_review_at: now + INTERVALS_MS[0],
            last_reviewed_at: now,
            added_at: now,
            review_count: 0,
            correct_count: 0,
            is_mastered: false,
        });
        self.today_stats_mut().words_added += 1;
        self.save();
        true
    }

    pub fn add_words_batch(&mut self, words: Vec<String>) -> u32 {
        let mut count = 0u32;
        let now = chrono::Utc::now().timestamp_millis();
        for word in words {
            if self.data.records.contains_key(&word) {
                continue;
            }
            self.data.records.insert(word.clone(), LearningRecord {
                word,
                stage: 0,
                next_review_at: now + INTERVALS_MS[0],
                last_reviewed_at: now,
                added_at: now,
                review_count: 0,
                correct_count: 0,
                is_mastered: false,
            });
            count += 1;
        }
        if count > 0 {
            self.today_stats_mut().words_added += count;
            self.save();
        }
        count
    }

    pub fn remove_word(&mut self, word: &str) -> bool {
        let removed = self.data.records.remove(word).is_some();
        if removed { self.save(); }
        removed
    }

    pub fn get_due_words(&self) -> Vec<LearningRecord> {
        let now = chrono::Utc::now().timestamp_millis();
        self.data.records.values()
            .filter(|r| !r.is_mastered && r.next_review_at <= now)
            .cloned()
            .collect()
    }

    pub fn get_due_count(&self) -> usize {
        let now = chrono::Utc::now().timestamp_millis();
        self.data.records.values()
            .filter(|r| !r.is_mastered && r.next_review_at <= now)
            .count()
    }

    pub fn submit_review(&mut self, word: &str, result: ReviewResult) {
        let now = chrono::Utc::now().timestamp_millis();
        if let Some(record) = self.data.records.get_mut(word) {
            record.review_count += 1;
            record.last_reviewed_at = now;

            match result {
                ReviewResult::Remembered => {
                    record.correct_count += 1;
                    if record.stage >= 8 {
                        record.is_mastered = true;
                        record.next_review_at = i64::MAX;
                        self.today_stats_mut().words_mastered += 1;
                    } else {
                        record.stage += 1;
                        record.next_review_at = now + INTERVALS_MS[record.stage as usize];
                    }
                }
                ReviewResult::Fuzzy => {
                    record.stage = record.stage.saturating_sub(1);
                    record.next_review_at = now + INTERVALS_MS[record.stage as usize];
                }
                ReviewResult::Forgot => {
                    record.stage = record.stage.saturating_sub(3);
                    record.next_review_at = now + INTERVALS_MS[record.stage as usize];
                }
            }

            let stats = self.today_stats_mut();
            stats.reviews_done += 1;
            if result == ReviewResult::Remembered { stats.correct += 1; }

            self.update_streak();
            self.save();
        }
    }

    pub fn get_today_new_count(&self) -> u32 {
        let today = Self::today_str();
        self.data.daily_stats.iter()
            .find(|s| s.date == today)
            .map(|s| s.words_added)
            .unwrap_or(0)
    }

    pub fn get_daily_limit(&self) -> u32 {
        self.data.config.daily_new_limit
    }

    pub fn set_daily_limit(&mut self, limit: u32) {
        self.data.config.daily_new_limit = limit;
        self.save();
    }

    pub fn get_active_book(&self) -> usize {
        self.data.config.active_book
    }

    pub fn set_active_book(&mut self, index: usize) {
        self.data.config.active_book = index;
        self.save();
    }

    pub fn get_book_progress(&self, book_index: usize) -> usize {
        self.data.config.book_progress.get(&book_index).copied().unwrap_or(0)
    }

    pub fn advance_book_progress(&mut self, book_index: usize, count: usize) {
        let current = self.get_book_progress(book_index);
        self.data.config.book_progress.insert(book_index, current + count);
        self.save();
    }

    pub fn get_learning_config(&self) -> LearningConfig {
        self.data.config.clone()
    }

    pub fn get_stats(&self) -> LearningStatsResponse {
        let total = self.data.records.len() as u32;
        let mastered = self.data.records.values().filter(|r| r.is_mastered).count() as u32;
        let in_progress = total - mastered;
        let due_today = self.get_due_count() as u32;

        let today = Self::today_str();
        let today_stats = self.data.daily_stats.iter().find(|s| s.date == today);
        let today_reviews = today_stats.map(|s| s.reviews_done).unwrap_or(0);
        let today_correct = today_stats.map(|s| s.correct).unwrap_or(0);
        let accuracy = if today_reviews > 0 {
            (today_correct as f64 / today_reviews as f64 * 100.0) as u32
        } else { 0 };

        LearningStatsResponse {
            total,
            mastered,
            in_progress,
            due_today,
            streak: self.data.streak_count,
            today_reviews,
            today_accuracy: accuracy,
            daily_stats: self.data.daily_stats.clone(),
        }
    }

    pub fn get_learning_words(&self, filter: &str) -> Vec<LearningRecord> {
        let now = chrono::Utc::now().timestamp_millis();
        self.data.records.values()
            .filter(|r| match filter {
                "mastered" => r.is_mastered,
                "in_progress" => !r.is_mastered,
                "due" => !r.is_mastered && r.next_review_at <= now,
                _ => true,
            })
            .cloned()
            .collect()
    }

    pub fn is_word_in_learning(&self, word: &str) -> bool {
        self.data.records.contains_key(word)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStatsResponse {
    pub total: u32,
    pub mastered: u32,
    pub in_progress: u32,
    pub due_today: u32,
    pub streak: u32,
    pub today_reviews: u32,
    pub today_accuracy: u32,
    pub daily_stats: Vec<DailyStats>,
}

pub fn create_review_store(app_data_dir: PathBuf) -> ReviewStore {
    Arc::new(Mutex::new(ReviewStoreInner::load(app_data_dir)))
}
