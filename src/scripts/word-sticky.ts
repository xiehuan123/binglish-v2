import { getCurrentWindow } from "@tauri-apps/api/window";
import { load } from "@tauri-apps/plugin-store";
import { getWordPage } from "./shared/invoke";

const PAGE_SIZE = 5;
let currentPage = 0;
let totalPages = 1;

const appWindow = getCurrentWindow();

let store: Awaited<ReturnType<typeof load>> | null = null;

async function getStore() {
  if (!store) store = await load("config.json");
  return store;
}

async function loadSavedPage(): Promise<number> {
  try {
    const s = await getStore();
    const page = await s.get<number>("sticky_page");
    return typeof page === "number" ? page : 0;
  } catch { return 0; }
}

async function savePage(page: number) {
  try {
    const s = await getStore();
    await s.set("sticky_page", page);
    await s.save();
  } catch { /* ignore */ }
}

async function loadPage(page: number) {
  const data = await getWordPage(page, PAGE_SIZE);
  currentPage = data.current_page;
  totalPages = data.total_pages;
  savePage(currentPage);

  document.getElementById("pageInfo")!.textContent = `${currentPage + 1} / ${totalPages}`;

  const prevBtn = document.getElementById("prevBtn") as HTMLButtonElement;
  const nextBtn = document.getElementById("nextBtn") as HTMLButtonElement;
  prevBtn.disabled = currentPage <= 0;
  nextBtn.disabled = currentPage >= totalPages - 1;

  const list = document.getElementById("wordList")!;
  list.innerHTML = data.words.map(w => {
    const ph = w.phonetic ? `/${w.phonetic}/` : "";
    return `<div class="word-item">
      <div class="word-top">
        <span class="word-name">${w.word}</span>
        <span class="word-phonetic">${ph}</span>
      </div>
      <div class="word-trans">${w.trans}</div>
    </div>`;
  }).join("");
}

document.getElementById("prevBtn")!.addEventListener("click", (e) => {
  e.stopPropagation();
  if (currentPage > 0) loadPage(currentPage - 1);
});

document.getElementById("nextBtn")!.addEventListener("click", (e) => {
  e.stopPropagation();
  if (currentPage < totalPages - 1) loadPage(currentPage + 1);
});

document.getElementById("closeBtn")!.addEventListener("click", (e) => {
  e.stopPropagation();
  appWindow.close();
});

loadSavedPage().then(p => loadPage(p));
