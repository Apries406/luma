# Luma 编辑器实现策略

## 目标与边界

Luma 借鉴 Zed 编辑器的分层、事务边界和行为不变量，但独立实现源码，不兼容 `.zed/` 配置，也不依赖 Zed 的 `editor`、`text`、`language`、`multi_buffer` 等内部 crate。Lec 已确认选择独立实现最小编辑器，不采用 GPL Zed editor 直接集成路线。

固定 Zed 提交 `251854020a9dbea1a388bf392c9bd04fd136a557` 中，GPUI 与 `gpui_platform` 标记为 Apache-2.0；Zed editor crate 标记为 GPL-3.0-or-later。因此：

- 可以通过公开 API 使用 GPUI，并在遵守 Apache-2.0 的前提下改编 GPUI 示例。
- 可以研究 Zed editor 的行为、接口与架构思想。
- 不复制、逐行翻译或改编 GPL editor 源码、独特注释、命名和内部数据结构。
- 独立实现应保留设计说明与测试作为来源和行为证据。

## Stage 1 当前实现

- `editor/document.rs` 实现 UTF-8 `Document`、LF 规范化、行起点、单调 revision 与原子 `EditBatch`。
- `editor/history.rs` 实现全文 snapshot 版 v1 undo/redo；同时恢复 selection，连续 typing 合并，paste/IME 等形成 barrier，新编辑清空 redo。
- `editor/session.rs` 实现单 selection、anchor/active 方向、preferred horizontal column 与带 transaction token 的 composition。
- `editor/controller.rs` 独立承担 GPUI 键盘、鼠标、clipboard、IME 与 Accessibility；源码编辑不再复用搜索框 `TextInput`。
- `editor/layout.rs` 保存 document revision、双轴 scroll offset 与延迟 caret reveal 请求；最终 shifted `TextLayout` 同时服务 paint、hit test、caret 和 IME geometry。
- workbench gutter 由 `Document::line_count` 实时生成，并显示当前行、dirty marker 与 `Ln/Col`；不再绘制固定假行号。
- 当前实现仍以全文 `String`/snapshot 为最小正确 v1；大文件 rope、虚拟化、语法高亮、文件 I/O、LSP 与多光标均未进入本切片。

## 最小分层

1. **Document**：UTF-8 文本、LF 规范化、行起点索引、单调 revision；原子应用同一 revision 的非重叠 `EditBatch`。
2. **EditHistory**：命令事务、undo/redo、编辑前后 selection、连续输入合并及显式 barrier。
3. **EditorSession**：单 selection 起步，保存 anchor、active、preferred horizontal x、鼠标拖选和 IME composition 状态。
4. **LayoutViewport**：某一 document revision 的不可变布局快照，统一提供 hit test、caret/range rect、scroll state 与延迟 `EnsureCaretVisible`。
5. **Adapters**：平台 IME 与 clangd/LSP 边界；只在边界转换 UTF-16，不直接修改渲染状态。

```text
key / IME
  → EditorSession command transaction
  → Document.apply(EditBatch) + selection update
  → revision++
  → queue EnsureCaretVisible
  → repaint / clangd didChange

layout
  → consume EnsureCaretVisible using actual caret rectangle

clangd
  → revision gate
  → UTF-16 range conversion
  → atomically replace diagnostics
  → repaint
```

## 必须保持的不变量

### Text 与事务

- 明确区分 `ByteOffset`、`Utf16Offset`、`LineColumnUtf16`、`VisualPoint`，不混用坐标空间。
- `EditBatch` 中所有 range 都引用同一编辑前 revision；先验证 UTF-8 边界、排序和不重叠，再原子应用。
- 最外层命令事务提交；嵌套事务合并；空事务丢弃；新编辑清空 redo。
- Undo/redo 同时恢复文本和 selection。
- 同一焦点与 selection 上的连续输入可以合并；切焦、粘贴、格式化、保存、补全和 IME commit 必须形成 barrier。

### Selection

- v1 只做一个 selection；多光标成为明确产品需求后再引入集合不变量。
- selection 保存 anchor 与 active，方向由两者推导。
- 垂直移动保留 preferred x；水平移动清除。
- 文本编辑与 selection 更新处于同一命令事务。

### IME

- 平台输入 range 是 UTF-16；Luma 文档内部始终使用 UTF-8 byte offset。
- `CompositionState` 保存 marked byte range 与事务 token；连续 marked update 属于同一 undo 事务。
- commit 替换 marked text、清理 composition 并结束事务；unmark、undo、切焦必须安全清理状态。
- 候选窗矩形和 point-to-index 必须共用最新布局快照与 scroll offset。
- v1 只支持一个 IME caret，不做多光标 fan-out。

### Viewport

- selection command 只产生 `EnsureCaretVisible` 请求；排版完成后使用真实 caret rect 调整 scroll。
- 同一几何路径服务 caret、selection、鼠标 hit test 和 IME candidate bounds。
- v1 关闭换行时 visual row 可等于 logical row；不要提前复制完整 DisplayMap。

### Diagnostics

- clangd diagnostics 是派生状态，不进入 undo history。
- 每次编辑先隐藏旧 diagnostics；只接受与当前 document revision 对应的发布结果。
- 每个 document/source 整体替换 diagnostics；空发布即清空。
- 零宽 range 扩展一个 code point；重叠时按 Error、Warning、Info、Hint 取最高严重度。

## 明确不做

当前 Stage 1 切片不实现 CRDT、Lamport clock、tombstone、协作 undo、MultiBuffer/excerpt、完整 DisplayMap 管线、历史 LSP snapshot rebase、多服务端诊断合并、内联诊断块或 scrollbar marker。

`TextInput` 现在只服务文件搜索；源码区由独立 `EditorController` 管理。下一切片只在现有行为测试和真实平台证据要求下增加保存/文件模型或语法能力，不为未来需求预建抽象。
