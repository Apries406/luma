# GPUI 技术验证（阶段 0）

## 结论

- **macOS Apple Silicon 技术路线：条件 GO。** 当前固定版本可以创建原生 GPUI 窗口，并完成窗口、绘制、输入、IME、选择、剪贴板、焦点、macOS 菜单、快捷键、双轴 caret-following、Accessibility 基线与本地 Release bundle 验证。Stage 1 最小独立编辑器已替换源码区输入原型；真实中文 IME 仍需在新 controller 上人工复测。
- **跨平台阶段 0：HOLD。** GitHub Actions `macos-15` 与 `windows-2025` 已完成真实 Release build；Windows job 还成功启动 EXE 并发现顶层窗口。但仍没有 Windows 10/11 交互式微软拼音、150%/200% DPI 或 Narrator 实机证据，因此不能宣布 Windows 产品验收通过。

验证日期：2026-09-11 CST。

## 固定版本

- GPUI：声明版本 `0.2.2`
- `gpui_platform`：声明版本 `0.1.0`
- 上游：Zed Industries `zed`
- 完整提交：`251854020a9dbea1a388bf392c9bd04fd136a557`
- Rust：`1.97.1` stable
- Zig：`0.16.0`
- GPUI 许可证：Apache-2.0
- macOS `gpui_platform` feature：`font-kit`
- Windows `gpui_platform` feature：无额外 feature

不使用 branch、`gpui = "*"`、GPUIX 或 crates.io 同名源码替代品。选择该 Git 提交的原因记录在 [`BUILDING.md`](BUILDING.md)。上游依据：

- [GPUI Cargo.toml](https://github.com/zed-industries/zed/blob/251854020a9dbea1a388bf392c9bd04fd136a557/crates/gpui/Cargo.toml)
- [gpui_platform Cargo.toml](https://github.com/zed-industries/zed/blob/251854020a9dbea1a388bf392c9bd04fd136a557/crates/gpui_platform/Cargo.toml)
- [官方 input 示例](https://github.com/zed-industries/zed/blob/251854020a9dbea1a388bf392c9bd04fd136a557/crates/gpui/examples/input.rs)
- [官方 Accessibility 示例](https://github.com/zed-industries/zed/blob/251854020a9dbea1a388bf392c9bd04fd136a557/crates/gpui/examples/a11y.rs)
- [固定 Rust toolchain](https://github.com/zed-industries/zed/blob/251854020a9dbea1a388bf392c9bd04fd136a557/rust-toolchain.toml)

## 实际环境

- 主机：macOS 26.6.2（Build 25G83），Apple Silicon arm64
- Xcode：26.6（17F113）
- macOS SDK：26.5
- Metal Toolchain：安装后可用；初次构建曾因缺少 `metal` 失败
- 初始 PATH 中没有 Zig、rustc 或 cargo
- 已从官方发布源校验 SHA-256 后，在忽略目录 `.tools/` 安装 Zig 0.16.0 与 Rust 1.97.1；没有静默使用其他版本
- Zig macOS arm64 归档 SHA-256：`b23d70deaa879b5c2d486ed3316f7eaa53e84acf6fc9cc747de152450d401489`

## 验证矩阵

| 项目 | macOS 结果 | 证据/限制 |
|---|---|---|
| 创建窗口 | 通过 | Debug 与 Release 进程均创建可枚举原生窗口；标题为 `Luma — Learn. Code. Run.` |
| 原生 UI 绘制 | 通过 | 实际窗口截图确认浅色品牌 Landing 与深色 IDE workbench 均由 GPUI 绘制 |
| 文本绘制 | 通过 | 中英文、示例 `main.c`、占位符、光标和选区均实际绘制 |
| 按钮点击 | 通过 | 真实鼠标点击 `Start Coding` 后进入可编辑 `main.c` workbench；Run 状态更新也已验证 |
| 键盘事件 | 通过 | Cmd+Enter 后计数从 0 变为 1；事件历史显示 `cmd-enter` |
| 单行文本输入 | 通过 | 实际输入、光标、删除与选择通过；Home/End 已接入，但没有单独保留视觉证据 |
| 多行文本输入 | 通过 | 实际提交 `你好`，Enter 后第二行输入 `line2` |
| 中文输入法 | 通过 | 系统简体拼音 `com.apple.inputmethod.SCIM.ITABC`；实际看到 marked text `ni hao`、系统候选窗首选“你好”，并提交为 UTF-8 `你好` |
| UTF-8 / UTF-16 范围 | 通过 | 单元测试覆盖汉字、组合字符和 emoji，不在 Unicode scalar 中间切分 |
| 扩展字素移动 | 通过 | 单元测试覆盖 `👩‍💻` 与组合重音字符边界 |
| 中文文件名显示 | 通过 | 窗口实际显示 `你好.cpp` |
| 剪贴板复制/剪切/粘贴 | 通过 | Luma Cmd+C 后在 UTF-8 locale 读得预期文本；Cmd+X 把 `cuttest` 写入剪贴板并清空输入；Cmd+V 将指定 `copytest` 写入聚焦输入框 |
| 鼠标文本选择 | 通过 | AXRaise 后真实 mouse-down/drag/up，截图显示选择高亮 |
| caret-following 内部滚动 | 通过 | 生产 `TextInput` 使用一个负像素 offset；粘贴 64 行后 caret 仍位于可视区域，滚动后点击可见第 49 行并插入 `x`。关闭软换行后，粘贴 600 字符单行，行尾 `END` 与 caret 仍可见；再次点击可见中段并插入 `x`，确认双轴 mouse hit-test 与最终 `TextLayout` 坐标一致。IME range/candidate geometry 使用同一 layout，并在 offset 变化后请求平台重查，但长组合串仍需 Windows 实机验证 |
| 焦点切换 | 通过 | 闭环断言为 `initial=single → Tab=multi → Shift+Tab=single`；Tab/Shift+Tab 使用 GPUI 焦点链 |
| 命令入口 | macOS 通过 / Windows 待实测 | macOS AX 树存在 `Luma → Run / Services / Quit Luma`；AXPress Run 后计数从 0 变为 1。固定 GPUI Windows 后端不会显示 `set_menus` 模型；Luma 因此提供 Windows-only、可聚焦的 client-side `Run  Ctrl+Enter` 按钮，dispatch 同一个 `Activate` action，但仍需 Windows/Narrator/DPI 实测 |
| macOS Retina | 通过 | 760×640 级逻辑窗口截图约为 1656×1482 像素（包含窗口阴影），文本与光标清晰 |
| 基础 Accessibility | 历史程序化角色/焦点通过，最新重验受环境阻塞；VoiceOver 人工待测 | 文件搜索框定义为 `AXTextField`，源码编辑器定义为 `AXTextArea`，label 分别为 `File search`、`Source editor`。Stage 0 Release 曾连续 3/3 返回 `AXTextArea / Source editor`，Stage 1 初版也曾通过并完成真实 typing/undo/redo。最终重验时当前桌面环境只向 AX 暴露循环的 `AXApplication`；不可变 Stage 0 提交 `af609c8` 在同环境同样失败，故不能判定为 Stage 1 回归，也不能把本轮记为通过。VoiceOver 全程保持关闭 |
| 最小本地 Release bundle | 通过 | `scripts/package-macos.sh release`，`plutil` 与严格 `codesign --verify` 通过，bundle ID `dev.lec.luma`；arm64 Mach-O 可执行文件启动并创建窗口。`Contents/Resources/ThirdPartyLicenses` 已包含 GPUI/Zed Apache-2.0 文本与来源清单。仅 ad-hoc 签名，`spctl` 会拒绝，不是可分发包 |
| Release 二进制体积 | 通过 | Stage 1 Cargo 产物 `target/release/luma` 为 `6,297,520` bytes；bundle 内重签后的可执行文件为 `6,279,024` bytes |
| Release `.app` 体积 | 通过 | Stage 1 ad-hoc 重签且加入第三方许可文本后 `du` 为 6,156 KiB，不含 Zig 工具链 |
| Zig 工具链体积 | 记录 | 单独统计 `.tools/zig-0.16.0/zig` 为 181,004 KiB，不计入 Luma `.app` |
| Windows Release 构建/启动 | CI 通过 / 交互实机待测 | GitHub Actions run [`34584248930`](https://github.com/Apries406/luma/actions/runs/34584248930) 的 `windows-2025` job `103214655404` 已通过 fmt、test、Clippy、Release build、EXE 启动与顶层窗口 smoke。该证据确认 CI runner 可构建和启动，但不替代 Windows 10/11 人工输入、DPI、Narrator 与 GPU/驱动验收 |
| Windows 150% / 200% DPI | **未验证** | 必须 Windows 实机或 VM |
| Windows 微软拼音 / Narrator | **未验证** | 必须 Windows 实机或 VM |

最终本地产物 SHA-256（用于本次工作区核对，不替代 Git commit 或 CI artifact）：

```text
4e311c769dfe573817190b42356f5a2ff633b0213ec808d6bfcc094ab9a0e175  target/release/luma
5db0cf0552c89d4de25037befe4336fa377ff65888d59274ca0e27910de0c330  dist/Luma.app/Contents/MacOS/luma
daf46ca31e92bdac303f603c582a2e451d87143058ee5210b3752f0484f1c3b3  dist/Luma.app/Contents/Info.plist
5683044430784a7f90c7fe391c217be5f684051ed505c428d7cb59ab78680d57  LICENSES/Apache-2.0-Zed.txt
```

## 质量门禁

以下命令在最终源码上实际成功：

```sh
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check --workspace --locked --target x86_64-pc-windows-msvc
./scripts/package-macos.sh release
# 启动 Release 后：xcrun swift scripts/verify-macos-accessibility.swift <luma-pid>
```

结果：

- 单元测试：22 passed，0 failed（含原子 multi-edit、stale/重叠/非法 UTF-8 batch、CRLF、Unicode selection、undo/redo selection、typing merge/barrier、UTF-16/IME 与双轴 caret reveal）
- macOS AX：历史 Release harness 曾连续 3/3 通过，Stage 1 初版也曾返回 `AXTextArea / Source editor` 并完成真实编辑链；最终重验的 AX provider 环境异常同样影响不可变 Stage 0 基线，故本轮不新增 AX pass。测试前后 VoiceOver 保持关闭
- Clippy：0 项 Luma warning/error
- Release：本地 bundle 构建成功；ad-hoc `codesign --verify --deep --strict` 成功；未做 Developer ID 签名或公证，`spctl` 拒绝
- 已知依赖提示：`block v0.1.6` 被 Cargo 标为 future-incompatibility；这是锁定 GPUI 依赖树中的上游依赖，不是 Luma Clippy 失败

## 验证中发现并修复的问题

1. 输入框在滚动 flex 容器中被默认收缩，导致文字只显示下半部。为输入控件增加 `flex_shrink_0()` 后，以重建和窗口截图确认修复。
2. 原生菜单的 `Activate` action 最初只监听视图焦点路径；菜单 AXPress 成功但计数不变。改为应用级 `cx.on_action` 后，菜单与 Cmd/Ctrl+Enter 共用同一处理器，复测通过。
3. `osascript` 没有 Accessibility 授权，不能作为证据工具；改用已授权的 macOS AX API、CoreGraphics 事件和精确窗口截图。直接向 PID 投递按键会绕过输入法，因此中文 IME 最终使用真实全局键盘事件并观察系统候选窗。
4. UTF-16 range 的起止端最初都向右裁剪，代理对内部查询会把 `a😀b` 的 `2..3` 错变成空区间。新增先失败的回归测试后，改为非空范围起点向左、终点向右；空 caret 仍保持单点。
5. IME 更新已有 marked text 时，replacement range 最初被错误当作文档绝对位置。新增“非零前缀 + 相对组合区 range”的先失败测试后，composition 更新相对 marked range 解析，commit 则替换完整 marked range。最终二进制真实拼音提交 `你` 仍通过；非零前缀场景因当前前台事件自动化不稳定，以回归测试锁定并保留人工复测项。
6. 多行框最初错误暴露为单行 `Role::TextInput`；已改为 `Role::MultilineTextInput`。Windows Release 也增加 GUI subsystem，避免伴随控制台窗口，并补充 Ctrl+Q。辅助技术 `SetValue` 若遇到 active native composition 会有与切焦相同的同步风险，因此阶段 0 不注册不安全的伪处理器，列入后续输入模型工作。
7. 首版脚本只复制 linker-signed Mach-O，完成 bundle 后严格签名验证失败。现在 bundle 完成后重新 ad-hoc 签整个 `.app`，`codesign --verify --deep --strict` 已通过；`spctl` 因无 Developer ID/公证仍按预期拒绝。
8. 打包脚本最初写死 `target/`，可能在设置 `CARGO_TARGET_DIR` 时复制旧二进制；现在解析相对/绝对自定义 target 目录，并已用 `/tmp` symlink target 实测。
9. 单体 `main.rs` 同时承担启动、状态与两个页面渲染；现按功能拆为 `main.rs`、`app.rs`、`view/landing.rs`、`view/workbench.rs` 与独立 `text_input.rs`，不增加依赖或抽象层。规范见 [`CODE_STYLE.md`](CODE_STYLE.md)。
10. 首个 caret 滚动尝试使用 `ScrollHandle`，但 pinned GPUI 的 div scroll extent 可能低估 nowrap glyph 宽度。最终改为 `TextInput` 私有像素 offset，并让绘制、鼠标 hit-test 和 IME geometry 共用最终 `TextLayout`；纯 helper 有双轴最小滚动单测。
11. workbench 曾绘制固定 `1..=18` 行号，文本滚动后会显示错误行号。阶段 0 删除该虚假 gutter；等正式 `LayoutViewport` 能提供真实可见行时再恢复。
12. 多行输入原型原先允许软换行，导致无法验证代码行的水平 caret reveal；改为 GPUI `whitespace_nowrap`。Pinned GPUI 只将 `wrap_width` 设为 `None`，仍保留 `\n` 硬换行；600 字符单行及滚动后 hit-test 实测通过。
13. `Role::MultilineTextInput` 已正确映射为 `AXTextArea`，但节点 `AXFocused=true` 时 application 曾仍返回窗口。增加 `.focusable()` 不改变结果并已回退；源码追踪确认 `GPUIWindow` 缺少 AccessKit 官方 window→content-view focus forwarder。Luma 仅在 macOS 启动时调用 `accesskit_macos::add_focus_forwarder_to_window_class("GPUIWindow")`，未修改 vendored GPUI；持久化 AX harness 在 Release 上连续 3/3 通过。
14. `input.rs`/`editor_input` 容易把 Stage 0 原型误解为生产编辑器，先改名为 `text_input.rs`/`source_input`；Stage 1 进一步让 `TextInput` 只服务文件搜索，并由独立 `EditorController` 接管源码区。未接入 GPL-3.0-or-later 的 Zed editor crate。
15. 固定假行号已由真实 `Document::line_count` gutter 替代；当前行、dirty marker 与 `Ln/Col` 都直接读取 editor 状态。初版 Release 截图暴露 gutter 与源码 8 px 上边距不一致；现统一使用 `EDITOR_PADDING`，并把双轴 scroll 上限计入 padding。
16. Backspace/Delete 内部会临时扩展 selection；若此时才创建 history snapshot，undo 会恢复错误选区。controller 现在在命令开始前保存 selection，并用 AX 聚焦后的真实按键验证 typing→undo→redo→Backspace→undo 全链路。
17. 独立审查发现 typing merge 会跨切焦、viewport resize 不会重新 reveal caret、非文本 clipboard 会遗留一次 barrier。现分别在 focus-out 插入 history barrier、把 viewport size 纳入 reveal 判定、只在 clipboard 实际含文本时设置 paste barrier；resize 行为有同 revision 回归测试。

## 已知风险

1. GPUI 仍是 pre-1.0，版本间可能有破坏性变化；必须保持完整 Git SHA 与 `Cargo.lock`。
2. 源码区已使用独立最小编辑器，但 v1 仍采用全文 `String` 与全文 history snapshot；没有 rope、虚拟化、语法树、高亮、多光标或海量文本优化。只有真实大文件需求与性能数据出现后才升级文本结构。
3. `LayoutViewport` 使用 editor 自己的像素 offset，并统一应用于绘制、mouse hit-test 和 IME range geometry；它不是可自由拖动的完整滚动模型。超长 IME composition、窗口 resize 与候选框位置仍需 Windows 实机验证。
4. editor 在 selection command、unmark 与 undo 前安全结束本地 composition，但 GPUI 没有公开的跨平台 API 主动取消 native composition；跨输入框切焦时的组合态必须在 Windows/macOS 后续专门验证。基于同一原因，辅助技术 `SetValue` 尚未实现，避免 native IME 后续更新覆盖错误文本。
5. 固定 GPUI 的 Windows `set_menus` 只保存菜单模型，不创建原生 HMENU；Luma 已增加 Windows-only client-side Run command 作为最小 command-entry alternative，但它不是 HMENU，必须由产品确认并在 Windows/Narrator/DPI 实机验收。
6. 当前 macOS 主机只能借助未提交的临时 Zig RC shim 完成 Windows target hosted `cargo check`；真实 Windows Release 编译、链接与顶层窗口 smoke 由 GitHub `windows-2025` runner 通过。DPI、微软拼音、Narrator、GPU/驱动和发行打包仍必须在可交互 Windows 10/11 环境验证。
7. 当前尚未实现 Zig Core、`luma_core` 静态库、稳定 C ABI、项目系统或 F5 构建/运行；这些属于后续阶段，不能由本次 spike 推断为已完成。
8. 阶段 0 的 `.app` 仅是 ad-hoc 签名的最小本地 bundle，尚未加入应用图标、Developer ID 签名、公证或发行安装器。
9. 初始不可变提交为 `af609c805ac322b191ad99674acc71cb78ecb22d`，对应 GitHub Actions run [`34584248930`](https://github.com/Apries406/luma/actions/runs/34584248930)；macOS job `103214655180` 与 Windows job `103214655404` 全部成功。CI artifact 不能替代 Windows 人工截图与交互证据。
10. CI 第三方 Actions 已固定到完整 commit SHA；`actions/checkout` 与 `mlugg/setup-zig` 的 SHA 已分别核对远端 `v4`/`v2` ref，Rust SHA 已核对远端 `1.97.1` branch，且该固定 commit 的 `action.yml` 内嵌精确 toolchain。Workflow 使用 `contents: read` 最小权限并禁用 checkout token 持久化，还包括手动触发、45 分钟 timeout、macOS bundle lint/signature、Windows Release 顶层窗口 smoke 与二进制 SHA-256 日志；首次远端 matrix 已通过。本机没有 `actionlint`/`pwsh`，YAML 另经本地解析与静态复核。
11. macOS AX 角色和 application focus 曾用持久化 native harness 通过；当前桌面环境的 AX provider 异常也可在不可变 Stage 0 基线复现，须在环境恢复后重跑当前 Stage 1。VoiceOver 的实际语音、导航顺序和编辑反馈只能由用户人工复测，不能由 AX API 断言替代；自动化不得自行开启或切换系统 VoiceOver。
12. 项目自身许可证仍待 Lec 团队确认。`cargo metadata --locked` 已审计 632 个依赖且 `license`/`license_file` 缺失为 0；发行前仍要生成并人工复核完整第三方 notice 文件，而不只是元数据统计。

## 阶段 0 剩余门禁

跨平台阶段 0 保持 **HOLD**。远端 `windows-2025` fmt、test、Clippy、Release build 与启动 smoke 已通过；改为整体 GO 前仍需：
1. 由产品确认现有 Windows client-side Run command 可作为 command-entry alternative；若不接受，再实现完整 client-side menu。caret-following 内部滚动代码已完成，但仍需 Windows 实机验证。
2. 在当前 AX provider 环境恢复后重跑 Stage 1 程序化 `AXFocusedUIElement`，并由用户完成人工 VoiceOver 导航、label 与源码编辑反馈复测。
3. 在 Windows 10 或 11 x86_64 上实际启动 Release 窗口，至少完成 [`WINDOWS_TEST_CHECKLIST.md`](WINDOWS_TEST_CHECKLIST.md) 中的窗口、文本、单/多行输入、跨输入框 IME 组合切焦、微软拼音提交、剪贴板、命令入口、150%/200% DPI 与 Narrator P0 项。
4. 记录 Windows EXE 与包体积、系统版本、GPU/驱动和截图；未执行项继续标为“未验证”。

在此之前，只能说 **macOS GPUI 技术路线已验证**，不能说 **Luma 跨平台阶段 0 已完成**。
