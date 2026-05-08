import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";

interface DailyStats {
  date: string;
  reviews_done: number;
  correct: number;
  words_added: number;
  words_mastered: number;
}

interface LearningStatsResponse {
  total: number;
  mastered: number;
  in_progress: number;
  due_today: number;
  streak: number;
  today_reviews: number;
  today_accuracy: number;
  daily_stats: DailyStats[];
}

const appWindow = getCurrentWindow();

async function init() {
  const stats = await invoke<LearningStatsResponse>("get_learning_stats");

  document.getElementById("totalWords")!.textContent = String(stats.total);
  document.getElementById("inProgress")!.textContent = String(stats.in_progress);
  document.getElementById("masteredCount")!.textContent = String(stats.mastered);
  document.getElementById("dueToday")!.textContent = String(stats.due_today);
  document.getElementById("streak")!.textContent = String(stats.streak);
  document.getElementById("todayReviews")!.textContent = String(stats.today_reviews);
  document.getElementById("todayAccuracy")!.textContent = `${stats.today_accuracy}%`;

  updateRing(stats.mastered, stats.total);
  renderChart(stats.daily_stats);
}

function updateRing(mastered: number, total: number) {
  const ring = document.getElementById("ringProgress") as unknown as SVGCircleElement;
  const circumference = 2 * Math.PI * 42;
  ring.style.strokeDasharray = `${circumference}`;
  const ratio = total > 0 ? mastered / total : 0;
  const offset = circumference * (1 - ratio);
  ring.style.strokeDashoffset = `${offset}`;
}

function renderChart(dailyStats: DailyStats[]) {
  const chart = document.getElementById("chart")!;
  if (dailyStats.length === 0) {
    chart.innerHTML = '<div class="chart-empty">暂无数据</div>';
    return;
  }

  const maxReviews = Math.max(...dailyStats.map(s => s.reviews_done), 1);

  chart.innerHTML = dailyStats.map(s => {
    const height = Math.max((s.reviews_done / maxReviews) * 100, 4);
    const day = s.date.slice(8);
    const accuracy = s.reviews_done > 0 ? Math.round(s.correct / s.reviews_done * 100) : 0;
    return `<div class="chart-bar-wrap" title="${s.date}\n复习 ${s.reviews_done} 次\n正确率 ${accuracy}%">
      <div class="chart-bar" style="height: ${height}%"></div>
      <div class="chart-label">${day}</div>
    </div>`;
  }).join("");
}

document.getElementById("closeBtn")!.addEventListener("click", () => appWindow.close());

init();
