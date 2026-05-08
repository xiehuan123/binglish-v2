use crate::review_store::{ReviewStore, ReviewResult, LearningRecord, LearningStatsResponse, LearningConfig};
use crate::word_db::{WordDb, WordEntry};
use tauri::State;

#[tauri::command]
pub fn add_word_to_learning(store: State<'_, ReviewStore>, word: String) -> bool {
    store.lock().add_word(word)
}

#[tauri::command]
pub fn add_words_batch(store: State<'_, ReviewStore>, words: Vec<String>) -> u32 {
    store.lock().add_words_batch(words)
}

#[tauri::command]
pub fn get_due_words(store: State<'_, ReviewStore>) -> Vec<LearningRecord> {
    store.lock().get_due_words()
}

#[tauri::command]
pub fn get_due_count(store: State<'_, ReviewStore>) -> usize {
    store.lock().get_due_count()
}

#[tauri::command]
pub fn submit_review(store: State<'_, ReviewStore>, word: String, result: String) {
    let review_result = match result.as_str() {
        "remembered" => ReviewResult::Remembered,
        "fuzzy" => ReviewResult::Fuzzy,
        _ => ReviewResult::Forgot,
    };
    store.lock().submit_review(&word, review_result);
}

#[tauri::command]
pub fn get_learning_stats(store: State<'_, ReviewStore>) -> LearningStatsResponse {
    store.lock().get_stats()
}

#[tauri::command]
pub fn get_learning_words(store: State<'_, ReviewStore>, filter: String) -> Vec<LearningRecord> {
    store.lock().get_learning_words(&filter)
}

#[tauri::command]
pub fn remove_word_from_learning(store: State<'_, ReviewStore>, word: String) -> bool {
    store.lock().remove_word(&word)
}

#[tauri::command]
pub fn is_word_in_learning(store: State<'_, ReviewStore>, word: String) -> bool {
    store.lock().is_word_in_learning(&word)
}

#[tauri::command]
pub fn get_new_words(
    store: State<'_, ReviewStore>,
    word_db: State<'_, WordDb>,
    count: usize,
) -> Vec<WordEntry> {
    let store = store.lock();
    let book_index = store.get_active_book();
    let progress = store.get_book_progress(book_index);
    let db = word_db.lock();
    let candidates = db.get_sequential_words(book_index, progress, count * 3);
    candidates.into_iter()
        .filter(|w| !store.is_word_in_learning(&w.word))
        .take(count)
        .collect()
}

#[tauri::command]
pub fn commit_new_words(
    store: State<'_, ReviewStore>,
    word_db: State<'_, WordDb>,
    words: Vec<String>,
) -> u32 {
    let mut store = store.lock();
    let book_index = store.get_active_book();
    let added = store.add_words_batch(words.clone());
    if added > 0 {
        let db = word_db.lock();
        let book_total = db.book_total_words(book_index);
        let current_progress = store.get_book_progress(book_index);
        let new_progress = (current_progress + added as usize).min(book_total);
        store.data_mut().config.book_progress.insert(book_index, new_progress);
        store.save();
    }
    added
}

#[tauri::command]
pub fn get_today_new_count(store: State<'_, ReviewStore>) -> u32 {
    store.lock().get_today_new_count()
}

#[tauri::command]
pub fn set_daily_limit(store: State<'_, ReviewStore>, limit: u32) {
    store.lock().set_daily_limit(limit);
}

#[tauri::command]
pub fn set_learning_book(store: State<'_, ReviewStore>, word_db: State<'_, WordDb>, index: usize) {
    store.lock().set_active_book(index);
    word_db.lock().set_active_book(index);
}

#[tauri::command]
pub fn get_learning_config(store: State<'_, ReviewStore>) -> LearningConfig {
    store.lock().get_learning_config()
}

#[tauri::command]
pub fn get_book_info(word_db: State<'_, WordDb>) -> Vec<BookInfo> {
    let db = word_db.lock();
    (0..5).map(|i| {
        let book = crate::word_db::WordBook::from_index(i);
        BookInfo {
            index: i,
            label: book.label().to_string(),
            total: db.book_total_words(i),
        }
    }).collect()
}

#[derive(serde::Serialize)]
pub struct BookInfo {
    pub index: usize,
    pub label: String,
    pub total: usize,
}
