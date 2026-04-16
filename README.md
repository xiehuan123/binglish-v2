# Binglish v2

> Binglish 第二代 — 基于 Tauri 2 + Rust + TypeScript 重写的 AI 桌面英语学习应用。

自动更换必应 Bing 每日壁纸，顺便学个单词。点亮屏幕，欣赏美景，邂逅知识，聚沙成塔。

## 与 v1 的区别

| | v1 (Python) | v2 (Tauri) |
|---|---|---|
| 技术栈 | Python 3 + tkinter + pystray | Tauri 2 + Rust + Vanilla TypeScript |
| 打包体积 | ~30MB (PyInstaller) | ~5MB (Tauri) |
| 跨平台 | 两套独立代码 | 一套代码，条件编译 |
| UI | tkinter 覆盖层 | WebView (HTML/CSS/TS) |
| 更新机制 | 手动下载替换 | tauri-plugin-updater 内置签名更新 |
| 配置存储 | configparser (ini) | tauri-plugin-store (JSON) |
| 音频 | playsound3 / multiprocessing | rodio (独立线程) |

## 功能

- 每 3 小时自动下载 Bing 壁纸并设置为桌面背景
- EXIF 元数据解析：单词、词典链接、发音 MP3、版权信息
- 系统托盘菜单：查单词 / 听单词 / 看单词 / 随机复习 / 复制保存 / 壁纸信息 / 分享壁纸
- 定时休息提醒：空闲检测 + 全屏检测 + 倒计时锁屏 + 双语励志语录 + 单词测验 + 随机冷知识
- Today in History：历史上的今天（中英双语）
- Binglish Games：Sentence Master（句子重组）+ Wordle（猜词）
- Song of the Day：每日歌曲推荐（rodio 播放）
- 开机自启 / 自动更新

## 项目结构

```
binglish-v2/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── src/                              # 前端 (Vanilla TypeScript + Vite)
│   ├── index.html                    # 隐藏宿主窗口
│   ├── rest-overlay.html             # 休息提醒覆盖层
│   ├── history-overlay.html          # 历史上的今天覆盖层
│   ├── game-overlay.html             # 游戏界面
│   ├── styles/
│   │   ├── base.css
│   │   ├── rest.css
│   │   ├── history.css
│   │   └── games.css
│   └── scripts/
│       ├── rest-overlay.ts
│       ├── history-overlay.ts
│       ├── game-lobby.ts             # Sentence Master + Wordle
│       └── shared/
│           ├── invoke.ts             # Tauri invoke 封装
│           └── types.ts
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── icons/                        # 全套应用图标
│   └── src/
│       ├── main.rs
│       ├── lib.rs                    # 插件注册 + app builder
│       ├── state.rs                  # AppState 共享状态
│       ├── tray.rs                   # 系统托盘菜单
│       ├── scheduler.rs              # 壁纸定时刷新 + 休息监控
│       ├── wallpaper_setter.rs       # 跨平台壁纸设置 (条件编译)
│       ├── idle_detector.rs          # 跨平台空闲检测 (条件编译)
│       └── commands/
│           ├── mod.rs
│           ├── wallpaper.rs          # 壁纸下载 + EXIF 解析
│           ├── audio.rs              # 音频播放
│           ├── history.rs            # 历史上的今天
│           ├── games.rs              # 游戏数据
│           └── system.rs             # 全屏检测、冷知识
```

## 开发

前置要求：Node.js、Rust toolchain、Tauri CLI。

```bash
cd binglish-v2
npm install
npm run tauri dev
```

## 构建

```bash
npm run tauri build
```

产物位于 `src-tauri/target/release/bundle/`。

## 后端服务

所有数据依赖 `ss.blueforge.org` 提供：

| 接口 | 用途 |
|---|---|
| `/bing` | 合成壁纸（Bing 图 + 单词 EXIF） |
| `/bing/songoftheday.json` | 每日歌曲 |
| `/bing/games.json` | 游戏数据 |
| `/bing/uselessfact.json` | 随机冷知识 |
| `/getHistory` | 历史上的今天 |

## Rust 依赖

reqwest · kamadak-exif · rodio · serde · tokio · chrono · parking_lot · windows (Win32 API)

Tauri 插件：store · autostart · updater · shell · dialog

## 许可

与 Binglish v1 保持一致。
