# Windows 10/11 x86_64 人工验收清单

> 当前 macOS 主机不能声称 Windows 实机通过。CI 只覆盖编译和自动测试。

## 环境

- [ ] Windows 10 最新维护版本
- [ ] Windows 11 最新维护版本
- [ ] x86_64 真机或 VM（记录 GPU 与驱动）
- [ ] `zig version` 输出精确 `0.16.0`
- [ ] `rustc --version` 输出 `1.97.1`

## 阶段 0

- [ ] Debug 与 Release 构建可重复
- [ ] 创建窗口；深色背景和中文文件名 `你好.cpp` 正确显示
- [ ] 按钮点击与 Ctrl+Enter 键盘动作
- [ ] 单行文本输入、光标、Shift 选择、鼠标拖选
- [ ] 多行输入、换行、caret-following 垂直和水平滚动
- [ ] 微软拼音 IME：组合文本、候选窗位置、提交、取消，以及组合中 Tab/Shift+Tab/鼠标切换输入框
- [ ] 中文与 Emoji 复制、剪切、粘贴
- [ ] Tab/Shift+Tab 焦点切换不破坏 IME 组合状态
- [ ] 可见命令入口：产品确认 Windows-only client-side `Run  Ctrl+Enter` 可替代菜单；鼠标、focused Enter、focused Space 与 Ctrl+Enter 各执行一次；Narrator 读为“Run current file”按钮。该控件不是 HMENU，仅调用 `set_menus` 也不算通过
- [ ] 文件拖放（阶段 2 前可记为未实现）
- [ ] 150% DPI：文字、点击区域、光标、选择、滚动
- [ ] 200% DPI：文字、点击区域、光标、选择、滚动
- [ ] Narrator 能读出应用名、按钮名、状态和输入框
- [ ] 最小 Release 打包可启动
- [ ] 记录 EXE、资源、包总大小和最大五项

## 证据

记录 Windows 版本、设备、日期、操作者、构建提交、每项结果、截图和复现步骤。未执行项必须标记“未验证”，不能留空后宣称通过。
