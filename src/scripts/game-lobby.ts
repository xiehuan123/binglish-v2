import { getCurrentWindow } from "@tauri-apps/api/window";
import { getGameData } from "./shared/invoke";
import type { GameData } from "./shared/types";

const appWindow = getCurrentWindow();
let gameData: GameData | null = null;
let gameActive = false;
let startTime = 0;
let timerInterval: number | null = null;

document.getElementById("exitBtn")!.addEventListener("click", close);
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") close();
});

function close() {
  gameActive = false;
  if (timerInterval) clearInterval(timerInterval);
  appWindow.close();
}

function startTimer() {
  startTime = Date.now();
  document.getElementById("gameHud")!.style.display = "flex";
  timerInterval = window.setInterval(() => {
    const elapsed = Math.floor((Date.now() - startTime) / 1000);
    const m = String(Math.floor(elapsed / 60)).padStart(2, "0");
    const s = String(elapsed % 60).padStart(2, "0");
    document.getElementById("timerText")!.textContent = `Time: ${m}:${s}`;
  }, 1000);
}

async function init() {
  try {
    gameData = (await getGameData()) as GameData;
  } catch {
    close();
    return;
  }
  if (!gameData) { close(); return; }

  document.getElementById("btnShuffle")!.addEventListener("click", () => startGame("shuffle"));
  document.getElementById("btnWordle")!.addEventListener("click", () => startGame("wordle"));
}

function startGame(type: "shuffle" | "wordle") {
  document.getElementById("lobby")!.style.display = "none";
  document.getElementById("gameArea")!.style.display = "block";
  gameActive = true;
  startTimer();

  if (type === "shuffle") initSentenceMaster();
  else initWordle();
}

function initSentenceMaster() {
  if (!gameData?.shuffle) return;
  const raw = gameData.shuffle.en;
  const cn = gameData.shuffle.cn;
  const wordsOnly = raw.match(/[\w']+/g) || [];
  const tokens = raw.match(/[\w']+|[^\w\s]/g) || [];
  const indexed = wordsOnly.map((w, i) => ({ id: i, word: w }));
  const shuffled = [...indexed].sort(() => Math.random() - 0.5);

  const userOrder: (typeof indexed[0] | null)[] = new Array(wordsOnly.length).fill(null);
  let attempts = 1;

  const area = document.getElementById("gameArea")!;
  area.innerHTML = `
    <div class="sentence-master">
      <div class="title" style="font-size:32px">Sentence Master</div>
      <div class="subtitle">还原被打乱的句子，点击下方单词填入横线，点击横线上的单词重填。</div>
      <div class="slots-container" id="slotsContainer"></div>
      <div class="pool-container" id="poolContainer"></div>
      <div class="translation" id="translationText" style="display:none"></div>
    </div>
  `;
  document.getElementById("attemptsText")!.textContent = `尝试次数: ${attempts}`;

  const slotsEl = document.getElementById("slotsContainer")!;
  const poolEl = document.getElementById("poolContainer")!;
  const slotBtns: HTMLButtonElement[] = [];
  const poolBtns: Map<number, HTMLButtonElement> = new Map();

  let wIdx = 0;
  for (const t of tokens) {
    if (/[\w']+/.test(t)) {
      const curr = wIdx;
      const btn = document.createElement("button");
      btn.className = "slot-btn";
      btn.textContent = "______";
      btn.addEventListener("click", () => onSlotClick(curr));
      slotsEl.appendChild(btn);
      slotBtns.push(btn);
      wIdx++;
    } else {
      const span = document.createElement("span");
      span.className = "punctuation";
      span.textContent = t;
      slotsEl.appendChild(span);
    }
  }

  for (const obj of shuffled) {
    const btn = document.createElement("button");
    btn.className = "pool-btn";
    btn.textContent = obj.word;
    btn.addEventListener("click", () => onPoolClick(obj));
    poolEl.appendChild(btn);
    poolBtns.set(obj.id, btn);
  }

  function onPoolClick(obj: { id: number; word: string }) {
    if (!gameActive) return;
    for (let i = 0; i < userOrder.length; i++) {
      if (userOrder[i] === null) {
        userOrder[i] = obj;
        poolBtns.get(obj.id)!.style.display = "none";
        slotBtns[i].textContent = obj.word;
        slotBtns[i].classList.add("filled");

        if (obj.word.toLowerCase() === wordsOnly[i].toLowerCase()) {
          slotBtns[i].classList.add("correct");
          slotBtns[i].classList.add("locked");
        } else {
          slotBtns[i].classList.add("wrong");
          attempts++;
          document.getElementById("attemptsText")!.textContent = `尝试次数: ${attempts}`;
        }
        checkWin();
        break;
      }
    }
  }

  function onSlotClick(idx: number) {
    if (!gameActive || userOrder[idx] === null) return;
    if (userOrder[idx]!.word.toLowerCase() === wordsOnly[idx].toLowerCase()) return; // locked
    const obj = userOrder[idx]!;
    userOrder[idx] = null;
    slotBtns[idx].textContent = "______";
    slotBtns[idx].className = "slot-btn";
    poolBtns.get(obj.id)!.style.display = "";
  }

  function checkWin() {
    if (userOrder.some((u) => u === null)) return;
    const correct = userOrder.every(
      (u, i) => u!.word.toLowerCase() === wordsOnly[i].toLowerCase()
    );
    if (correct) {
      gameActive = false;
      if (timerInterval) clearInterval(timerInterval);
      slotBtns.forEach((b) => { b.classList.add("correct", "locked"); b.disabled = true; });
      poolBtns.forEach((b) => { b.disabled = true; });

      let rank = "Well Done!";
      if (attempts === 1) rank = "Godlike!";
      else if (attempts <= 2) rank = "Impressive!";
      else if (attempts <= 4) rank = "Excellent!";
      else if (attempts <= 6) rank = "Good Job!";

      const msg = document.getElementById("resultMsg")!;
      msg.textContent = `🎉 ${rank} 🎉`;
      msg.classList.add("success");

      const trans = document.getElementById("translationText")!;
      trans.textContent = cn;
      trans.style.display = "block";
    }
  }
}

function initWordle() {
  if (!gameData?.wordle) return;
  const answer = gameData.wordle.word.toLowerCase();
  const hint = gameData.wordle.hint;
  const ROWS = 6;
  const COLS = 5;

  let currentRow = 0;
  let currentCol = 0;
  let finished = false;
  const grid: string[][] = Array.from({ length: ROWS }, () => Array(COLS).fill(""));
  const keyStates: Map<string, string> = new Map();

  const area = document.getElementById("gameArea")!;
  area.innerHTML = `
    <div class="wordle-container">
      <div class="wordle-rules">
        <h2>Binglish Wordle</h2>
        <p>规则：<br>1. 目标：6次机会猜出5字母单词<br>2. 绿色：字母存在且位置正确<br>3. 黄色：字母存在但位置错<br>4. 灰色：字母不在答案中${hint ? `<br><br>提示：${hint}` : ""}</p>
      </div>
      <div class="wordle-board">
        <div class="wordle-grid" id="wordleGrid"></div>
        <div class="keyboard" id="keyboard"></div>
      </div>
    </div>
  `;

  // 构建网格
  const gridEl = document.getElementById("wordleGrid")!;
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const cell = document.createElement("div");
      cell.className = "wordle-cell";
      cell.id = `cell-${r}-${c}`;
      gridEl.appendChild(cell);
    }
  }

  // 构建键盘
  const kbRows = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];
  const kbEl = document.getElementById("keyboard")!;
  kbRows.forEach((row, ri) => {
    const rowEl = document.createElement("div");
    rowEl.className = "keyboard-row";
    if (ri === 2) {
      const enter = document.createElement("button");
      enter.className = "key wide";
      enter.textContent = "Enter";
      enter.addEventListener("click", () => submitGuess());
      rowEl.appendChild(enter);
    }
    for (const ch of row) {
      const key = document.createElement("button");
      key.className = "key";
      key.textContent = ch;
      key.id = `key-${ch}`;
      key.addEventListener("click", () => typeLetter(ch));
      rowEl.appendChild(key);
    }
    if (ri === 2) {
      const del = document.createElement("button");
      del.className = "key wide";
      del.textContent = "⌫";
      del.addEventListener("click", () => deleteLetter());
      rowEl.appendChild(del);
    }
    kbEl.appendChild(rowEl);
  });

  // 键盘事件
  document.addEventListener("keydown", (e) => {
    if (finished || !gameActive) return;
    if (e.key === "Enter") submitGuess();
    else if (e.key === "Backspace") deleteLetter();
    else if (/^[a-zA-Z]$/.test(e.key)) typeLetter(e.key.toLowerCase());
  });

  function typeLetter(ch: string) {
    if (finished || currentCol >= COLS) return;
    grid[currentRow][currentCol] = ch;
    const cell = document.getElementById(`cell-${currentRow}-${currentCol}`)!;
    cell.textContent = ch;
    cell.classList.add("active");
    currentCol++;
  }

  function deleteLetter() {
    if (finished || currentCol <= 0) return;
    currentCol--;
    grid[currentRow][currentCol] = "";
    const cell = document.getElementById(`cell-${currentRow}-${currentCol}`)!;
    cell.textContent = "";
    cell.classList.remove("active");
  }

  function submitGuess() {
    if (finished || currentCol < COLS) return;
    const guess = grid[currentRow].join("").toLowerCase();

    // 评估
    const result: string[] = Array(COLS).fill("absent");
    const answerChars = answer.split("");
    const used = Array(COLS).fill(false);

    // 先标记 correct
    for (let i = 0; i < COLS; i++) {
      if (guess[i] === answerChars[i]) {
        result[i] = "correct";
        used[i] = true;
      }
    }
    // 再标记 present
    for (let i = 0; i < COLS; i++) {
      if (result[i] === "correct") continue;
      for (let j = 0; j < COLS; j++) {
        if (!used[j] && guess[i] === answerChars[j]) {
          result[i] = "present";
          used[j] = true;
          break;
        }
      }
    }

    // 更新 UI
    for (let i = 0; i < COLS; i++) {
      const cell = document.getElementById(`cell-${currentRow}-${i}`)!;
      cell.classList.remove("active");
      cell.classList.add(result[i]);

      const ch = guess[i];
      const prev = keyStates.get(ch);
      if (result[i] === "correct" || (!prev && result[i] !== "correct")) {
        if (result[i] === "correct" || prev !== "correct") {
          keyStates.set(ch, result[i]);
        }
      }
      const keyEl = document.getElementById(`key-${ch}`);
      if (keyEl) {
        keyEl.className = `key ${keyStates.get(ch) || ""}`;
      }
    }

    if (guess === answer) {
      finished = true;
      gameActive = false;
      if (timerInterval) clearInterval(timerInterval);
      const ranks = ["Genius!", "Magnificent!", "Impressive!", "Splendid!", "Great!", "Phew!"];
      const msg = document.getElementById("resultMsg")!;
      msg.textContent = `🎉 ${ranks[Math.min(currentRow, 5)]} 🎉`;
      msg.classList.add("success");
      return;
    }

    currentRow++;
    currentCol = 0;

    if (currentRow >= ROWS) {
      finished = true;
      gameActive = false;
      if (timerInterval) clearInterval(timerInterval);
      const msg = document.getElementById("resultMsg")!;
      msg.textContent = `答案是: ${answer.toUpperCase()}`;
      msg.style.color = "#E74C3C";
    }
  }
}

init();

