# Project Suzu

![Project Suzu 图标](assets/branding/Suzu_icon.png)

Project Suzu 是一个用 Rust 编写的视觉小说/galgame 框架。它包含脚本编译、运行时状态、渲染层描述、文本显示、语音同步、音频状态、资源打包、存档、输入映射、桌面平台集成、示例工程和发布工作流。

## 当前能力

- `.szs` 脚本解析与编译：对白、角色、背景、选择项、变量、条件、跳转、调用、等待、存档命令。
- 桌面窗口与渲染：基于 `winit` 和 `wgpu`，支持精灵层、混合模式、背景转场、动画、视觉效果和文本纹理。
- 文本系统：打字机显示、点击等待、换行标签、ruby 注音数据、竖排布局、历史记录和语音回放钩子。
- 音频系统：BGM/语音通道状态、淡入淡出、存档快照、可插拔 `AudioBackend` 命令同步。
- 资源系统：PNG/JPEG/WebP 发现、异步加载、LRU 缓存、JSON manifest、`.suzupack` 资源包、校验和。
- 系统功能：存档/读档、自动存档、缩略图、系统菜单、自动模式、已读快进、配置持久化。
- 平台与发布：桌面示例、Web canvas 壳、移动端目标描述、CI、Release workflow、本地打包脚本。

## 下载使用

打开 GitHub Release：

[https://github.com/fangbm/ProjectSuzu/releases/tag/v0.1.0](https://github.com/fangbm/ProjectSuzu/releases/tag/v0.1.0)

按系统下载：

- Windows: `project-suzu-windows-x64.tar.gz`
- macOS: `project-suzu-macos-x64.tar.gz`
- Linux: `project-suzu-linux-x64.tar.gz`

解压后可以看到：

- `suzu-hello-world`
- `suzu-branching-story`
- `suzu-ui-save-load-demo`
- `suzu-compiler`
- `suzu-packer`
- `suzu-bench`

Windows 包中对应文件为 `.exe`。
三个示例程序在 Windows 下使用 GUI 子系统，双击启动时不会额外弹出控制台窗口；如果启动失败，会显示错误弹窗。命令行工具 `suzu-compiler`、`suzu-packer`、`suzu-bench` 仍保留终端输出。

## 从源码运行

```powershell
git clone https://github.com/fangbm/ProjectSuzu.git
cd ProjectSuzu
cargo run -p suzu-hello-world
```

其他示例：

```powershell
cargo run -p suzu-branching-story
cargo run -p suzu-ui-save-load-demo
start examples\web-browser-shell\index.html
```

性能 smoke test：

```powershell
cargo run -p suzu-bench -- 1000
```

## 编写脚本

脚本文件使用 `.szs` 格式：

```text
@script version=1
@bg file="bg_school_evening" method=crossfade time=500
# 铃
你好，欢迎来到 Project Suzu。[l][r]这里是第二行。

@choice "去屋顶" goto=roof
@choice "留在教室" goto=classroom

*roof
# 铃
屋顶的风很舒服。
```

常用命令包括：

- `@bg`: 切换背景。
- `@char`: 显示或更新角色。
- `@hidechar`: 隐藏角色。
- `@choice`: 创建选择项。
- `@if/@else/@endif`: 条件分支。
- `@set`: 设置变量。
- `@jump`, `@call`, `@return`: 流程控制。
- `@playbgm`, `@stopbgm`, `@voice`, `@playvoice`, `@stopvoice`: 音频和语音。
- `@wait`, `@hidemsg`, `@showmsg`, `@savename`, `@autosave`: 系统控制。

完整参考见：

[docs/scripting-reference.md](docs/scripting-reference.md)

## 编译脚本

```powershell
cargo run -p suzu-compiler -- examples\hello-world\script\main.szs
```

编译错误会包含行列信息；未知命令会给出相近命令建议。

## 打包资源

生成 JSON manifest：

```powershell
cargo run -p suzu-packer -- examples\hello-world --output target\hello-world-assets.json
```

生成 `.suzupack`：

```powershell
cargo run -p suzu-packer -- examples\hello-world --pack target\hello-world.suzupack
```

## 本地发布包

先检查发布输入：

```powershell
.\scripts\package-desktop.ps1 -Check
```

生成桌面发布包：

```powershell
.\scripts\package-desktop.ps1
```

输出位置：

```text
dist/project-suzu-desktop.zip
```

## 开发验证

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
cargo test -p suzu-script --features lua
```

开发检查清单见：

[docs/developer-checks.md](docs/developer-checks.md)

## 项目结构

- `crates/suzu-app`: 高层 app facade 和运行时状态。
- `crates/suzu-script`: 脚本 AST、解析器、编译器、VM 命令队列和扩展注册。
- `crates/suzu-render`: 渲染层、动画、转场、后处理配置和适配器边界。
- `crates/suzu-text`: 文本布局、打字机 reveal、ruby、竖排和语音同步。
- `crates/suzu-audio`: 音频通道状态、淡入淡出和后端命令同步。
- `crates/suzu-asset`: 资源发现、加载、缓存、manifest 和 `.suzupack`。
- `crates/suzu-save`: 存档、自动存档、缩略图和游戏状态。
- `crates/suzu-input`: 键盘、鼠标、滚轮、触摸输入映射。
- `crates/suzu-platform`: 桌面平台、移动/Web 目标描述。
- `tools/`: 编译器、资源打包器、benchmark CLI。
- `examples/`: 示例工程、压力脚本和 Web 壳。
- `docs/`: 用户、脚本、发布、开发文档。

## 文档

- [docs/user-guide.md](docs/user-guide.md)
- [docs/scripting-reference.md](docs/scripting-reference.md)
- [docs/release-packaging.md](docs/release-packaging.md)
- [docs/developer-checks.md](docs/developer-checks.md)
- [docs/release-checklist.md](docs/release-checklist.md)

## 许可证

本项目采用双许可证：

```text
MIT OR Apache-2.0
```

详见 `LICENSE-MIT` 和 `LICENSE-APACHE`。
