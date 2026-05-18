# Project Suzu 快速上手

这份文档面向第一次打开 Project Suzu 的开发者，目标是在一个短流程里跑通：安装工具链、运行示例、用零代码工程启动剧情、编译脚本、打包资源和执行本地自检。更完整的框架说明见 `docs/framework-guide.md`。

Project Suzu 目前仍是 `0.1.x` 阶段。它适合原创视觉小说原型、脚本工具链实验、授权资源迁移验证，以及 Rust 应用内嵌视觉小说运行时；它不是完整 KRKR/TJS/KAG 替代品，也不内置商业游戏专用 XP3 处理器。

## 1. 准备 Rust

安装 stable toolchain，并确认 `rustfmt`、`clippy` 可用：

```powershell
rustup default stable
rustup component add rustfmt clippy
rustc --version
cargo --version
```

如果是在 Windows 上运行桌面示例，建议从 PowerShell 或 Windows Terminal 启动命令，便于看到工具输出。

## 2. 克隆并验证仓库

```powershell
git clone https://github.com/fangbm/ProjectSuzu.git
cd ProjectSuzu
cargo fmt --all -- --check
cargo test --workspace
```

更完整的质量门禁是：

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
cargo test -p suzu-script --features lua
```

## 3. 运行内置示例

最小示例：

```powershell
cargo run -p suzu-hello-world
```

带分支选择的示例：

```powershell
cargo run -p suzu-branching-story
```

带标题界面、系统菜单、存档读档和设置入口的示例：

```powershell
cargo run -p suzu-ui-save-load-demo
```

短篇 VN 闭环示例：

```powershell
cargo run -p suzu-short-vn-demo
```

这些示例默认使用内置脚本和 fallback texture。即使没有外部图片资源，也能看到可运行画面。

## 4. 用零代码模板开始

如果你只想写脚本和放资源，优先从脚本优先模板开始：

```text
templates/starter-vn/
  game.suzu.toml
  assets/
  scenario/main.szs
```

直接运行：

```powershell
cargo run -p suzu-player -- templates\starter-vn
```

只做检查、不打开窗口：

```powershell
cargo run -p suzu-player -- --check templates\starter-vn
```

这个模板不需要 `src/main.rs`。`game.suzu.toml` 描述标题、入口脚本、窗口和资源目录；`scenario/main.szs` 是默认剧情入口；`assets/` 放图片、音频、字体等资源。目录规范见 `docs/project-layout.md`。

## 5. Rust 集成模板

如果你要把 Suzu 嵌入自己的 Rust 应用，再使用最小 Rust 模板：

```text
templates/minimal-vn/
  Cargo.toml
  README.md
  assets/
  script/main.szs
  src/main.rs
```

直接运行模板：

```powershell
cargo run --manifest-path templates\minimal-vn\Cargo.toml
```

模板默认从 `script/main.szs` 读取脚本，从 `assets/` 扫描图片资源。如果 `assets/` 为空，运行时会注册两张 2x2 fallback texture，方便你先确认窗口、脚本和 UI 流程可用。

复制模板做新项目时，先改这些位置：

- `Cargo.toml` 的 `name`、`version` 和依赖路径。
- `src/main.rs` 中的标题、副标题、fallback texture id。
- `script/main.szs` 中的角色、背景、标签和分支。
- `assets/` 中的图片、音频和 manifest。

模板位于仓库内部，因此默认使用相对路径依赖 `../../crates/suzu-app` 和 `../../crates/suzu-platform`。如果你把模板复制到仓库外，需要把这些依赖改成你自己的本地路径或后续发布的 crate/git 依赖。

## 6. 编写和检查脚本

脚本文件使用 `.szs`。新项目推荐 `syntax=indent`，最小结构可以从背景、角色、文本和选择开始：

```text
@script version=1 syntax=indent
bg bg_room
ch hero neutral
铃: 你好，Project Suzu。[l][r]
choice "继续" goto=label_continue

label label_continue:
铃: 脚本已经进入下一个标签。[l][r]
```

编译检查：

```powershell
cargo run -p suzu-compiler -- templates\starter-vn\scenario\main.szs
```

常用命令、文本 markup、变量和条件语法见 `docs/scripting-reference.md`。

## 7. 打包资源

把模板或示例资源导出为 JSON manifest：

```powershell
cargo run -p suzu-packer -- templates\minimal-vn --output target\minimal-vn-assets.json
```

导出为 `.suzupack`：

```powershell
cargo run -p suzu-packer -- templates\minimal-vn --pack target\minimal-vn.suzupack
```

应用侧可以通过 `SuzuApp::register_asset_manifest_file` 或 `SuzuApp::register_package_file` 注册这些产物。完整资源打包说明见 `docs/user-guide.md` 和 `docs/framework-guide.md`。

## 8. 使用 GUI 工具自检

三个 GUI 工具都支持 headless `--check`，不会打开窗口：

```powershell
cargo run -p suzu-launcher -- --check
cargo run -p suzu-player -- --check templates\starter-vn
cargo run -p suzu-xp3-viewer -- --check
cargo run -p suzu-editor -- --check
```

也可以指定工程或 XP3 文件：

```powershell
cargo run -p suzu-launcher -- --check --project-root templates\starter-vn
cargo run -p suzu-xp3-viewer -- --check --xp3 path\to\plain.xp3
cargo run -p suzu-editor -- --check --project-root templates\starter-vn
```

外部 XP3 plugin 只适用于你拥有或被授权处理的资源，并且必须显式传入授权确认：

```powershell
cargo run -p suzu-xp3-viewer -- --check --xp3 path\to\plain.xp3 --xp3-plugin path\to\plugin.json --i-have-rights-to-process-these-assets
```

XP3 支持边界见 `docs/xp3-support.md`，外部处理器接口见 `docs/xp3-plugin-interface.md`，法律和安全说明见 `LEGAL.md`、`SECURITY.md`。

## 9. 打包桌面发布

先检查发布输入：

```powershell
.\scripts\package-desktop.ps1 -Check
```

再生成本地包：

```powershell
.\scripts\package-desktop.ps1
```

发布包会包含桌面工具、示例二进制、hello-world 资源包、核心文档、许可证、第三方许可证和品牌说明。发布流程细节见 `docs/release-packaging.md` 和 `docs/release-checklist.md`。

## 下一步

- 读 `docs/project-layout.md`，了解零代码工程目录、`game.suzu.toml` 和资源 ID 规则。
- 读 `docs/framework-guide.md`，了解运行时 API、存档、标题界面、菜单和桌面平台层。
- 读 `docs/scripting-reference.md`，补齐 `.szs` 脚本语法。
- 读 `docs/api-stability.md`，确认哪些接口在 `0.1.x` 内会尽量保持兼容。
- 从 `templates/starter-vn` 复制一个项目，替换脚本和资源；需要 Rust 自定义入口时再参考 `templates/minimal-vn`。
