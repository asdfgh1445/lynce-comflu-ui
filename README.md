# Lynce AI 桌面出图工作站

**仅限 Mac**（Apple Silicon，M 系列芯片）的本地 AI 绘图工作站：**Lynce 启动器**（Tauri 2）+ **SwarmUI / ComfyUI** 后端 + **liblib 风格三模块网页界面**。一个 App 管安装、启动、模型、扩展，浏览器里出图。

> 本项目不做 Windows/Linux 适配。要求 Apple Silicon Mac、macOS 13+，16GB 内存可跑（全程 MPS 加速，无需 NVIDIA 卡）。

## 目录结构

```
├── aki-mac-launcher/     # Lynce 启动器源码（Tauri 2：Rust 后端 + React 前端）
│   └── src-tauri/target/release/bundle/   # v0.2 安装包（Lynce.app / dmg）
├── liblib-ui/            # 网页界面母本（图像生成 / Web UI / ComfyUI 工作流 三标签）
├── runtime/              # 本机运行环境（SwarmUI + ComfyUI，见 .gitignore：不随项目分发）
├── README.md             # 本文件
└── .gitignore
```

## 快速开始（新机器）

**准备**：Apple Silicon Mac、macOS 13+、16GB+ 内存；模型自备（见下）。

1. 安装 `aki-mac-launcher/src-tauri/target/release/bundle/dmg/Lynce_0.2.0_aarch64.dmg`（或自行构建，见下）
2. 打开 Lynce → 「环境与安装」页 → 一键安装 SwarmUI 与 ComfyUI
   - 自动 clone + 建 venv + 装依赖，**pip 默认走国内镜像**（豆瓣→腾讯→阿里→清华→官方回退；官方 PyPI 在国内网络经常卡死）
   - SwarmUI 首次启动会自动下载 .NET 运行时（装到 `~/.dotnet`，一次性）
3. 「启动」页启动 SwarmUI（默认 `http://127.0.0.1:7801`），App 内「工作台」即出图界面
4. 独立 ComfyUI（默认 8188 端口）是高级选项，按需安装启动

**模型自备**：把 checkpoint / LoRA / VAE 等放到 SwarmUI 的 `Models/` 对应子目录（启动器安装后位于 `~/SwarmUI/Models`，可在「高级选项」里改目录）。本项目不附带任何模型。

**从源码构建启动器**：`cd aki-mac-launcher && npm install && npm run tauri build`（需要 Node 18+、Rust、Xcode Command Line Tools）。

## 界面入口

启动 SwarmUI 后（默认端口 7801）：

- `http://127.0.0.1:7801/liblib.html` — 三模块主界面（图像生成 / Web UI / ComfyUI 工作流）
- `http://127.0.0.1:7801/` — SwarmUI 原生界面
- `http://127.0.0.1:7821` — ComfyUI 后端 API（SwarmUI 自动拉起）

界面母本在 `liblib-ui/`，改完部署到 SwarmUI 的 `src/Content/wwwroot/` 即可。

## 实测性能参考（M5 / 16GB）

| 分辨率 | 步数 | 耗时 |
|---|---|---|
| 512×512 | 16 | ≈ 8 分钟 |
| 768×768 | 16 | ≈ 18 分钟 |
| 1024×1024 | 16 | ≈ 38–50 分钟 |

- 16GB 内存跑图时**关闭多余 Chrome 标签**，1024 生成期间不要刷新页面
- 页面疑似卡死：`POST :7821/interrupt` + `POST :7821/queue -d '{"clear":true}'`，必要时杀掉 ComfyUI 进程（SwarmUI 会自动重新拉起）

## 已知约定 / 坑

- 界面 iframe 有 HTTP 缓存，改 HTML 后需 `?v=时间戳` 强刷
- ComfyUI 首次启动需 1–3 分钟，属正常
- 启动器 App 标识目前是 `com.zhaomingzhe.lynce`，自行分发时可改 `aki-mac-launcher/src-tauri/tauri.conf.json`
- 若把本项目放在**带空格的路径**下运行：Python venv 的控制台脚本 shebang 不支持带空格路径，本项目通过 `~/.lynce-shims/`（无空格符号链接指向 venv）解决；工作区移动位置后需重建：
  ```bash
  mkdir -p ~/.lynce-shims
  ln -sfn "<项目路径>/runtime/SwarmUI/dlbackend/ComfyUI/venv" ~/.lynce-shims/dlbackend-venv
  ln -sfn "<项目路径>/runtime/ComfyUI/venv" ~/.lynce-shims/comfy-venv
  ```
  新机器用启动器全新安装则无此问题。
