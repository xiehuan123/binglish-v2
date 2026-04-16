import { getCurrentWindow } from "@tauri-apps/api/window";
import { restCompleted, getUselessFact, getWallpaperInfo } from "./shared/invoke";

const REST_LOCK_SECONDS = 30;

const REST_QUOTES: [string, string][] = [
  ["Even a Ferrari needs a pit stop. You're a Ferrari, right?", "法拉利也需要进站加油。你也是法拉利，对吧？"],
  ["The computer won't run away. Promise.", "电脑不会长腿跑掉的，我保证。"],
  ["Time to blink manually.", "是时候手动眨眨眼了。"],
  ["Your spine called. It wants to be straight for a bit.", "你的脊椎打电话来了，它想稍微直一直。"],
  ["Hydrate or diedrate. Go drink water.", "要么喝水，要么枯萎。去喝水。"],
  ["404: Energy Not Found. Please reboot yourself.", "404：未找到能量。请重启你自己。"],
  ["Step away from the glowing rectangle.", "离那个发光的长方形远一点。"],
  ["You've been sitting longer than a gargoyle.", "你坐得比石像鬼还久。"],
  ["Reality is calling. It's high resolution out there.", "现实在呼唤。外面的分辨率很高的。"],
  ["Give your mouse a break. It's exhausted.", "放过你的鼠标吧，它累坏了。"],
  ["Look at something further than 50cm away. Like a wall.", "看点50厘米以外的东西。比如墙。"],
  ["If you don't rest, your bugs will multiply.", "如果你不休息，你的Bug会繁殖的。"],
  ["Stretch. You don't want to turn into a shrimp.", "伸个懒腰。你不想变成一只虾米吧。"],
  ["Pause game. Life continues.", "游戏暂停。生活继续。"],
  ["Go annoy your cat/dog/colleague for a moment.", "去骚扰一下你的猫/狗/同事吧。"],
  ["Ctrl+Alt+Del your fatigue.", "Ctrl+Alt+Del 强制结束你的疲劳。"],
  ["Loading 'Energy'... Please wait 30 seconds.", "正在加载\"能量\"... 请等待30秒。"],
  ["A rested brain is a sexy brain.", "休息过的大脑才是性感的大脑。"],
  ["Don't let the pixels hypnotize you.", "别让像素把你催眠了。"],
  ["Nature called. Not the bathroom, actual nature.", "大自然在召唤。不是指厕所，是真的大自然。"],
  ["Your chair misses your absence.", "你的椅子怀念你不在的时候。"],
  ["Refresh your soul, not just the webpage.", "刷新一下灵魂，而不只是网页。"],
  ["Keep calm and take a break.", "保持冷静，休息一下。"],
  ["System Overheat. Cooling required.", "系统过热。需要冷却。"],
  ["Battery Low. Please recharge with coffee or tea.", "电量低。请用咖啡或茶充电。"],
  ["Remember the sun? It's that bright ball in the sky.", "还记得太阳吗？就是天上那个亮球。"],
  ["Typing speed -50% due to fatigue.", "由于疲劳，打字速度 -50%。"],
  ["AFK (Away From Keyboard) for a bit.", "暂时 AFK 一下吧。"],
  ["You are not a robot. Or are you?", "你不是机器人。还是说你是？"],
  ["Your brain has left the chat. Please wait for it to reconnect.", "你的大脑已退出群聊。请等待重连。"],
  ["Touch grass. Literally.", "去摸摸草。字面意思。"],
  ["Stand up. Your butt is falling asleep.", "站起来。你的屁股睡着了。"],
  ["Taking a break is part of the algorithm.", "休息也是算法的一部分。"],
  ["Don't code a memory leak in your own brain.", "别给自己脑子里写出内存泄漏了。"],
  ["A 5-minute break saves 5 hours of debugging.", "休息5分钟，省下5小时改 Bug。"],
  ["Esc key is not just on the keyboard.", "Esc 键不只在键盘上。"],
  ["The internet will survive without you for 5 minutes.", "没你这5分钟，互联网也不会崩。"],
  ["Touch grass. Literally.", "去摸摸草。字面意思。"],
  ["Disconnect to reconnect.", "断开连接，为了更好地连接。"],
  ["Screen time is up. Real time begins.", "屏幕时间到。现实时间开始。"],
];

const appWindow = getCurrentWindow();
let countdown = REST_LOCK_SECONDS;
let quizShown = false;

async function init() {
  // 随机选一条语录
  const [en, cn] = REST_QUOTES[Math.floor(Math.random() * REST_QUOTES.length)];
  document.getElementById("quoteEn")!.textContent = en;
  document.getElementById("quoteCn")!.textContent = cn;

  // 淡入
  requestAnimationFrame(() => {
    document.getElementById("restOverlay")!.classList.add("visible");
  });

  // 倒计时
  updateCountdown();
  const timer = setInterval(() => {
    countdown--;
    updateCountdown();

    // 半程时显示单词测验
    if (countdown <= Math.floor(REST_LOCK_SECONDS / 2) && !quizShown) {
      quizShown = true;
      showWordQuiz();
    }

    if (countdown <= 0) {
      clearInterval(timer);
      const btn = document.getElementById("unlockBtn") as HTMLButtonElement;
      btn.disabled = false;
      btn.textContent = "回去工作";
      btn.classList.add("active");
    }
  }, 1000);

  // 异步获取冷知识
  try {
    const fact = await getUselessFact();
    if (fact.en) {
      document.getElementById("factEn")!.textContent = fact.en;
      document.getElementById("factCn")!.textContent = fact.cn;
      document.getElementById("factSection")!.style.display = "block";
    }
  } catch { /* ignore */ }

  // 解锁按钮
  document.getElementById("unlockBtn")!.addEventListener("click", async () => {
    if (countdown > 0) return;
    await restCompleted();
    appWindow.close();
  });
}

function updateCountdown() {
  const display = Math.max(0, countdown);
  document.getElementById("countdown")!.textContent = String(display);
  if (display > 0) {
    (document.getElementById("unlockBtn") as HTMLButtonElement).textContent =
      `请等待 ${display} 秒...`;
  }
}

async function showWordQuiz() {
  try {
    const info = await getWallpaperInfo() as { word: string | null };
    if (!info.word) return;

    const word = info.word;
    const section = document.getElementById("quizSection")!;
    section.style.display = "block";

    // 简单测验：打乱字母让用户识别
    const shuffled = word.split("").sort(() => Math.random() - 0.5).join("");
    section.innerHTML = `
      <div class="question">What word is this? <strong>${shuffled}</strong></div>
      <div class="options">
        <button class="option-btn" data-answer="${word}">${word}</button>
        <button class="option-btn" data-answer="wrong1">${generateDecoy(word, 0)}</button>
        <button class="option-btn" data-answer="wrong2">${generateDecoy(word, 1)}</button>
      </div>
    `;

    // 随机排列按钮
    const container = section.querySelector(".options")!;
    const buttons = Array.from(container.children) as HTMLElement[];
    buttons.sort(() => Math.random() - 0.5);
    buttons.forEach((b) => container.appendChild(b));

    // 点击事件
    container.addEventListener("click", (e) => {
      const btn = (e.target as HTMLElement).closest(".option-btn") as HTMLElement;
      if (!btn) return;
      const allBtns = container.querySelectorAll(".option-btn");
      allBtns.forEach((b) => {
        (b as HTMLButtonElement).disabled = true;
        if (b.getAttribute("data-answer") === word) {
          b.classList.add("correct");
        }
      });
      if (btn.getAttribute("data-answer") !== word) {
        btn.classList.add("wrong");
      }
    });
  } catch { /* ignore */ }
}

function generateDecoy(word: string, seed: number): string {
  const vowels = "aeiou";
  const chars = word.split("");
  const idx = (seed * 3 + 1) % chars.length;
  const c = chars[idx].toLowerCase();
  if (vowels.includes(c)) {
    chars[idx] = vowels[(vowels.indexOf(c) + 1) % vowels.length];
  } else {
    chars[idx] = String.fromCharCode(
      ((c.charCodeAt(0) - 97 + 3) % 26) + 97
    );
  }
  return chars.join("");
}

init();
