# Luma 代码规范

Luma 以功能边界组织代码，参考 Zed 的模块命名与 Rust 习惯，但不复制其产品内部实现。所有验证代码与正式代码遵循同一标准。

## Ponytail 原则

按以下顺序选择实现，命中后停止：复用现有代码、标准库、平台能力、已安装依赖、最少自有代码。没有当前需求和测试证据时，不添加抽象层、扩展点、配置项、依赖或“以后可能用”的接口。

简单不是把所有代码塞进一个文件；最小正确结构是让一个功能只在一个明确位置变化。

## Rust 文件与模块

- `main.rs` 只负责进程启动、平台初始化和顶层 action 注册。
- 根应用状态、状态转换和顶层 Render 放在 `app.rs`。
- 独立产品界面放在 `view/<feature>.rs`，例如 `view/landing.rs`、`view/workbench.rs`。
- 可复用控件按职责命名，例如 `text_input.rs`；不要用含混的 `input.rs`、`utils.rs`、`common.rs` 或 `misc.rs`。
- 不创建 `mod.rs`；使用 `view.rs` 声明 `view/` 下的模块。
- 只有两个以上功能确实共享且需统一修改的值，才提取共享模块。单一使用者的常量留在功能文件。
- 不为一个实现创建 trait、factory、manager 或 service。
- 模块默认私有；只按调用边界使用 `pub(super)` 或 `pub(crate)`，不要为方便全部公开。

## 命名

- 类型和文件名表达领域职责：`LumaApp`、`WorkspaceTab`、`landing.rs`。
- 函数名表达动作或结果：`open_editor`、`render_landing`、`reveal_scroll`。
- 使用完整英文单词；除 `cx`、`id`、UTF、IME 等项目或框架惯例外，不使用缩写变量名。
- 不使用 `Spike`、`Temp`、`Helper` 作为长期产品类型名。

## 状态与 UI

- 状态由拥有它的功能模块管理；render 函数只从状态生成元素，不复制业务状态。
- 同一命令只有一条执行路径。按钮、快捷键、菜单和辅助功能 Invoke 应 dispatch 同一 action。
- 交互控件必须有稳定 ID、语义 Role、可读 label、键盘焦点和可见 focus 状态。
- UI 不展示虚假能力；未实现目录选择、构建或调试时，文案必须说明实际行为。
- UTF-16 只存在于平台/LSP 边界；编辑器内部使用 UTF-8 byte offset，并通过命名区分坐标空间。

## 错误与安全

- 不使用 `unwrap`/`expect` 处理可恢复输入或运行时错误；将错误传播到 UI 或显式记录。
- 不静默丢弃失败结果。
- 不使用可能越界的直接索引处理用户文本；保持 Unicode scalar 与 grapheme 边界。
- 平台差异用小范围 `cfg` 隔离，不复制整套功能实现。

## 测试与验证

- 非平凡分支至少有一个最小可运行测试，并覆盖生产实际调用的 helper。
- 修复先覆盖根因与共享路径，再改代码。
- Rust 改动至少运行：

  ```sh
  cargo fmt --all --check
  cargo test --workspace --locked
  cargo clippy --workspace --all-targets --locked -- -D warnings
  ```

- macOS UI 改动必须启动真实 `.app` 或 debug binary 并保存窗口证据。
- hosted Windows `cargo check` 只能记录为类型/条件编译证据；不得写成 Windows 实机通过。

## 上游与许可

- 使用 GPUI public API；不得依赖 Zed 的 GPL editor 内部 crate。
- 研究 Zed editor 时只借鉴行为、不变量和分层，不逐行翻译源码。
- 改编 Apache-2.0 示例必须在源文件标注来源、固定提交、修改者和许可证，并在发行包携带许可文本。
