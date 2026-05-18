# 更新日志

## 未发布

## 0.2.1 - 2026-05-18

- 改进 `suzu-player` 无参数启动：从 release 包双击启动且当前目录不是 Suzu 工程时，会自动打开内置的 `templates/starter-vn` starter project。
- 保持显式工程目录和 `--entry` 启动的严格行为，因此用户主动传入的错误工程路径仍会报告真实加载错误。
- 增加更清楚的“未找到工程”提示，并在工程布局指南中记录内置模板 fallback 行为。

## 0.2.0 - 2026-05-17

- 为 v0.2.0 工程收敛拆分桌面平台层、脚本编译器和 app runtime 测试。
- 增加首个 `examples/short-vn-demo` 切片，以及完整短篇 VN proof 的规划文档。
- 更新脚本参考，让 `@anim` 和 `@fx` 示例匹配当前属性名。
- 增加 `syntax=indent`、`syntax=braces`、`syntax=markup` parser front end，并保持现有 classic `.szs` 语法为默认。
- 为仓库文档、模板、示例和 release package 增加简体中文配套文档。
- 改进内置标题界面，支持可配置背景纹理、本地化菜单文本、读档/设置子页和鼠标命中。
- 修复桌面标题界面和系统菜单的鼠标交互，扩大对话框 next 提示区域，并让脚本结束后清空最后一句对白。
- 增加通过 `game.suzu.toml`、`scenario/main.szs`、`suzu-project` 和零代码 `suzu-player` 启动的脚本优先低门槛工程流程。
- 增加 `templates/starter-vn`、工程布局文档、launcher 的运行/检查/打开编辑器动作，以及 `bg`、`ch`、`voice` 等短脚本命令。

## 0.1.6 - 2026-05-15

- 增加入门指南和最小视觉小说模板，方便新项目起步。
- 改进 GUI `--check` 诊断，让 launcher、XP3 viewer 和 editor 在不开窗口的情况下报告已验证路径与 archive/plugin 状态。
- 将 XP3 viewer 的 archive 索引和条目预览放到后台执行，提升大 archive 下的 UI 响应性。
- 在 Windows 上隐藏外部 XP3 processor 的控制台窗口，减少预览和加载资源时的闪窗。
- 为 XP3 脚本/文本条目增加 UTF-8、UTF-16 和 Shift_JIS 检测预览。
- 保持本地项目续写上下文不进入 Git 仓库。

## 0.1.5 - 2026-05-15

- 增加详细框架使用指南，覆盖项目设置、脚本、资源、运行时 API、工具、打包和排错。
- 将 runtime app facade 拆分为更聚焦的模块，同时保持 `SuzuApp`、`TitleMenuAction`、`SystemMenuAction` 公共导出稳定。
- 拆分 launcher 和 XP3 viewer GUI 入口，并为 launcher、XP3 viewer、editor 增加 headless `--check`。
- 增加 workspace smoke-test crate，覆盖脚本编译、runtime 推进、package archive 加载、存档恢复、明文 XP3 加载和 KAG 转换。
- 增加最小 `suzu-packer` library 入口，以及通过 `AssetManager` 注册 package archive 的能力。
- 增加 `cargo-about` 第三方许可证、API 稳定说明、品牌说明、法律/安全 plugin 指南和 release package 检查。
- 增加 `suzu.xp3-plugin.v1` 外部 XP3 processor 接口文档。
- 更新 CI 和 release 质量门禁，运行 GUI check 命令。

## 0.1.4 - 2026-05-14

- 合入实验性 XP3 archive reader 和 XP3-backed asset loading。
- 增加 XP3 viewer 和统一 launcher preview 工具。
- 增加 KRKR package scan 和有限 KAG-to-Suzu 转换实验。
- 增加明确 XP3 支持边界和外部 XP3 plugin hook。
- 扩展 CI 触发到 feature branch 和版本标签。
- 增加 release 质量门禁、法律指南和 Windows/Linux artifact 准备。
- 修复 workspace repository metadata。

## 0.1.3 - 2026-05-11

- 增加首个 visual script editor MVP，包括 `suzu-editor-core` 和 `suzu-editor` 桌面工具。
- 增加 `.szs` import/export、graph diagnostics、project scanning、undo command primitive、node editing UI 和 editor packaging。
- 更新 release 文档，让桌面包包含 visual script editor。

## 0.1.0 - 2026-05-11

- 创建 Project Suzu Rust workspace，包含 app、script、render、text、audio、asset、save、input、platform 等边界清晰的 crates。
- 增加桌面 `winit`/`wgpu` 渲染、保留式 sprite layers、transition、tween animation、text rendering、post-process 配置和 WGSL shader 加载。
- 增加 `.szs` parser/compiler、VM queue、label、choice、variable、condition、call、wait、save command、message visibility、diagnostics、versioning 和可选 Lua extension registration。
- 增加 dialogue reveal、history UI、ruby annotation data、vertical glyph layout、voice cue marker 和 timestamp-driven voice reveal plan。
- 增加 audio channel state、fade、save snapshot 和 backend command synchronization。
- 增加 save/load slot、thumbnail、autosave/quicksave、config persistence、input map、system menu、auto mode、skip mode 和 read-state persistence。
- 增加 asset discovery、async texture loading、LRU cache、manifest、`.suzupack` archive 写入/读取、compression metadata 和 checksum validation。
- 增加桌面示例、web shell、stress scene、benchmark CLI、本地桌面打包、GitHub CI/release workflow 和用户/开发文档。
- 增加 license、contribution、security 和 release note 等仓库元数据。
