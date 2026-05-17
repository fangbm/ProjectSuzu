# Project Suzu

[![CI](https://github.com/fangbm/ProjectSuzu/actions/workflows/ci.yml/badge.svg)](https://github.com/fangbm/ProjectSuzu/actions/workflows/ci.yml)

![Project Suzu 图标](assets/branding/Suzu_icon.png)

Project Suzu 是一个用 Rust 编写的视觉小说/galgame 框架。它包含脚本编译、运行时状态、渲染层描述、文本显示、语音同步、音频状态、资源打包、存档、输入映射、桌面平台集成、示例工程和发布工作流。

## 当前能力

- `.szs` 脚本解析与编译：对白、角色、背景、选择项、变量、条件、跳转、调用、等待、存档命令。
- 桌面窗口与渲染：基于 `winit` 和 `wgpu`，支持精灵层、混合模式、背景转场、动画、视觉效果和文本纹理。
- 文本系统：打字机显示、点击等待、换行标签、ruby 注音数据、竖排布局、历史记录和语音回放钩子。
- 音频系统：BGM/语音通道状态、淡入淡出、存档快照、可插拔 `AudioBackend` 命令同步。
- 资源系统：PNG/JPEG/WebP 发现、异步加载、LRU 缓存、JSON manifest、`.suzupack` 资源包、校验和。
- 系统功能：存档/读档、自动存档、缩略图、系统菜单、自动模式、已读快进、配置持久化。
- 标题界面：启动后可显示开始菜单，支持开始游戏、继续游戏、读取存档、设置入口和退出。
- 零代码工程入口：`suzu-player` 可直接打开带 `game.suzu.toml` 的 KRKR 风格工程目录。
- 平台与发布：桌面示例、Web canvas 壳、移动端目标描述、CI、Release workflow、本地打包脚本。

## 功能状态

| 功能 | 状态 |
| --- | --- |
| `.szs` 解析与编译 | 相对稳定 |
| 运行时 app facade | 实验中 |
| 存档快照 | 实验中 |
| 可视化剧本编辑器 | 预览 |
| 明文 XP3 资源加载 | 预览 |
| XP3 Viewer 与 KRKR 包扫描 | 预览 |
| KRKR 包扫描模式 | 有限预览 |

公开仓库只提供明文 XP3 读取和外部 XP3 插件接口。项目方可以为自己有权处理的资源包提供外部插件，但游戏专用处理器不放入本仓库。支持边界见：[docs/xp3-support.md](docs/xp3-support.md)、[LEGAL.md](LEGAL.md) 和 [SECURITY.md](SECURITY.md)。

## 下载使用

打开 GitHub Release：

[https://github.com/fangbm/ProjectSuzu/releases/tag/v0.2.0](https://github.com/fangbm/ProjectSuzu/releases/tag/v0.2.0)

按系统下载：

- Windows: `project-suzu-v0.2.0-windows-x64.tar.gz`
- Linux: `project-suzu-v0.2.0-linux-x64.tar.gz`

解压后可以看到：

- `suzu-hello-world`
- `suzu-branching-story`
- `suzu-ui-save-load-demo`
- `suzu-short-vn-demo`
- `suzu-compiler`
- `suzu-packer`
- `suzu-bench`
- `suzu-player`
- `suzu-editor`

Windows 包中对应文件为 `.exe`。
三个示例程序在 Windows 下使用 GUI 子系统，双击启动时不会额外弹出控制台窗口；如果启动失败，会显示错误弹窗。命令行工具 `suzu-compiler`、`suzu-packer`、`suzu-bench` 仍保留终端输出；双击 `suzu-compiler` 或 `suzu-packer` 会显示用法并等待按 Enter，双击 `suzu-bench` 会运行默认基准并等待按 Enter。带参数使用时建议从终端运行。
示例程序启动后会先进入标题界面；使用方向键选择，按 Enter 或 Space 确认。

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
cargo run -p suzu-short-vn-demo
cargo run -p suzu-player -- templates\krkr-like-vn
cargo run -p suzu-editor
start examples\web-browser-shell\index.html
```

`suzu-player` 是面向作者的低门槛入口。标准工程只需要 `game.suzu.toml`、`scenario/main.szs` 和 `assets/`；不需要编写 Rust `main.rs`。目录规范见：[docs/project-layout.zh-CN.md](docs/project-layout.zh-CN.md)。

桌面示例默认启用标题界面。框架层可通过 `GameConfig.title_screen.enabled = true` 打开标题入口，并通过 `TitleScreenConfig` 设置标题和副标题；如果保持默认值，`SuzuApp` 会像普通运行时一样直接进入脚本，方便嵌入到自定义启动流程中。
可视化剧本编辑器可以通过 `cargo run -p suzu-editor` 启动；当前 MVP 支持扫描工程、打开 `.szs`、编辑常见节点、导出保存和刷新诊断。

XP3/KRKR 测试工具支持加载外部 XP3 插件模块。在 `suzu-launcher` 或 `suzu-xp3-viewer` 的 `XP3 plugin` 输入框填入模块路径即可；命令行转换可使用：

```powershell
cargo run -p suzu-launcher -- --krkr2suzu "D:\game" "D:\out" --xp3-plugin D:\plugins\xp3-plugin.json --i-have-rights-to-process-these-assets
```

明文 XP3 支持范围见：[docs/xp3-support.md](docs/xp3-support.md)。外部 XP3 处理器接口说明见：[docs/xp3-plugin-interface.md](docs/xp3-plugin-interface.md)。

快速上手见：[docs/getting-started.zh-CN.md](docs/getting-started.zh-CN.md)。完整框架使用指南见：[docs/framework-guide.zh-CN.md](docs/framework-guide.zh-CN.md)。工程目录规范见：[docs/project-layout.zh-CN.md](docs/project-layout.zh-CN.md)。低门槛模板见：[templates/krkr-like-vn/README.zh-CN.md](templates/krkr-like-vn/README.zh-CN.md)，Rust 集成模板见：[templates/minimal-vn/README.zh-CN.md](templates/minimal-vn/README.zh-CN.md)。短篇 VN demo 计划见：[docs/short-vn-demo.zh-CN.md](docs/short-vn-demo.zh-CN.md)。API 稳定性说明见：[docs/api-stability.zh-CN.md](docs/api-stability.zh-CN.md)。脚本参考见：[docs/scripting-reference.zh-CN.md](docs/scripting-reference.zh-CN.md)。XP3 插件接口见：[docs/xp3-plugin-interface.zh-CN.md](docs/xp3-plugin-interface.zh-CN.md)。第三方依赖许可证原文见：[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md)，中文导读见：[THIRD_PARTY_LICENSES.zh-CN.md](THIRD_PARTY_LICENSES.zh-CN.md)。品牌素材边界见：[assets/branding/README.zh-CN.md](assets/branding/README.zh-CN.md)。

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

[docs/scripting-reference.zh-CN.md](docs/scripting-reference.zh-CN.md)

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

[docs/developer-checks.zh-CN.md](docs/developer-checks.zh-CN.md)

可视化剧本编辑器开发文档见：

[docs/visual-script-editor-development-plan.zh-CN.md](docs/visual-script-editor-development-plan.zh-CN.md)

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
- `tools/suzu-editor`: 可视化剧本编辑器桌面入口。
- `examples/`: 示例工程、压力脚本和 Web 壳。
- `docs/`: 用户、脚本、发布、开发文档。

## 文档

- [docs/user-guide.zh-CN.md](docs/user-guide.zh-CN.md)
- [docs/scripting-reference.zh-CN.md](docs/scripting-reference.zh-CN.md)
- [docs/release-packaging.zh-CN.md](docs/release-packaging.zh-CN.md)
- [docs/developer-checks.zh-CN.md](docs/developer-checks.zh-CN.md)
- [docs/release-checklist.zh-CN.md](docs/release-checklist.zh-CN.md)

## 许可证

本项目采用双许可证：

```text
MIT OR Apache-2.0
```

详见 `LICENSE-MIT` 和 `LICENSE-APACHE`。
