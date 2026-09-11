# Luma

Luma 是 Lec 团队为 C/C++ 初学者打造的零配置桌面 IDE：**打开，就能写 C。** / **Learn. Code. Run.**

- **P0 目标平台：** Windows 10/11 x86_64、macOS Apple Silicon aarch64；当前仅 macOS 已实测。
- **准备 Zig：** 必须使用精确版本 Zig 0.16.0；参见 [`docs/BUILDING.md`](docs/BUILDING.md)。
- **构建 Luma：** `cargo build -p luma-ui --locked`。
- **运行 Luma：** `cargo run -p luma-ui --locked`。
- **预览品牌网站：** `python3 -m http.server 4173 --directory site`，打开 `http://127.0.0.1:4173/`。
- **第一个 C 程序：** 打开 `.c` 文件并按 F5；阶段 3 完成前此产品路径尚不可用。

> 当前进度：固定 GPUI 路线已通过 macOS 实机构建与 GitHub Actions `macos-15`/`windows-2025` Release 门禁。Stage 1 最小独立编辑器已实现 UTF-8 `Document`、原子 `EditBatch`、selection、undo/redo、IME 边界、真实 gutter 与双轴 caret-following；Windows 实机输入法/DPI/Narrator、macOS 人工 VoiceOver 及项目许可证仍待确认，因此跨平台阶段 0 继续保持 HOLD。

## 固定产品标识

- 团队：`Lec`
- CLI / 桌面可执行文件：`luma`
- 用户配置目录：`.luma/`
- Zig 核心静态库：`luma_core`
- macOS bundle ID：`dev.lec.luma`

当前已产出 `luma` 桌面可执行文件与 Stage 1 最小独立编辑器；`.luma/` 配置与 `luma_core` 尚未实现。

## 技术边界

规划中的核心引擎、项目系统、构建系统、运行系统、诊断系统、工具链系统和依赖管理系统将使用 Zig 0.16.0；跨平台 GPU 界面与编辑器状态层使用 Rust GPUI。后续桌面可执行文件将由 Cargo 构建并链接 Zig 静态库，边界为窄 C ABI。当前尚未创建 Zig 核心或 ABI。

当前仓库没有 Electron、WebView、React、TypeScript、Node.js、GPUIX 或插件系统。`site/` 是不参与桌面运行时的纯静态 HTML/CSS/JavaScript 品牌网站。

编辑器实现只借鉴 Zed 的事务边界、selection 不变量、IME UTF-16 边界、延迟 caret reveal 与诊断替换策略；Luma 不兼容 `.zed/` 配置，也不链接或复制 Zed 的 GPL editor 内部 crate。详见 [`docs/EDITOR_STRATEGY.md`](docs/EDITOR_STRATEGY.md)。所有验证与正式代码统一遵循 [`docs/CODE_STYLE.md`](docs/CODE_STYLE.md)。

## 阶段状态

阶段 0 的实测证据和未验证项见 [`docs/GPUI_SPIKE.md`](docs/GPUI_SPIKE.md)。当前只推进已批准的 Stage 1 最小编辑器切片；Windows 实机与人工辅助技术门禁完成前，不启动 Zig Core、项目系统、LSP 或大规模 IDE 开发。
