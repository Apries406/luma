# Third-Party Licenses

Luma 项目自身许可证尚待 Lec 团队确认。本文件只记录第三方组件，不替团队选择项目许可证。

## GPUI / gpui_platform

- 来源：Zed Industries 的 Zed 仓库，提交 `251854020a9dbea1a388bf392c9bd04fd136a557`
- 版本：`gpui 0.2.2`、`gpui_platform 0.1.0`
- 许可证：Apache License 2.0
- 上游：https://github.com/zed-industries/zed
- 许可证文本：[`LICENSES/Apache-2.0-Zed.txt`](LICENSES/Apache-2.0-Zed.txt)
- 上游许可证：https://github.com/zed-industries/zed/blob/251854020a9dbea1a388bf392c9bd04fd136a557/LICENSE-APACHE

## AccessKit macOS

- 来源：AccessKit 项目的 `accesskit_macos 0.26.3`
- 许可证：MIT OR Apache-2.0
- 上游：https://github.com/AccessKit/accesskit
- 用途：macOS 上将 `GPUIWindow` 的 accessibility focus 转发到 AccessKit content view；该版本已由 `Cargo.lock` 固定

## Zig Toolchain

Zig 0.16.0 是与 Luma 核心分开发行和统计体积的外部工具链。打包或再分发前必须依据 Zig 0.16.0 发布包中的 LICENSE 与第三方 notices 生成发行物清单。

## Rust 依赖

发行流程必须根据 `Cargo.lock` 生成完整许可证报告。阶段 0 的紧凑输入原型参考并改编了 Apache-2.0 的官方 GPUI `examples/input.rs`；没有引入 Zed 编辑器内部 crate 作为 Luma 业务层 API。正式发行时须随包携带适用的 Apache-2.0 许可文本与依赖 notices。
