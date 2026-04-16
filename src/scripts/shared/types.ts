export interface WallpaperInfo {
  word: string | null;
  url: string | null;
  mp3: string | null;
  copyright: string | null;
  copyright_url: string | null;
  id: string | null;
  music_name: string | null;
  music_url: string | null;
}

export interface HistoryEvent {
  year: string;
  en: string;
  cn: string;
}

export interface GameData {
  shuffle: { en: string; cn: string } | null;
  wordle: { word: string; hint: string | null } | null;
}

export interface UselessFact {
  en: string;
  cn: string;
}
