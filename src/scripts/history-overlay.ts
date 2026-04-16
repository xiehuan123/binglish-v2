import { getCurrentWindow } from "@tauri-apps/api/window";
import { getHistoryToday } from "./shared/invoke";
import type { HistoryEvent } from "./shared/types";

const appWindow = getCurrentWindow();

document.getElementById("exitBtn")!.addEventListener("click", close);
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") close();
});

function close() {
  appWindow.close();
}

async function init() {
  const now = new Date();
  document.getElementById("dateText")!.textContent =
    `${now.getFullYear()}/${String(now.getMonth() + 1).padStart(2, "0")}/${String(now.getDate()).padStart(2, "0")}`;

  try {
    const events = (await getHistoryToday()) as HistoryEvent[];
    const list = document.getElementById("historyList")!;
    if (!events.length) {
      list.innerHTML = `<div class="loading">No events found for today.</div>`;
      return;
    }
    list.innerHTML = events
      .map(
        (ev) => `
      <div class="history-item">
        <div class="year">${ev.year}</div>
        <div class="en">${ev.en}</div>
        <div class="cn">${ev.cn}</div>
      </div>`
      )
      .join("");
  } catch (e) {
    document.getElementById("historyList")!.innerHTML =
      `<div class="loading">Failed to load history data.</div>`;
    console.error(e);
  }
}

init();
