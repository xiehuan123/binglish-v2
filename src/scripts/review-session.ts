import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import {
  getDueWords,
  getDueCount,
  getNewWords,
  commitNewWords,
  submitReview,
  getLearningStats,
  getLearningConfig,
  getBookInfo,
  setDailyLimit,
  setLearningBook,
  getTodayNewCount,
  type WordEntry,
  type LearningRecord,
  type ReviewResult,
  type BookInfo,
  type LearningConfig,
} from "./shared/invoke";

const appWindow = getCurrentWindow();

let config: LearningConfig;
let books: BookInfo[] = [];

interface CardItem {
  word: string;
  phonetic: string;
  trans: string;
  sentence_en: string;
  isNew: boolean;
}

let cards: CardItem[] = [];
let currentIndex = 0;
let correctCount = 0;
let totalReviewed = 0;
let newWordsLearned = 0;

async function init() {
  config = await getLearningConfig();
  books = await getBookInfo();
  await refreshHome();
  bindEvents();
}

async function refreshHome() {
  const stats = await getLearningStats();
  const todayNew = await getTodayNewCount();
  const dueCount = await getDueCount();

  document.getElementById("homeNewCount")!.textContent = `${todayNew}/${config.daily_new_limit}`;
  document.getElementById("homeDueCount")!.textContent = String(dueCount);
  document.getElementById("homeMastered")!.textContent = String(stats.mastered);

  const btn = document.getElementById("btnStart") as HTMLButtonElement;
  const remaining = config.daily_new_limit - todayNew;
  const total = remaining + dueCount;
  btn.disabled = total === 0;
  btn.textContent = total > 0 ? `开始学习 (${total})` : "今日已完成";

  populateBookSelect();
  populateLimitSelect();
}

function populateBookSelect() {
  const select = document.getElementById("selectBook") as HTMLSelectElement;
  select.innerHTML = "";
  books.forEach((book) => {
    const opt = document.createElement("option");
    opt.value = String(book.index);
    opt.textContent = `${book.label} (${book.total}词)`;
    if (book.index === config.active_book) opt.selected = true;
    select.appendChild(opt);
  });
}

function populateLimitSelect() {
  const select = document.getElementById("selectLimit") as HTMLSelectElement;
  for (const opt of select.options) {
    if (Number(opt.value) === config.daily_new_limit) {
      opt.selected = true;
      break;
    }
  }
}

function showView(id: string) {
  document.querySelectorAll(".view").forEach((el) => el.classList.add("hidden"));
  document.getElementById(id)!.classList.remove("hidden");
}

function playWord(word: string) {
  const audio = new Audio(`https://dict.youdao.com/dictvoice?type=2&audio=${encodeURIComponent(word)}`);
  audio.play().catch(() => {});
}

// --- Start Learning Session ---
async function startSession() {
  cards = [];
  currentIndex = 0;
  correctCount = 0;
  totalReviewed = 0;
  newWordsLearned = 0;

  // Fetch new words
  const todayNew = await getTodayNewCount();
  const remaining = config.daily_new_limit - todayNew;
  if (remaining > 0) {
    const newWords = await getNewWords(remaining);
    for (const w of newWords) {
      cards.push({
        word: w.word,
        phonetic: w.phonetic,
        trans: w.trans,
        sentence_en: w.sentence_en,
        isNew: true,
      });
    }
  }

  // Fetch due review words
  const dueWords = await getDueWords();
  for (const record of dueWords) {
    cards.push({
      word: record.word,
      phonetic: "",
      trans: "",
      sentence_en: "",
      isNew: false,
    });
  }

  if (cards.length === 0) {
    return;
  }

  // Pre-load details for review words
  loadReviewDetails(dueWords);

  document.getElementById("headerTitle")!.textContent = "学习中";
  showView("viewCard");
  showCard();
}

async function loadReviewDetails(records: LearningRecord[]) {
  if (records.length === 0) return;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const data = await invoke<{ words: { word: string; phonetic: string; trans: string; sentence_en?: string }[] }>(
      "get_word_page", { page: 0, pageSize: 9999 }
    );
    for (const card of cards) {
      if (card.isNew) continue;
      const found = data.words.find((w) => w.word === card.word);
      if (found) {
        card.phonetic = found.phonetic;
        card.trans = found.trans;
        card.sentence_en = found.sentence_en || "";
      }
    }
  } catch { /* ignore */ }
}

// --- Card Display ---
function showCard() {
  const card = cards[currentIndex];
  document.getElementById("cardProgress")!.textContent = `${currentIndex + 1}/${cards.length}`;
  document.getElementById("cardWord")!.textContent = card.word;
  document.getElementById("cardPhonetic")!.textContent = "";
  document.getElementById("cardTrans")!.textContent = "";
  document.getElementById("cardSentence")!.textContent = "";
  document.getElementById("answerReveal")!.classList.add("hidden");
  document.getElementById("btnShowAnswer")!.classList.remove("hidden");
  document.getElementById("resultBtns")!.classList.add("hidden");
}

function showAnswer() {
  const card = cards[currentIndex];
  document.getElementById("cardPhonetic")!.textContent = card.phonetic ? `/${card.phonetic}/` : "";
  document.getElementById("cardTrans")!.textContent = card.trans;
  document.getElementById("cardSentence")!.textContent = card.sentence_en;
  document.getElementById("answerReveal")!.classList.remove("hidden");
  document.getElementById("btnShowAnswer")!.classList.add("hidden");
  document.getElementById("resultBtns")!.classList.remove("hidden");
  playWord(card.word);
}

async function handleResult(result: ReviewResult) {
  const card = cards[currentIndex];

  if (card.isNew) {
    // Commit new word to learning system, then submit initial review
    await commitNewWords([card.word]);
    newWordsLearned++;
  }

  await submitReview(card.word, result);
  totalReviewed++;
  if (result === "remembered") correctCount++;

  currentIndex++;
  if (currentIndex >= cards.length) {
    const accuracy = totalReviewed > 0 ? Math.round((correctCount / totalReviewed) * 100) : 0;
    let statsText = `复习 ${totalReviewed} 词，正确率 ${accuracy}%`;
    if (newWordsLearned > 0) {
      statsText = `新学 ${newWordsLearned} 词，${statsText}`;
    }
    document.getElementById("headerTitle")!.textContent = "完成";
    document.getElementById("doneText")!.textContent = "学习完成!";
    document.getElementById("doneStats")!.textContent = statsText;
    showView("viewDone");
  } else {
    showCard();
  }
}

async function backToHome() {
  document.getElementById("headerTitle")!.textContent = "今日学习";
  showView("viewHome");
  await refreshHome();
}

// --- Events ---
function bindEvents() {
  document.getElementById("closeBtn")!.addEventListener("click", () => appWindow.close());
  document.getElementById("btnStart")!.addEventListener("click", startSession);
  document.getElementById("btnShowAnswer")!.addEventListener("click", showAnswer);
  document.getElementById("btnForgot")!.addEventListener("click", () => handleResult("forgot"));
  document.getElementById("btnFuzzy")!.addEventListener("click", () => handleResult("fuzzy"));
  document.getElementById("btnRemembered")!.addEventListener("click", () => handleResult("remembered"));
  document.getElementById("btnBackHome")!.addEventListener("click", backToHome);

  document.getElementById("selectBook")!.addEventListener("change", async (e) => {
    const index = Number((e.target as HTMLSelectElement).value);
    await setLearningBook(index);
    config.active_book = index;
    await emit("book-changed", { index });
    await refreshHome();
  });

  document.getElementById("selectLimit")!.addEventListener("change", async (e) => {
    const limit = Number((e.target as HTMLSelectElement).value);
    await setDailyLimit(limit);
    config.daily_new_limit = limit;
    await refreshHome();
  });

  // Click word to play audio
  document.getElementById("cardWord")!.addEventListener("click", () => {
    if (cards[currentIndex]) playWord(cards[currentIndex].word);
  });
}

init();
