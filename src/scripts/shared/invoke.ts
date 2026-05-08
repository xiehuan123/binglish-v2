import { invoke } from "@tauri-apps/api/core";

export interface WordEntry {
  word: string;
  phonetic: string;
  trans: string;
  sentence_en: string;
  sentence_cn: string;
}

export interface LearningRecord {
  word: string;
  stage: number;
  next_review_at: number;
  last_reviewed_at: number;
  added_at: number;
  review_count: number;
  correct_count: number;
  is_mastered: boolean;
}

export interface LearningStats {
  total: number;
  mastered: number;
  in_progress: number;
  due_today: number;
  streak: number;
  today_reviews: number;
  today_accuracy: number;
  daily_stats: { date: string; reviews_done: number; correct: number; words_added: number; words_mastered: number }[];
}

export interface LearningConfig {
  daily_new_limit: number;
  active_book: number;
  book_progress: Record<number, number>;
}

export interface BookInfo {
  index: number;
  label: string;
  total: number;
}

export type ReviewResult = "remembered" | "fuzzy" | "forgot";

export async function updateWallpaper() {
  return invoke<string>("update_wallpaper");
}

export async function getCurrentWord() {
  return invoke<string | null>("get_current_word");
}

export async function getGameData() {
  return invoke("get_game_data");
}

export async function restCompleted() {
  return invoke("rest_completed");
}

export async function isFullscreen() {
  return invoke<boolean>("is_fullscreen");
}

export async function getWordPage(page: number, pageSize: number) {
  return invoke<{ words: { word: string; phonetic: string; trans: string }[]; current_page: number; total_pages: number }>("get_word_page", { page, pageSize });
}

// Review commands
export async function addWordToLearning(word: string) {
  return invoke<boolean>("add_word_to_learning", { word });
}

export async function addWordsBatch(words: string[]) {
  return invoke<number>("add_words_batch", { words });
}

export async function getDueWords() {
  return invoke<LearningRecord[]>("get_due_words");
}

export async function getDueCount() {
  return invoke<number>("get_due_count");
}

export async function submitReview(word: string, result: ReviewResult) {
  return invoke("submit_review", { word, result });
}

export async function getLearningStats() {
  return invoke<LearningStats>("get_learning_stats");
}

export async function getLearningWords(filter: string) {
  return invoke<LearningRecord[]>("get_learning_words", { filter });
}

export async function removeWordFromLearning(word: string) {
  return invoke<boolean>("remove_word_from_learning", { word });
}

export async function isWordInLearning(word: string) {
  return invoke<boolean>("is_word_in_learning", { word });
}

// New learning flow commands
export async function getNewWords(count: number) {
  return invoke<WordEntry[]>("get_new_words", { count });
}

export async function commitNewWords(words: string[]) {
  return invoke<number>("commit_new_words", { words });
}

export async function getTodayNewCount() {
  return invoke<number>("get_today_new_count");
}

export async function setDailyLimit(limit: number) {
  return invoke("set_daily_limit", { limit });
}

export async function setLearningBook(index: number) {
  return invoke("set_learning_book", { index });
}

export async function getLearningConfig() {
  return invoke<LearningConfig>("get_learning_config");
}

export async function getBookInfo() {
  return invoke<BookInfo[]>("get_book_info");
}
