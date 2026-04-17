# Binglish v2

> Binglish 第二代 — 基于 Tauri 2 + Rust + TypeScript 重写的 AI 桌面英语学习应用。

每次开机自动更换壁纸，顺便学个单词。点亮屏幕，欣赏美景，邂逅知识，聚沙成塔。

## 功能

- 自动下载随机高清壁纸（picsum.photos / Bing 每日壁纸，多源备用）
- 本地词库 3920 词（CET4+CET6），含音标、中文释义、中英例句
- 每次换壁纸随机选词，渲染单词卡片到壁纸底部（半透明蒙版 + 双字体混排）
- 自定义壁纸：上传自己的图片，只换单词不换图
- 每天早上 10 点自动更新壁纸和单词
- 定时休息提醒：空闲检测 + 全屏检测 + 倒计时锁屏 + 双语励志语录 + 单词测验
- Binglish Games：Sentence Master（句子重组）+ Wordle（猜词）
- 开机自启

## 与 v1 的区别

| | v1 (Python) | v2 (Tauri) |
|---|---|---|
| 技术栈 | Python 3 + tkinter + pystray | Tauri 2 + Rust + Vanilla TypeScript |
| 打包体积 | ~30MB (PyInstaller) | ~5MB (Tauri) |
| 跨平台 | 两套独立代码 | 一套代码，条件编译 |
| 单词来源 | 服务端合成 | 本地词库（CET4+CET6） |
| 壁纸来源 | 服务端合成 | picsum / Bing API 多源 |
| 文字渲染 | 服务端 | 客户端本地渲染（Lato + 黑体双字体） |

## 项目结构

```
binglish-v2/
├── src/                              # 前端 (Vanilla TypeScript + Vite)
│   ├── index.html
│   ├── rest-overlay.html
│   ├── game-overlay.html
│   └── scripts/
│       ├── rest-overlay.ts
│       ├── game-lobby.ts
│       └── shared/invoke.ts
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── resources/
│   │   ├── words.json                # 本地词库 (CET4+CET6, 3920词)
│   │   ├── Lato-Bold.ttf             # 英文字体 (支持IPA音标)
│   │   └── SimHei.ttf                # 中文字体
│   └── src/
│       ├── lib.rs
│       ├── state.rs                  # AppState
│       ├── tray.rs                   # 系统托盘菜单
│       ├── scheduler.rs              # 每天10点定时 + 休息监控
│       ├── wallpaper_setter.rs       # 跨平台壁纸设置
│       ├── idle_detector.rs          # 跨平台空闲检测
│       ├── text_renderer.rs          # 单词卡片渲染（双字体混排）
│       ├── word_db.rs                # 本地词库查询
│       └── commands/
│           ├── wallpaper.rs          # 壁纸下载 + 渲染 + 自定义
│           ├── audio.rs              # 音频播放
│           ├── games.rs              # 游戏数据
│           └── system.rs             # 全屏检测
```

## 开发

```bash
npm install
npm run tauri dev
```

## 构建

```bash
npm run tauri build
```

## 发布

```bash
npm run release:major    # 大版本
npm run release:minor    # 小版本
npm run release          # 补丁版本
```

## Rust 依赖

reqwest · image · imageproc · ab_glyph · kamadak-exif · rodio · serde · tokio · chrono · parking_lot · rand

Tauri 插件：store · autostart · updater · shell · dialog

## 许可

与 Binglish v1 保持一致。
