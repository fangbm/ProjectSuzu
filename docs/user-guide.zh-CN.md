# Project Suzu 用户指南

Project Suzu 是一个 Rust 视觉小说框架。它提供脚本编译器、保留式场景模型、文本 reveal 和历史记录、存档/读档状态、音频状态同步、资源打包和桌面示例。

第一次使用时请先看 `docs/getting-started.md`。完整端到端开发说明见 `docs/framework-guide.md`。

## 快速开始

运行 hello-world 示例：

```powershell
cargo run -p suzu-hello-world
```

运行最小项目模板：

```powershell
cargo run --manifest-path templates\minimal-vn\Cargo.toml
```

编译脚本：

```powershell
cargo run -p suzu-compiler -- examples\hello-world\script\main.szs
```

打包资源 archive：

```powershell
cargo run -p suzu-packer -- examples\hello-world --pack target\hello-world.suzupack
```

运行全部验证门禁：

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p suzu-script --features lua
.\scripts\package-desktop.ps1 -Check
```

启动可视化剧本编辑器 MVP：

```powershell
cargo run -p suzu-editor
```

启动统一项目入口工具：

```powershell
cargo run -p suzu-launcher
```

检查并预览 KiriKiri XP3 archive：

```powershell
cargo run -p suzu-xp3-viewer -- D:\game\data.xp3
```

## 项目结构

- `crates/suzu-app`：高层视觉小说 app facade。
- `crates/suzu-script`：DSL parser、compiler、VM command queue、diagnostics 和 extension registration。
- `crates/suzu-render`：render layer data、transition、post-process configuration 和 shader loading。
- `crates/suzu-text`：markup normalization、reveal state、ruby data、vertical layout 和 voice reveal sync。
- `crates/suzu-audio`：audio channel state、fade、snapshot 和 backend command synchronization。
- `crates/suzu-save`：JSON save slot、quicksave、autosave、thumbnail、history 和 audio state。
- `crates/suzu-asset`：texture discovery、async loading、LRU cache、manifest、`.suzupack` archive 读取和实验性 KiriKiri XP3 archive 读取。
- `crates/suzu-input`：keyboard、mouse、wheel 和 selection trigger map。
- `crates/suzu-platform`：桌面 `winit`/`wgpu` 集成和平台配置类型。
- `crates/suzu-editor-core`：可视化剧本编辑器 document model、import/export、graph diagnostics、project scan 和 undo command。
- `tools/suzu-launcher`：统一桌面入口，用于打开 Suzu 项目、导入 XP3 archive 和运行 preview。
- `tools/suzu-xp3-viewer`：桌面 XP3 检查和图片/文本预览工具。

## 运行时流程

1. 将 `.szs` source 解析为 `ScriptDocument`。Parser 会从脚本头检测 `syntax=classic`、`syntax=indent`、`syntax=braces` 或 `syntax=markup`。
2. 将 document 编译为 VM commands。
3. 把 commands 交给 `SuzuApp`。
4. 如果 `GameConfig.title_screen.enabled` 为 true，则先显示标题界面。
5. 用 frame delta 和 input event 推进 app。
6. 用 platform renderer 渲染 app scene。
7. 通过 save manager 捕获存档。

## XP3 资源

`suzu-asset` 通过 `Xp3Archive` 提供实验性 KiriKiri XP3 archive 解析。Reader 会索引 XP3 `File` entry，提取 stored 或 zlib-compressed segment，并可以直接注册到 `AssetManager`：

```rust
let mut app = SuzuApp::default();
app.register_xp3_file("data.xp3")?;
app.load_script_asset("main")?;
```

注册 XP3 后，支持的 entry 会作为普通 asset 暴露。例如 `scenario/main.szs` 会变成脚本 asset `main`，`image/bg_school.png` 会变成 texture asset `bg_school`。脚本可以像普通资源一样引用这些 id，例如 `@bg file="bg_school"`。

受保护 XP3 entry 会被索引用于报告，但 Project Suzu 默认只读取明文 entry。拥有兼容外部 XP3 processor 的项目可以通过 plugin hook 传入：

```rust
use suzu_asset::Xp3PluginModule;

let module = Xp3PluginModule::from_json_file("xp3-plugin.json")?;
app.register_xp3_file_with_options("data.xp3", module.xp3_options())?;
```

仓库不包含游戏专用 XP3 processor 或私有处理规则。此类 processor 应保留在公开仓库之外，并由有权使用它们的应用提供。

手动测试时可用 `suzu-xp3-viewer` 打开 XP3 path。它会列出 indexed entry、标记 protected entry、预览明文图片资源和 UTF-8 script/text 文件。选择 `.szs` script entry 后按 Start Game，可注册 XP3、加载脚本并运行嵌入式游戏 preview。

## 标题界面

设置 `GameConfig.title_screen.enabled = true` 后，运行时会从标题菜单开始，而不是立刻推进脚本。内置标题菜单支持 Start、Continue、Load、Settings 和 Quit。Start 会重置运行时并推进脚本到第一个等待点；Continue 会优先恢复 autosave 或 slot 0；Load 会打开读档页，可选择 autosave 和前 5 个 slot；Settings 会打开标题设置页，可调整文本速度、自动播放延迟和主音量。`TitleScreenConfig.background_texture` 可以指向已注册的 texture id，用作游戏标题背景；`TitleScreenConfig.labels` 可以覆盖菜单文本，便于本地化。系统菜单中的 Return Title 会重置运行时并重新显示标题界面。

## 示例

- `examples/hello-world`：最小脚本、标题界面和资源打包流程。
- `examples/branching-story`：标题界面、选择项、标签和条件变量。
- `examples/ui-save-load-demo`：标题界面、存档/读档、设置、历史和菜单流程。
- `examples/short-vn-demo`：首个完整短篇 VN 切片，覆盖标题界面、选择、变量、自动存档、效果和打包。
- `examples/stress-scene`：benchmark 输入用脚本级压力场景。
- `examples/web-browser-shell`：未来 Wasm bundle 的静态浏览器 canvas shell。

## 脚本编写风格

Classic `.szs` 仍是默认风格：

```text
@script version=1
@bg file="school"
# Suzu
Hello.
```

手写剧情时，indent 风格通常更容易扫读：

```text
@script version=1 syntax=indent
bg file="school"
Suzu: Hello.
choice "Library" goto=library
label library:
Suzu: Let us begin.
```

工具生成脚本时，如果更适合生成结构化文本，可以选择 `syntax=braces` 或 `syntax=markup`。完整风格说明和等价示例见 `docs/scripting-reference.zh-CN.md`。

## 可视化剧本编辑器

可视化剧本编辑器计划见 `docs/visual-script-editor-development-plan.md`。其中覆盖 editor MVP 范围、原生 Rust 桌面架构、`.szs` import/export 模型、node graph 设计、resource picker、diagnostics、preview workflow、tests 和 development milestones。

初始 editor binary 为 `suzu-editor`。它可以扫描 Project Suzu 文件夹、打开 `.szs`、显示导入后的 visual nodes、编辑常见 node fields、导出回 `.szs`，并运行 graph/compile diagnostics。
