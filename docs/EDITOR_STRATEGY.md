# Luma 编辑器实现策略

## 目标与边界

Luma 借鉴 Zed 编辑器的分层、事务边界和行为不变量，但独立实现源码，不兼容 `.zed/` 配置，也不依赖 Zed 的 `editor`、`text`、`language`、`multi_buffer` 等内部 crate。Lec 已确认选择独立实现最小编辑器，不采用 GPL Zed editor 直接集成路线。

固定 Zed 提交 `251854020a9dbea1a388bf392c9bd04fd136a557` 中，GPUI 与 `gpui_platform` 标记为 Apache-2.0；Zed editor crate 标记为 GPL-3.0-or-later。因此：

- 可以通过公开 API 使用 GPUI，并在遵守 Apache-2.0 的前提下改编 GPUI 示例。
- 可以研究 Zed editor 的行为、接口与架构思想。
- 不复制、逐行翻译或改编 GPL editor 源码、独特注释、命名和内部数据结构。
- 独立实现应保留设计说明与测试作为来源和行为证据。

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
- 普通连续输入可以短时间合并；粘贴、格式化、保存、补全和 IME commit 必须形成 barrier。

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

阶段 1 前不实现 CRDT、Lamport clock、tombstone、协作 undo、MultiBuffer/excerpt、完整 DisplayMap 管线、历史 LSP snapshot rebase、多服务端诊断合并、内联诊断块或 scrollbar marker。

当前 `TextInput` 仍只是 Stage 0 的 GPUI 输入验证原型。它不应继续膨胀为生产编辑器；下一阶段从上述 `Document` 和事务边界开始，以行为测试驱动独立实现。
