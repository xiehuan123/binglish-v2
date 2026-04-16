import { invoke } from "@tauri-apps/api/core";

export async function updateWallpaper(isRandom: boolean) {
  return invoke("update_wallpaper", { isRandom });
}

export async function getWallpaperInfo() {
  return invoke("get_wallpaper_info");
}

export async function toggleMusic() {
  return invoke<boolean>("toggle_music");
}

export async function isMusicPlaying() {
  return invoke<boolean>("is_music_playing");
}

export async function getHistoryToday() {
  return invoke("get_history_today");
}

export async function getGameData() {
  return invoke("get_game_data");
}

export async function getUselessFact() {
  return invoke<{ en: string; cn: string }>("get_useless_fact");
}

export async function restCompleted() {
  return invoke("rest_completed");
}

export async function isFullscreen() {
  return invoke<boolean>("is_fullscreen");
}
