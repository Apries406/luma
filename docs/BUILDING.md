# Building Luma

## 固定版本

- Zig `0.16.0`，不接受相近版本。
- Rust `1.97.1`，由仓库根目录 `rust-toolchain.toml` 固定。
- GPUI `0.2.2` 与 `gpui_platform` `0.1.0`，来自 Zed 完整提交 `251854020a9dbea1a388bf392c9bd04fd136a557`。

这里的 `0.2.2` 是该 Git 提交中 crate 清单声明的版本，不代表使用 crates.io 的同名归档。crates.io `gpui 0.2.2` 的校验和为 `979b45cfa6ec723b6f42330915a1b3769b930d02b2d505f9697f8ca602bee707`，其 VCS 元数据指向带 dirty 状态的提交 `69e2130295c2649963eb639fc70b4f2ee8ea1624`；它与本仓库锁定的 Git 源不是同一源码身份。锁定提交 `251854…` 是因为它包含当前 `gpui_platform` Windows 后端、AccessKit API 与官方 Accessibility 示例。`Cargo.lock` 应保留 `git+https://github.com/zed-industries/zed?...#251854…` 来源；依赖不得改为 branch、`*`、浮动版本或 crates.io 同名包。

## macOS Apple Silicon

要求 Xcode、macOS SDK 与 Metal Toolchain。仅安装 Xcode 主程序仍可能在 GPUI shader 构建时报 `cannot execute tool 'metal' due to missing Metal Toolchain`；遇到该错误时执行：

```sh
xcodebuild -downloadComponent MetalToolchain
xcrun metal --version
```

验证工具链：

```sh
zig version
rustc --version
cargo --version
xcodebuild -version
```

期望前三项分别以 `0.16.0`、`rustc 1.97.1`、`cargo 1.97.1` 开头。

本次会话的 PATH 受运行环境隔离，仓库内安装了仅用于验证的 `.tools/`（已忽略）。使用：

```sh
export RUSTUP_HOME="$PWD/.tools/rustup-home"
export CARGO_HOME="$PWD/.tools/cargo-home"
export PATH="$PWD/.tools/cargo-home/bin:$PWD/.tools/zig-0.16.0:$PATH"
```

正常开发机可使用系统安装的精确版本，不要求 `.tools/`。

构建和检查：

```sh
zig version
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --locked
cargo build --workspace --release --locked
cargo run -p luma-ui --locked
./scripts/package-macos.sh release
```

脚本生成 `dist/Luma.app`，在 bundle 完成后做 ad-hoc 重签并执行严格 `codesign` 验证。这只保证本机开发 bundle 自洽；Developer ID 签名、公证和 Gatekeeper 分发不属于阶段 0，`spctl` 拒绝该 ad-hoc bundle 是预期结果。

## Windows x86_64

安装 Visual Studio 2022 Build Tools（Desktop development with C++ 与 Windows SDK）、Rust 1.97.1 MSVC 工具链及 Zig 0.16.0。锁定 GPUI 的 Windows Release shader 构建仍调用 legacy `fxc.exe`；构建前确认 Windows SDK 提供它，或将 `GPUI_FXC_PATH` 指向有效的 `fxc.exe`。资源编译还需要 `llvm-rc` 或等效 MSVC/LLVM 工具链。使用 PowerShell：

```powershell
zig version
rustc --version
cargo --version
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --release --locked
```

Windows 必须在 Windows 主机或 CI 上构建；macOS 本机构建不能替代 Windows GPUI 验证。人工项目见 [`WINDOWS_TEST_CHECKLIST.md`](WINDOWS_TEST_CHECKLIST.md)。
