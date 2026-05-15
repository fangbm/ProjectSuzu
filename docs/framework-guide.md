# Project Suzu 框架使用指南

本文是一份面向开发者的完整使用指南，目标是把 Project Suzu 从“能跑示例”串到“能搭一个自己的视觉小说项目”。它覆盖项目结构、脚本、资源、运行时 API、存档、标题界面、工具链、XP3 边界、测试和发布流程。

Project Suzu 仍处于 `0.1.x` 阶段。运行时、脚本格式和工具链已经能支撑小型项目和实验性迁移工作，但 API 仍可能在后续版本中收敛。需要兼容性判断时，以 `docs/api-stability.md` 为准。

## 适用场景

适合使用 Project Suzu 的场景：

- 编写原创视觉小说或 galgame 原型。
- 用 Rust 集成一个可测试、可嵌入的视觉小说运行时。
- 编写 `.szs` 脚本并用内置编译器验证。
- 打包图片、脚本、音频等资源到 `.suzupack`。
- 读取明文 XP3 资源，做资产预览或授权迁移实验。
- 使用外部 XP3 processor 接口处理你有权处理的资源。

不适合把 Project Suzu 当作：

- 完整 KRKR/TJS/KAG 引擎替代品。
- 商业游戏解包、破解或 DRM 绕过工具。
- 内置游戏专用 XP3 处理器集合。
- 稳定到 1.0 级别的长期公共 API。

## 核心概念

Project Suzu 的框架模型可以分成几层：

- `suzu-script`：解析 `.szs` 文本，生成命令队列。
- `suzu-app`：高层视觉小说运行时 facade，核心类型是 `SuzuApp`。
- `suzu-render`：保留式渲染层、动画补间、后处理和 shader 配置。
- `suzu-text`：文本 markup、打字机显示、等待点、ruby 数据和语音同步。
- `suzu-audio`：BGM/voice/se 状态、淡入淡出和后端命令同步。
- `suzu-asset`：图片加载、manifest、`.suzupack`、XP3 archive 读取。
- `suzu-save`：可序列化游戏状态、存档槽、自动存档和缩略图。
- `suzu-input`：键盘、鼠标、滚轮和选择操作映射。
- `suzu-platform`：桌面窗口、GPU 渲染和平台配置。

典型运行流：

1. 读取或内嵌 `.szs` 脚本。
2. 用 `SuzuApp::load_script` 或 `SuzuApp::load_script_asset` 加载脚本。
3. 注册图片、脚本、音频等资源。
4. 进入桌面事件循环。
5. 每帧输入事件进入 `SuzuApp`。
6. `SuzuApp` 推进脚本、动画、文本、音频和 UI 状态。
7. 平台层把 `DesktopFrame` 渲染到窗口。

## 环境准备

需要 Rust 工具链。推荐使用 stable toolchain：

```powershell
rustup default stable
rustup component add rustfmt clippy
```

克隆并验证仓库：

```powershell
git clone https://github.com/fangbm/ProjectSuzu.git
cd ProjectSuzu
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

运行最小示例：

```powershell
cargo run -p suzu-hello-world
```

## 推荐项目结构

如果你在 Project Suzu workspace 内新增一个项目，可以参考：

```text
examples/my-novel/
  Cargo.toml
  build.rs
  script/
    main.szs
  assets/
    bg/
      school.png
    char/
      eileen.png
    voice/
      eileen_001.ogg
  src/
    main.rs
    error_dialog.rs
```

如果你在仓库外做独立项目，建议结构类似：

```text
my-novel/
  Cargo.toml
  script/
    main.szs
  assets/
    bg/
    char/
    voice/
  src/
    main.rs
```

内部示例项目可以直接依赖 workspace crates。仓库外项目则需要通过 git dependency 或发布后的 crate 依赖方式接入。当前仓库主要面向 workspace 内开发。

## 最小 Cargo 项目

一个最小桌面项目需要依赖 `suzu-app` 和 `suzu-platform`：

```toml
[package]
name = "my-novel"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
suzu-app = { path = "../../crates/suzu-app" }
suzu-platform = { path = "../../crates/suzu-platform" }
```

如果在 Project Suzu workspace 内，可以直接使用 workspace dependencies 的写法，参考 `examples/hello-world/Cargo.toml`。

## 最小运行入口

最小 `src/main.rs`：

```rust
use anyhow::Result;
use suzu_app::{GameConfig, SuzuApp, TitleScreenConfig};
use suzu_platform::{run_desktop, WindowConfig};

fn main() -> Result<()> {
    let mut app = SuzuApp::new(game_config());
    app.register_textures_from_dir("examples/my-novel/assets")?;
    app.load_script(include_str!("../script/main.szs"))?;

    run_desktop(WindowConfig::default(), app)
}

fn game_config() -> GameConfig {
    GameConfig {
        title_screen: TitleScreenConfig {
            enabled: true,
            title: "My Novel".to_owned(),
            subtitle: "Project Suzu".to_owned(),
        },
        ..GameConfig::default()
    }
}
```

关键点：

- `SuzuApp::new` 创建运行时。
- `register_textures_from_dir` 扫描图片资源。
- `load_script` 编译脚本字符串。
- `run_desktop` 创建窗口并开始渲染。

Windows GUI 程序可以加：

```rust
#![cfg_attr(windows, windows_subsystem = "windows")]
```

如果这样做，启动失败时不会自动出现控制台，建议像示例一样加 `error_dialog.rs` 显示错误。

## 窗口配置

`WindowConfig` 控制桌面窗口：

```rust
use suzu_core::Vec2;
use suzu_platform::WindowConfig;

let window = WindowConfig {
    title: "My Novel".to_owned(),
    logical_size: Vec2::new(1280.0, 720.0),
    resizable: true,
};
```

`GameConfig` 里的 `window` 字段也可序列化，但当前示例通常直接把 `WindowConfig` 传给 `run_desktop`。

## 标题界面

启用标题界面：

```rust
GameConfig {
    title_screen: TitleScreenConfig {
        enabled: true,
        title: "My Novel".to_owned(),
        subtitle: "Chapter 1".to_owned(),
    },
    ..GameConfig::default()
}
```

内置标题菜单包含：

- Start：从脚本开头开始。
- Continue：优先恢复 autosave，其次恢复 slot 0。
- Load：恢复 slot 0。
- Settings：进入设置入口。
- Quit：请求退出。

运行时系统菜单还支持返回标题。标题界面适合示例和普通桌面游戏；如果你想自己实现 launcher 或主菜单，可以保持 `title_screen.enabled = false`。

## 编写脚本

Project Suzu 脚本使用 `.szs` 格式。最小脚本：

```text
@script version=1
@bg file="bg_school_evening" time=500 method=crossfade
# 铃
你好，欢迎来到 Project Suzu。[l][r]
```

常见语法：

- 以 `;` 开头的是注释。
- `@command key=value` 是命令。
- `# 角色名` 设置下一段对白的说话人。
- 普通文本行会变成对白。
- `*label` 定义跳转标签。
- `[l]` 是等待点击点。
- `[r]` 是换行控制标签。

完整命令参考见 `docs/scripting-reference.md`。

## 场景和角色命令

背景：

```text
@bg file="bg_school_evening" time=800 method=crossfade
```

角色：

```text
@char name="eileen" face="happy" x=460 y=0 width=360 height=720 flip=false layer=10
```

隐藏角色：

```text
@hidechar name="eileen"
```

角色贴图 id 的约定由运行时决定。当前内置逻辑会根据 `name` 和 `face` 组合选择角色 texture id；如果脚本写 `name="eileen" face="happy"`，通常需要注册对应的角色资源。

## 动画和视觉效果

移动：

```text
@anim target="eileen" type=move_to x=500 y=40 duration=900
```

缩放：

```text
@anim target="eileen" type=zoom scale=1.2 duration=500
```

淡出：

```text
@anim target="eileen" type=fade opacity=0 duration=500
```

闪白：

```text
@fx type=flash color=#FFFFFF duration=250
```

震动：

```text
@fx type=quake intensity=8 duration=500
```

动画和效果按帧推进，不需要脚本手动更新。需要暂停脚本时使用 `@wait`。

## 流程控制

标签和跳转：

```text
*start
# 铃
开始。
@jump goto=end

*end
# 铃
结束。
```

调用和返回：

```text
@call goto=common
# 铃
回到主线。

*common
# 旁白
共通段落。
@return
```

选择项：

```text
@choice "去教室" goto=classroom
@choice "去天台" goto=roof

*classroom
# 铃
教室很安静。
@jump goto=end

*roof
# 铃
天台风很大。

*end
```

变量和条件：

```text
@set name=affection_eileen value=52
@choice "艾琳路线" goto=eileen cond=affection_eileen>=50
```

条件块：

```text
@if cond=affection_eileen>=50
# 艾琳
谢谢你。
@else
# 艾琳
下次再说吧。
@endif
```

## 音频和语音

BGM：

```text
@playbgm file="bgm_school" loop=true fadein=2000
@stopbgm fadeout=1000
```

语音：

```text
@voice file="voice_eileen_001"
# 艾琳
这一句会关联语音。
```

当前公开框架主要保存和同步音频状态；实际音频播放后端可由平台或应用集成。脚本层的语音 cue 会进入运行时状态，并用于历史记录的语音回放钩子。

## 资源加载

最简单的图片加载方式：

```rust
app.register_textures_from_dir("examples/my-novel/assets")?;
```

这会递归发现常见图片文件，并按文件 stem 注册 texture id。例如：

```text
assets/bg_school_evening.png -> bg_school_evening
assets/char/eileen.png      -> eileen
```

也可以手动注册单个 texture：

```rust
app.register_texture("bg_school_evening", "assets/bg_school_evening.png");
```

脚本引用：

```text
@bg file="bg_school_evening"
@char name="eileen"
```

## Manifest 和 `.suzupack`

生成 JSON manifest：

```powershell
cargo run -p suzu-packer -- examples\hello-world --output target\hello-world-assets.json
```

生成 `.suzupack`：

```powershell
cargo run -p suzu-packer -- examples\hello-world --pack target\hello-world.suzupack
```

运行时注册 manifest：

```rust
app.register_asset_manifest_file("target/hello-world-assets.json")?;
```

运行时注册 package archive：

```rust
app.assets.register_package_file("target/hello-world.suzupack")?;
```

`.suzupack` 适合 release 包携带资源。它包含 manifest、packed offset、packed size、checksum 和可选 RLE 压缩元数据。

## XP3 资源

明文 XP3 可以直接注册：

```rust
app.register_xp3_file("data.xp3")?;
app.load_script_asset("main")?;
```

XP3 entry 会按文件 stem 暴露为 asset id。例如：

```text
scenario/main.szs -> main
image/bg_school.png -> bg_school
```

外部 XP3 processor：

```rust
use suzu_asset::Xp3PluginModule;

let module = Xp3PluginModule::from_json_file("xp3-plugin.json")?;
app.register_xp3_file_with_options("data.xp3", module.xp3_options())?;
```

外部 processor 只适用于你拥有权利或明确授权处理的资源。公开仓库不提供游戏专用 processor、密钥或规避访问控制的说明。接口细节见 `docs/xp3-plugin-interface.md`，支持边界见 `docs/xp3-support.md` 和 `LEGAL.md`。

## 存档和恢复

捕获当前状态：

```rust
let state = app.capture_state();
```

恢复状态：

```rust
app.restore_state(state);
```

保存到 slot：

```rust
app.save_slot(0);
```

从 slot 读取：

```rust
app.load_slot(0);
```

存档包含：

- 脚本位置和 pending commands。
- 当前 scene。
- BGM/voice 状态。
- 变量。
- 历史记录。
- 已读对白 key。
- 可选 RGBA thumbnail。

如果要落盘，可以使用 `suzu-save` 的 JSON 读写工具，或围绕 `GameState` 自己做存储。

## 用户设置

`UserSettings` 包含音量、文本速度和窗口偏好：

```rust
use suzu_app::UserSettings;

let mut settings = UserSettings::default();
settings.audio.master_volume = 0.8;
settings.text.speed_chars_per_second = 90.0;
app.apply_user_settings(settings);
```

读写 JSON：

```rust
let settings = UserSettings::from_json_file("settings.json")?;
settings.write_json_file("settings.json")?;
```

## 输入和 UI 操作

桌面平台将常见输入转换为 `DesktopInputEvent`：

- Confirm：确认、推进对白、确认选择。
- Cancel：打开或关闭系统菜单。
- MoveSelection：上下移动选择。
- Scroll：滚动选择或历史记录。

应用内可以直接调用：

```rust
app.confirm();
app.reveal_dialogue_now();
app.set_auto_mode(true);
app.set_skip_mode(true);
```

自动模式会在对白完全显示后等待设置的 delay，再自动确认。已读快进只会跳过已经读过的对白，并会在未读对白、选择项或等待命令处停止。

## 桌面渲染集成

最常用路径是直接调用：

```rust
run_desktop(WindowConfig::default(), app)
```

如果你要写自己的平台层，需要实现 `DesktopApp`：

```rust
use suzu_platform::{DesktopApp, DesktopFrame, DesktopInputEvent};

impl DesktopApp for MyApp {
    fn input(&mut self, event: DesktopInputEvent) {
        // translate input into runtime actions
    }

    fn update(&mut self, delta_ms: u32) -> DesktopFrame {
        // advance runtime and return render frame
        DesktopFrame::default()
    }
}
```

`SuzuApp` 已经实现了桌面运行所需的 app behavior，所以普通项目不需要手写这一层。

## 可视化编辑器

启动编辑器：

```powershell
cargo run -p suzu-editor
```

headless 检查：

```powershell
cargo run -p suzu-editor -- --check --project-root examples\hello-world
```

当前编辑器是 MVP：

- 扫描 Project Suzu 项目。
- 打开 `.szs` 文件。
- 导入为可视化节点。
- 编辑常见节点字段。
- 导出回 `.szs`。
- 运行图诊断和编译诊断。

编辑器详细计划见 `docs/visual-script-editor-development-plan.md`。

## Launcher 和 XP3 Viewer

启动统一 launcher：

```powershell
cargo run -p suzu-launcher
```

headless 检查：

```powershell
cargo run -p suzu-launcher -- --check
```

启动 XP3 viewer：

```powershell
cargo run -p suzu-xp3-viewer -- D:\game\data.xp3
```

headless 检查：

```powershell
cargo run -p suzu-xp3-viewer -- --check
```

这些工具适合做资源检查、明文 XP3 预览、KRKR package scan 和有限 KAG 转换实验。不要把它们理解为完整 KRKR 运行器。

## 打包发布

本地 package 输入检查：

```powershell
.\scripts\package-desktop.ps1 -Check
```

生成本地 desktop package：

```powershell
.\scripts\package-desktop.ps1
```

默认输出：

```text
dist/project-suzu-desktop.zip
```

发布包会包含：

- 示例和工具二进制。
- README、CHANGELOG、LICENSE。
- `LEGAL.md`、`SECURITY.md`。
- `THIRD_PARTY_LICENSES.md`。
- 核心 docs。
- branding README 和 icon。
- hello-world `.suzupack` 和 manifest。

GitHub Release 通过 `v*` tag 触发。创建 tag 前先跑完整 gate。

## 验证清单

日常开发建议：

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

发布前建议：

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
cargo test -p suzu-script --features lua
cargo run -p suzu-launcher -- --check
cargo run -p suzu-xp3-viewer -- --check
cargo run -p suzu-editor -- --check
.\scripts\package-desktop.ps1 -Check
```

依赖变化后重新生成第三方许可证：

```powershell
cargo about generate about-markdown.hbs --workspace --locked --fail -o THIRD_PARTY_LICENSES.md
```

## 常见问题

脚本编译失败：

- 用 `cargo run -p suzu-compiler -- path\to\main.szs` 单独检查。
- 查看错误里的 line/column。
- 检查命令拼写，未知命令会给相近建议。

背景或角色不显示：

- 确认资源文件扩展名是支持的图片格式。
- 确认 `register_textures_from_dir` 指向正确目录。
- 确认脚本里的 `file` 或角色 texture id 和注册 id 一致。

选择项不出现：

- 确认条件变量已经设置。
- 确认 `cond` 表达式能被当前编译器支持。
- 确认脚本没有提前跳走。

存档恢复位置不对：

- 在等待点存档更容易推理。
- 确认恢复的是同一个脚本版本。
- 如果脚本结构改动很大，旧存档里的 pending commands 可能不适合继续使用。

XP3 读不到内容：

- 确认 archive 是明文或你提供了授权的外部 processor。
- 确认 entry 类型受支持。
- 用 `suzu-xp3-viewer` 查看 entry 是否被索引。
- 不要把完整 KRKR 兼容性问题简化为 XP3 读取问题。

GUI `--check` 失败：

- 确认路径存在。
- 如果使用 `--xp3-plugin`，必须同时传 `--i-have-rights-to-process-these-assets`。
- 查看 stderr 中的 anyhow 错误链。

## 迁移和兼容建议

- 以 `SuzuApp` facade 为主要集成点。
- 优先使用公开方法注册资源、加载脚本、捕获/恢复状态。
- 不要依赖 app 内部模块路径。
- `.szs` 文件显式写 `@script version=1`。
- `.suzupack` 当前格式版本是 `1`。
- XP3 plugin module 当前格式是 `suzu.xp3-plugin.v1`。
- pre-1.0 阶段每次升级前阅读 `CHANGELOG.md` 和 `docs/api-stability.md`。

## 下一步阅读

- `docs/scripting-reference.md`：脚本命令完整参考。
- `docs/user-guide.md`：简短入口指南。
- `docs/xp3-support.md`：XP3 支持边界。
- `docs/xp3-plugin-interface.md`：外部 XP3 processor 接口。
- `docs/developer-checks.md`：开发检查命令。
- `docs/release-packaging.md`：发布打包流程。
