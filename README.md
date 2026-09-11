# Luma

Luma 是 Lec 团队为 C/C++ 初学者打造的零配置桌面 IDE：**打开，就能写 C。** / **Learn. Code. Run.**

- **P0 目标平台：** Windows 10/11 x86_64、macOS Apple Silicon aarch64；当前仅 macOS 已实测。
- **准备 Zig：** 必须使用精确版本 Zig 0.16.0；参见 [`docs/BUILDING.md`](docs/BUILDING.md)。
- **构建 Luma：** `cargo build -p luma-ui --locked`。
- **运行 Luma：** `cargo run -p luma-ui --locked`。
- **预览品牌网站：** `python3 -m http.server 4173 --directory site`，打开 `http://127.0.0.1:4173/`。
- **第一个 C 程序：** 打开 `.c` 文件并按 F5；阶段 3 完成前此产品路径尚不可用。

> 当前进度：macOS GPUI 技术路线为条件 GO；双轴 caret-following 与程序化 AX 角色/焦点已实测，人工 VoiceOver 尚待复测。Windows Release、命令入口、输入法、DPI 与 Narrator 尚未实测，因此跨平台阶段 0 保持 HOLD。项目许可证待 Lec 团队确认。

## 固定产品标识

- 团队：`Lec`
- CLI / 桌面可执行文件：`luma`
- 用户配置目录：`.luma/`
- Zig 核心静态库：`luma_core`
- macOS bundle ID：`dev.lec.luma`

阶段 0 已产出 `luma` 桌面可执行文件；`.luma/` 配置与 `luma_core` 尚未实现。

## 技术边界

规划中的核心引擎、项目系统、构建系统、运行系统、诊断系统、工具链系统和依赖管理系统将使用 Zig 0.16.0；跨平台 GPU 界面使用 Rust GPUI。后续桌面可执行文件将由 Cargo 构建并链接 Zig 静态库，边界为窄 C ABI。阶段 0 仅实现 Rust/GPUI spike，尚未创建该核心或 ABI。

当前仓库没有 Electron、WebView、React、TypeScript、Node.js、GPUIX 或插件系统。`site/` 是不参与桌面运行时的纯静态 HTML/CSS/JavaScript 品牌网站。

编辑器实现只借鉴 Zed 的事务边界、selection 不变量、IME UTF-16 边界、延迟 caret reveal 与诊断替换策略；Luma 不兼容 `.zed/` 配置，也不链接或复制 Zed 的 GPL editor 内部 crate。详见 [`docs/EDITOR_STRATEGY.md`](docs/EDITOR_STRATEGY.md)。所有验证与正式代码统一遵循 [`docs/CODE_STYLE.md`](docs/CODE_STYLE.md)。

## 阶段状态

阶段 0 的实测证据和未验证项见 [`docs/GPUI_SPIKE.md`](docs/GPUI_SPIKE.md)。阶段 0 未通过前，不进入大规模 IDE 开发。
