import { invoke } from "@tauri-apps/api/core";

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
