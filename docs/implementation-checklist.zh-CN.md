# Project Suzu 实现检查清单

这份清单记录 Project Suzu 从规划走向第一套完整视觉小说框架切片的实现状态。状态含义：

- `[x]` 已实现并有测试覆盖
- `[~]` 部分实现，可用但尚不完整
- `[ ]` 尚未实现

## Phase 0 - 基础设施

- [x] 包含核心 crate、工具和示例工程的 Rust workspace。
- [x] 格式化、clippy 和测试 CI workflow。
- [x] 核心数学与错误类型：`Vec2`、`Color`、`Rect`、`Affine2`。
- [x] script、render、text、audio、asset、save、input、platform 和 app 的类型化模块边界。
- [x] tag release workflow 为 Linux 和 Windows 构建桌面产物。

## Phase 1 - 渲染核心

- [x] 桌面 `winit` 窗口和 `wgpu` surface/device 初始化。
- [x] 精灵渲染：纹理上传、tint、opacity、scale、rotation、horizontal flip。
- [x] 混合模式：normal、add、multiply、screen。
- [x] 背景转场：instant、crossfade、fade-through-color。
- [x] Tween 动画：move、zoom、shake、fade。
- [x] 通过 `cosmic-text` 进行帧级文本渲染。
- [x] 通过 `SpriteLayer` 和可复用 `LayerStack` API 实现保留式图层模型。
- [x] 后处理 pipeline 配置：bloom、tone mapping 和用户开关。
- [x] 用户定义 WGSL shader 加载与示例。

## Phase 2 - 文本系统

- [x] CJK 横排文本渲染和打字机显示。
- [x] 内联控制标签：`[l]` 点击等待和 `[r]` 换行。
- [x] confirm 输入下的对白等待和跳过行为。
- [x] 对白历史文本规范化。
- [x] `WritingMode::VerticalRl` 类型和竖排 glyph layout。
- [x] Ruby parser 和注音布局数据。
- [x] 对话框渲染包含可配置样式、说话人区域和点击提示文本。
- [x] Backlog/history UI，支持滚动和语音回放 hook。

## Phase 3 - 脚本与音频

- [x] DSL parser 支持注释、说话人、标签、命令、带引号参数和行内注释。
- [x] VM 命令队列支持标签、jump、call、return、插入命令和可存档 call stack。
- [x] 核心脚本命令：`@bg`、`@char`、`@hidechar`、`@anim`、`@fx`、`@choice`、`@if/@else/@endif`、`@set`、`@jump`、`@call`、`@return`、`@wait`、`@savename`、`@autosave`、`@hidemsg`、`@showmsg`。
- [x] 角色控制：face texture selection、position、size、layer、flip、show/update/hide。
- [x] 音频状态模型：BGM 和 voice channel，支持淡入淡出和存档快照。
- [x] 对白语音 cue 命令：`@voice` 将下一行文本绑定到 `VoiceSync`。
- [x] 音频后端接口和状态后端命令同步；`rodio`/`cpal` adapter 可通过 backend trait 接入。
- [x] Voice sync 支持基于 timestamp 的 reveal plan，并用语音播放时间推进文本 reveal。
- [x] Lua 扩展层，带可选 `mlua` command-list binding 和自定义命令注册。
- [x] 脚本诊断包含 source span、行列错误和命令建议。
- [x] 脚本格式版本和迁移规则。

## Phase 4 - 系统功能

- [x] Save manager：slot、quicksave、autosave、JSON 读写、脚本位置、call stack、scene、变量、history 和 audio state。
- [x] Asset manager：注册 texture、递归发现 PNG/JPEG/WebP、注册 package manifest、异步加载 texture、LRU 缓存。
- [x] 输入映射：桌面键盘、鼠标、滚轮和选择事件，支持可配置 trigger binding。
- [x] 存档缩略图。
- [x] 异步 asset loading 和 LRU cache。
- [x] 资源包 manifest 格式、archive reader、压缩元数据和 checksum 验证。
- [x] `suzu-packer` CLI 可扫描 asset root 并输出排序 JSON manifest。
- [x] `suzu-packer` 支持 archive 写入、压缩、checksum 和 package reader。
- [x] 系统菜单：settings、save、load、history、return title、quit。
- [x] Auto mode、已读对白 skip mode、read-state 持久化和可配置 text speed。
- [x] Project window/script 设置和用户 audio/text/window 设置持久化，并接入系统菜单。

## Phase 5 - 平台、打磨和示例

- [x] 完整桌面发布打包，本地 PowerShell bundle 脚本和 GitHub release workflow。
- [x] Android/iOS touch input 抽象和构建目标描述。
- [x] WebAssembly 构建目标描述和浏览器示例 shell。
- [x] Live2D integration adapter 边界。
- [x] Video playback adapter 边界。
- [x] 性能 benchmark CLI 和 stress scene 脚本。
- [x] 覆盖上手、脚本和发布打包的用户文档。
- [x] 三个示例：minimal hello-world、branching story、完整 UI/save/load demo。

## Phase 6 - 可视化剧本编辑器

- [x] Editor 开发计划覆盖 MVP 范围、架构、document model、UI、诊断、预览、测试和 milestone。
- [x] `suzu-editor-core` crate，包含 editor document model、`.szs` import/export、graph diagnostics、project scan 和 undo command primitive。
- [x] 初版 `suzu-editor` 桌面 binary：project scan、script open、visual node list、基础 node inspector、export/save 和 diagnostics panel。
- [x] Release packaging 包含 `suzu-launcher`、`suzu-editor`、`suzu-xp3-viewer` 和编辑器规划文档。
- [ ] 覆盖所有内置命令的丰富 node form。
- [ ] 可编辑 edge 的 branch graph visualization。
- [ ] 图片和音频资源 picker preview。
- [ ] 从指定 node 开始的嵌入式或伴随运行时预览。
- [ ] Editor sidecar `.editor.json` layout 持久化。
- [ ] import/export 等价性的 golden file fixtures。

## 当前验证门禁

完成每个阶段性切片后运行：

```powershell
C:\Users\方便面\.cargo\bin\cargo.exe fmt --all -- --check
C:\Users\方便面\.cargo\bin\cargo.exe clippy --workspace --all-targets -- -D warnings
C:\Users\方便面\.cargo\bin\cargo.exe test --workspace
```
