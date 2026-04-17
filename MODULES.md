# Binglish 模块架构分析

> Binglish — AI 桌面英语，自动更换 Bing 每日壁纸并叠加 AI 生成的英语单词学习内容。
> 版本：Windows v1.4.0 / macOS v1.0.2

## 项目概览

Binglish 是一个单文件桌面应用，采用 Python 3 编写，通过 PyInstaller 打包为独立可执行文件。项目按平台分为两个入口文件，共享相同的逻辑架构，所有状态通过全局变量管理。

```
binglish/
├── binglish.py            # Windows 主程序 (1756 行)
├── binglish.ico           # Windows 托盘图标
├── bundle.bat             # Windows PyInstaller 打包脚本
├── requirements.txt       # Windows 依赖
├── README.md
└── macos/
    ├── binglish_mac.py    # macOS 主程序 (1198 行)
    ├── binglish_mac.png   # macOS 托盘图标
    └── requirements.txt   # macOS 依赖
```

## 技术栈

| 层面 | 技术 |
|------|------|
| 语言 | Python 3 |
| GUI | tkinter (覆盖层 UI) + pystray (系统托盘) |
| 网络 | requests |
| 图像 | Pillow, exifread, qrcode |
| 音频 | playsound3 |
| 打包 | PyInstaller (--onefile --windowed) |
| 平台 API | Windows: ctypes.windll, winreg / macOS: osascript, ioreg |

## 模块划分

虽然代码以单文件形式组织，但逻辑上可划分为以下 12 个模块：

---

### 1. 常量与配置 (Constants & Configuration)

定义版本号、远程服务 URL、刷新间隔等全局常量，以及约 120 条中英双语休息提醒语录 (`REST_QUOTES`)。

**关键 URL：**
- `IMAGE_URL` — 壁纸图片接口 (`ss.blueforge.org/bing`)
- `MUSIC_JSON_URL` — 每日歌曲元数据
- `RELEASE_JSON_URL` — 版本更新信息
- `GAME_DATA_URL` — 游戏数据接口
- `HISTORY_URL_BASE` — 历史上的今天
- `USELESS_FACT_URL` — 随机冷知识

---

### 2. 配置持久化 (Config Persistence)

通过 `configparser` 读写 `binglish.ini`，持久化用户偏好设置。

**配置项：** 休息提醒开关、提醒间隔、空闲重置时间、锁屏时长、覆盖层颜色。

---

### 3. 原生对话框 (Native Dialog Helpers)

- **Windows：** 直接使用 `tkinter.messagebox`
- **macOS：** 封装 AppleScript 调用 (`mac_alert`, `mac_askyesno`)，提供原生外观的对话框

---

### 4. 壁纸引擎 (Wallpaper Engine)

**核心函数：** `update_wallpaper_job`

工作流程：
1. 从服务器下载合成壁纸图片
2. 通过 `exifread` 提取 EXIF 元数据（单词、词典 URL、MP3 URL、版权、分享 ID）
3. 获取每日歌曲元数据
4. 调用系统 API 设置桌面壁纸
5. 刷新托盘菜单内容

**平台差异：**
- Windows: `SystemParametersInfoW` (ctypes)
- macOS: `osascript` 执行 AppleScript

---

### 5. 定时调度器 (Scheduler)

**核心函数：** `run_scheduler`

启动时等待网络连接就绪，触发首次壁纸更新，之后每 3 小时循环刷新。

---

### 6. 休息提醒监控 (Rest Monitor)

**核心函数：** `rest_monitor_loop`

后台线程，每 5 秒轮询一次：
- 检测系统空闲时间（空闲 > 5 分钟则重置计时器）
- 活跃工作超过 45 分钟（可配置）时触发休息覆盖层
- Windows 额外检测：若前台为全屏应用则延迟提醒

**空闲检测：**
- Windows: `ctypes.windll.user32.GetLastInputInfo`
- macOS: `ioreg` 命令读取 `HIDIdleTime`

---

### 7. 覆盖层 UI (Overlay UI Renderers)

全屏 tkinter 覆盖层，包含三类：

| 覆盖层 | 功能 |
|--------|------|
| 休息提醒 | 倒计时锁屏 + 双语励志语录 + 当前单词测验 + 随机冷知识 |
| 历史上的今天 | 可滚动的历史事件列表 (WikiMedia) |
| 英语游戏 | 游戏大厅 + Sentence Master (造句) + Wordle (猜词) |

**平台差异：**
- Windows: 在隐藏的 `Tk()` 根窗口上创建 `Toplevel` 子窗口
- macOS: 每个覆盖层运行在独立的 `multiprocessing.Process` 中，避免 tkinter 线程安全问题

---

### 8. 音乐播放器 (Music Player)

通过 `multiprocessing.Process` 启动 `playsound3` 播放流媒体音频。监控线程/定时器检测播放结束并更新菜单状态。

**功能：** Song of the Day — 每个工作日推荐一首外文歌曲（来源 NPR）。

---

### 9. 自动更新 (Auto-Updater)

更新流程：
1. 检查远程 `release.json` / `release_mac.json` 获取最新版本
2. 下载新版二进制文件
3. SHA256 校验完整性
4. 生成平台更新脚本（Windows: `updater.bat` / macOS: `updater.sh`）
5. 脚本替换当前可执行文件并重启应用

---

### 10. 开机自启管理 (Startup Management)

- **Windows：** 读写注册表 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
- **macOS：** 创建/移除 `~/Library/LaunchAgents/com.binglish.app.plist`

---

### 11. 托盘菜单构建 (Tray Menu Builder)

**核心函数：** `build_menu_items`

根据当前状态动态构建系统托盘右键菜单：

```
查单词 → 必应词典
读单词 → AI 语音讲解
看单词 → 影视片段
随机复习 → 往期壁纸
复制保存 → 保存当前壁纸
图片信息 → 版权与内容信息
分享壁纸 → 二维码分享
提醒休息 → 定时休息 (可配置)
Today in History → 历史上的今天
Binglish Games → Sentence Master / Wordle
Song of the Day → 每日歌曲
检查更新 → 自动更新
开机自启 → 注册表/LaunchAgent
退出
```

---

### 12. 程序入口 (Entry Point)

**核心函数：** `main`

启动顺序：
1. 初始化配置 (`binglish.ini`)
2. 加载托盘图标 (`pystray.Icon`)
3. 启动调度器线程（壁纸刷新）
4. 启动休息监控线程
5. 进入主事件循环
   - Windows: `root.mainloop()` (tkinter)
   - macOS: `icon.run()` (pystray)

## 模块依赖关系

```
┌─────────────┐
│  程序入口    │
│   main()    │
└──────┬──────┘
       │ 启动
       ├──────────────┬───────────────┬──────────────┐
       ▼              ▼               ▼              ▼
┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────┐
│ 托盘菜单   │ │ 定时调度器 │ │ 休息监控   │ │ 配置持久化│
│ 构建       │ │            │ │            │ │          │
└─────┬──────┘ └─────┬──────┘ └─────┬──────┘ └──────────┘
      │              │              │
      │              ▼              ▼
      │        ┌────────────┐ ┌────────────┐
      │        │ 壁纸引擎   │ │ 覆盖层 UI  │
      │        └─────┬──────┘ └────────────┘
      │              │
      ├──────────────┼──────────────┐
      ▼              ▼              ▼
┌────────────┐ ┌────────────┐ ┌────────────┐
│ 音乐播放器 │ │ 自动更新   │ │ 开机自启   │
└────────────┘ └────────────┘ └────────────┘
```

## 后端服务依赖

所有数据均依赖 `ss.blueforge.org` 服务器提供：

| 接口 | 用途 |
|------|------|
| `/bing` | 合成壁纸图片（Bing 图 + 单词信息嵌入 EXIF） |
| `/bing/music.json` | 每日歌曲元数据 |
| `/bing/release.json` | Windows 版本更新信息 |
| `/bing/release_mac.json` | macOS 版本更新信息 |
| `/bing/game_data` | 游戏数据 (Sentence Master / Wordle) |
| `/bing/history` | 历史上的今天 |
| `/bing/useless_fact` | 随机冷知识 |
| `/bing/share` | 壁纸分享页面 |

## 平台差异总结

| 特性 | Windows | macOS |
|------|---------|-------|
| 设置壁纸 | `SystemParametersInfoW` (ctypes) | `osascript` AppleScript |
| 空闲检测 | `GetLastInputInfo` (ctypes) | `ioreg` HIDIdleTime |
| 对话框 | `tkinter.messagebox` | AppleScript (`mac_alert`) |
| 覆盖层进程模型 | `Toplevel` 子窗口 (单进程) | `multiprocessing.Process` (多进程) |
| 开机自启 | 注册表 `HKCU\...\Run` | LaunchAgents plist |
| 音效 | `winsound.Beep` (ctypes) | `afplay` |
| DPI 感知 | `SetProcessDpiAwareness` | N/A |
| 全屏检测 | `is_foreground_fullscreen` | N/A |
