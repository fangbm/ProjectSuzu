# Project Suzu 可视化剧本编辑器开发文档

> 文档状态：规划稿  
> 目标版本：Editor MVP  
> 适用项目：Project Suzu  
> 主要目标：为 `.szs` 剧本提供可视化编写、预览、校验、导入导出和资产引用管理能力。

## 一、建设目标

Project Suzu 当前已经具备脚本解析、编译、运行时预览、资源管理、存档和桌面示例。下一阶段需要补齐面向创作者的可视化剧本编辑器，让不熟悉 DSL 的用户也能创建视觉小说流程。

编辑器应做到：

- 可视化编辑剧情段落、对白、背景、角色、选项、条件、跳转、音频和系统命令。
- 与 `.szs` 文本脚本双向兼容，导入后能编辑，导出后能被现有 `suzu-compiler` 和运行时直接使用。
- 在编辑器内进行脚本诊断，展示行列、节点位置、缺失资源、死链跳转、条件格式错误。
- 连接现有运行时能力，支持快速预览当前场景或从指定节点开始播放。
- 管理项目资源，提供背景、角色立绘、语音、BGM 的浏览、引用和缺失检测。
- 保持纯本地工作流，优先支持 Windows / macOS / Linux 桌面端。

## 二、产品范围

### 2.1 MVP 必须完成

- 项目打开与保存：打开 Project Suzu 工程目录，识别 `script/`、`assets/`、配置文件和存档目录。
- `.szs` 导入：解析现有脚本为编辑器文档模型，保留可恢复的文本信息。
- `.szs` 导出：从可视化文档生成脚本，输出稳定、可读、可编译的文本。
- 剧情编辑：对白块、旁白块、命令块、选择块、标签块、条件块、跳转块、调用块。
- 资源选择器：背景、角色、音频、语音文件引用选择和缺失提示。
- 分支流程视图：展示标签、跳转、选择分支和不可达节点。
- 属性面板：编辑当前选中节点的字段。
- 实时诊断：复用 `suzu-script` 编译诊断，并追加编辑器级结构诊断。
- 运行预览：从开头或当前节点生成临时脚本并调用 `SuzuApp` 预览。
- 撤销/重做：覆盖节点增删、字段修改、排序、连接修改。

### 2.2 MVP 暂不实现

- 多人协作和云端同步。
- 内置图片、音频、Live2D 制作工具。
- 复杂时间轴动画编辑器。
- 所见即所得 UI 皮肤编辑器。
- 全量脚本语言高级重构。
- 手机端编辑器。

### 2.3 后续增强

- 时间轴式演出编辑：背景转场、角色移动、缩放、淡入淡出、特效。
- 语音对齐工具：按音频波形调整文字显示节奏。
- 可视化变量调试器：运行时查看变量、分支条件、已读状态。
- 剧情统计：字数、分支数、角色出场、语音覆盖率。
- 模板库：常用开场、章节、选择、存读档、标题界面模板。
- 插件系统：允许用户定义自定义命令的可视化表单。

## 三、目标用户与核心流程

### 3.1 目标用户

- 编剧：主要编辑对白、分支、章节结构。
- 演出：配置背景、角色位置、表情、转场、音效。
- 程序：维护自定义命令、变量、构建和发布。
- 测试：检查死分支、缺失资源、脚本编译错误。

### 3.2 核心流程

1. 用户打开 Project Suzu 工程目录。
2. 编辑器扫描脚本和资源，建立项目索引。
3. 用户选择一个 `.szs` 文件进入可视化编辑。
4. 左侧查看剧情结构或分支图，中间编辑节点，右侧修改属性。
5. 保存时生成编辑器文档快照和 `.szs` 脚本文本。
6. 用户点击预览，编辑器调用运行时播放当前剧本片段。
7. 发布前运行诊断，确认脚本、资源和跳转均有效。

## 四、技术方案

### 4.1 推荐技术栈

优先采用 Rust 原生桌面方案：

- UI：`egui` / `eframe`。
- 渲染预览：复用 `suzu-app`、`suzu-platform`、`suzu-render`。
- 脚本解析编译：复用 `suzu-script`。
- 资源索引：复用 `suzu-asset`。
- 配置保存：`serde` + JSON。
- 文件监听：后续可接入 `notify`，MVP 可手动刷新。

选择理由：

- 与现有 Rust workspace 一致。
- 便于复用解析、编译、诊断、资源和运行时。
- 避免引入 Web 框架和跨语言通信成本。
- 桌面工具可随 Release 一起打包。

### 4.2 新增模块规划

```text
crates/
  suzu-editor-core/
    src/
      document.rs        # 编辑器文档模型
      graph.rs           # 剧情图、标签、跳转、可达性分析
      import.rs          # .szs -> EditorDocument
      export.rs          # EditorDocument -> .szs
      diagnostics.rs     # 编辑器级诊断
      project.rs         # 工程扫描与路径管理
      undo.rs            # 命令式撤销/重做

tools/
  suzu-editor/
    src/
      main.rs            # 桌面入口
      app.rs             # eframe App
      panels/
        project_panel.rs
        outline_panel.rs
        graph_panel.rs
        node_panel.rs
        inspector_panel.rs
        diagnostics_panel.rs
        preview_panel.rs
      widgets/
        asset_picker.rs
        command_form.rs
        condition_editor.rs
        dialogue_editor.rs
```

### 4.3 依赖关系

```text
suzu-editor
  -> suzu-editor-core
  -> suzu-app
  -> suzu-platform

suzu-editor-core
  -> suzu-script
  -> suzu-asset
  -> suzu-save
  -> suzu-core
```

编辑器核心不依赖 GUI，保证导入、导出、诊断和图分析可以单独测试。

## 五、编辑器文档模型

### 5.1 顶层结构

```rust
pub struct EditorDocument {
    pub version: u32,
    pub source_path: Option<PathBuf>,
    pub metadata: EditorMetadata,
    pub nodes: Vec<EditorNode>,
    pub edges: Vec<EditorEdge>,
    pub comments: Vec<EditorComment>,
}
```

### 5.2 节点类型

```rust
pub enum EditorNodeKind {
    ScriptHeader { version: u32 },
    Label { name: String },
    Dialogue { speaker: Option<String>, text: String },
    Background { file: String, method: TransitionForm, time_ms: u32 },
    Character { name: String, face: Option<String>, position: PositionForm, layer: i32, flip: bool },
    HideCharacter { name: String },
    Animation { target: String, form: AnimationForm },
    Effect { form: EffectForm },
    Choice { options: Vec<ChoiceOptionForm> },
    SetVariable { name: String, value: String },
    If { condition: String, then_nodes: Vec<NodeId>, else_nodes: Vec<NodeId> },
    Jump { label: String },
    Call { label: String },
    Return,
    Wait { time_ms: u32 },
    Audio { form: AudioForm },
    MessageBox { visible: bool },
    SaveName { text: String },
    AutoSave,
    CustomCommand { name: String, args: Vec<CommandArgForm> },
    RawText { source: String },
}
```

### 5.3 节点元数据

```rust
pub struct EditorNode {
    pub id: NodeId,
    pub kind: EditorNodeKind,
    pub title: String,
    pub source_span: Option<SourceSpan>,
    pub layout: NodeLayout,
    pub locked: bool,
}
```

`source_span` 用于把可视化节点映射回 `.szs` 行列。导入现有脚本时必须尽量保留它，便于诊断跳转。

### 5.4 连接模型

```rust
pub enum EditorEdgeKind {
    Sequence,
    ChoiceBranch { option_index: usize },
    ConditionalThen,
    ConditionalElse,
    Jump,
    Call,
}
```

顺序连接用于普通剧情流；选择、条件、跳转和调用连接用于生成剧情图和死链诊断。

## 六、脚本导入与导出

### 6.1 导入规则

导入 `.szs` 时按以下步骤处理：

1. 调用 `suzu_script::parse_script` 获取文档和源位置信息。
2. 按脚本行顺序转换为 `EditorNode`。
3. 连续对白行可合并为一个 Dialogue 节点。
4. 连续 `@choice` 合并为一个 Choice 节点。
5. `@if/@else/@endif` 构造成条件节点和子块。
6. 标签、跳转、调用生成图连接。
7. 无法识别但 parser 保留的文本进入 RawText 节点。

### 6.2 导出规则

导出 `.szs` 时必须满足：

- 生成文本稳定，同一文档多次导出不产生无意义 diff。
- 优先输出 Project Suzu 推荐格式。
- 字符串参数统一使用双引号，并转义内部引号。
- 选择块连续输出，避免分散生成无法被编译器识别的选择组。
- 保留 RawText 节点原文，减少破坏用户手写内容。
- 导出后必须立即调用 `compile_script` 进行验证。

### 6.3 往返兼容目标

最小要求：

```text
source.szs -> import -> export -> compile ok
```

增强要求：

```text
source.szs -> import -> export -> import -> equivalent document graph
```

不要求首版做到文本字节级完全一致，但需要保持语义一致。

## 七、UI 设计

### 7.1 主界面布局

```text
┌──────────────────────────────────────────────────────────────┐
│ 菜单栏：文件 / 编辑 / 剧本 / 资源 / 预览 / 发布 / 帮助          │
├──────────────┬──────────────────────────────┬────────────────┤
│ 项目与大纲     │  中央编辑区                    │ 属性检查器       │
│              │  - 剧情列表                    │                │
│ script/      │  - 分支图                      │ 节点字段         │
│ assets/      │  - 场景预览                    │ 资源选择         │
│ diagnostics  │                              │ 条件编辑         │
├──────────────┴──────────────────────────────┴────────────────┤
│ 底部：诊断 / 搜索结果 / 构建输出 / 预览日志                     │
└──────────────────────────────────────────────────────────────┘
```

### 7.2 主要面板

- 项目面板：显示脚本、资源、配置、发布脚本。
- 大纲面板：按标签、章节、对白块显示结构。
- 分支图面板：显示剧情流程、选择、跳转、调用、不可达节点。
- 节点编辑区：顺序编辑剧情节点，支持拖拽排序。
- 属性检查器：编辑节点字段，包含校验提示。
- 资源选择器：预览图片、试听音频、插入资源路径。
- 诊断面板：显示错误、警告、提示，点击定位节点或源码行。
- 预览面板：运行当前脚本片段，支持从选中节点开始。

### 7.3 交互要求

- 新建节点使用菜单或工具栏按钮。
- 节点可拖拽排序。
- 选择项可以增删、重命名、绑定标签、设置条件。
- 标签重命名时提示是否同步更新跳转引用。
- 资源路径字段提供选择器，不要求用户手输。
- 诊断错误点击后聚焦对应节点。
- 保存前自动运行导出验证。

## 八、功能详细设计

### 8.1 项目管理

工程目录识别规则：

- 必须存在 `Cargo.toml` 或 `script/`。
- 推荐存在 `assets/`、`README.md`、`settings.json`。
- 可扫描 `examples/*/script` 作为示例剧本。

项目索引内容：

- 脚本文件列表。
- 资源文件列表。
- 资源类型：背景、角色、音频、语音、其他。
- 未引用资源。
- 缺失资源引用。

### 8.2 剧情节点编辑

对白节点字段：

- speaker：角色名，可为空。
- text：正文，支持 `[l]`、`[r]`、`[ruby=text]base[/ruby]`。
- voice：可选，导出为前置 `@voice`。

背景节点字段：

- file。
- method：instant / crossfade / fade-through-color。
- time_ms。
- color：fade-through-color 时可用。

角色节点字段：

- name。
- face。
- pos：left / center / right / custom。
- custom x/y。
- size w/h。
- layer。
- flip。

选择节点字段：

- option text。
- goto label。
- condition。
- 是否自动创建目标标签。

### 8.3 条件编辑器

MVP 允许文本输入条件表达式，并提供轻量辅助：

- 变量名补全。
- 运算符选择：`==`、`!=`、`>`、`<`、`>=`、`<=`。
- 布尔取反：`!flag`。
- 编译器验证。

后续可升级为可视化表达式树。

### 8.4 资源管理

资源扫描规则：

- 图片：png / jpg / jpeg / webp。
- 音频：ogg / wav / mp3 / flac。
- 脚本：szs。
- 资源 ID 默认使用无扩展名文件名，与 `AssetManager` 现有行为保持一致。

诊断规则：

- 引用的图片不存在：错误。
- 引用的音频不存在：警告或错误，由项目设置决定。
- 文件名与资源 ID 冲突：警告。
- 大小写不一致：跨平台警告。

### 8.5 预览系统

预览方式：

- 生成临时 `.szs` 字符串。
- 创建独立 `SuzuApp`。
- 注册当前项目资源。
- 从脚本开头或选中节点对应位置播放。

MVP 可以先使用独立预览窗口；后续再嵌入到编辑器面板。

预览入口：

- Play From Start。
- Play From Selected Node。
- Compile Only。
- Capture Frame。

### 8.6 诊断系统

诊断级别：

- Error：无法导出或无法编译。
- Warning：可运行但可能有问题。
- Info：优化建议。

诊断来源：

- `suzu-script` 解析和编译错误。
- 编辑器图分析错误。
- 资源索引错误。
- 导出验证错误。

图分析诊断：

- 跳转目标标签不存在。
- 标签重复。
- 选择项无目标。
- 节点不可达。
- 条件块为空。
- 调用后永不返回。
- 脚本没有任何可播放内容。

## 九、文件格式

### 9.1 编辑器工程文件

建议新增：

```text
.suzu/editor/project.json
.suzu/editor/layout.json
.suzu/editor/cache/
```

### 9.2 编辑器文档快照

每个脚本可选生成旁路文件：

```text
script/main.szs
script/main.szs.editor.json
```

原则：

- `.szs` 是权威运行文件。
- `.editor.json` 保存节点布局、折叠状态、注释、编辑器元数据。
- 没有 `.editor.json` 也必须能从 `.szs` 导入。

## 十、撤销与重做

采用命令模式：

```rust
pub enum EditorCommand {
    AddNode { node: EditorNode, index: usize },
    RemoveNode { node_id: NodeId },
    UpdateNode { node_id: NodeId, before: EditorNodeKind, after: EditorNodeKind },
    MoveNode { node_id: NodeId, from: usize, to: usize },
    AddEdge { edge: EditorEdge },
    RemoveEdge { edge_id: EdgeId },
}
```

要求：

- 每个用户操作生成一个可逆命令。
- 文本连续输入需要合并为单个撤销步骤。
- 保存后记录 clean checkpoint，用于提示未保存修改。

## 十一、测试计划

### 11.1 单元测试

- `.szs` 导入节点。
- 编辑器节点导出 `.szs`。
- choice 合并和导出。
- if/else 导入导出。
- label/jump/call 图连接。
- 缺失资源诊断。
- 撤销/重做命令。

### 11.2 黄金文件测试

建立测试目录：

```text
tests/fixtures/editor/
  simple_dialogue.szs
  branching_choice.szs
  conditional_route.szs
  audio_voice.szs
  custom_command.szs
```

每个 fixture 验证：

- 导入成功。
- 导出成功。
- 导出结果可编译。
- 图诊断符合预期。

### 11.3 集成测试

- 打开示例工程。
- 导入 `hello-world`。
- 编辑一行对白。
- 导出。
- 编译通过。
- 用 `SuzuApp` 预览第一句。

### 11.4 手工验收

- Windows 双击打开编辑器。
- 新建剧本并添加对白。
- 添加选择分支。
- 添加背景和角色。
- 保存后运行示例。
- 删除资源后诊断能提示。
- 标签改名后引用可同步。

## 十二、开发里程碑

### Phase E0 - 基础工程

- 新增 `suzu-editor-core` crate。
- 新增 `suzu-editor` tool。
- 接入基础窗口和菜单。
- 打开工程目录并显示脚本列表。

验收标准：

- `cargo run -p suzu-editor` 能打开桌面窗口。
- 可选择工程目录并列出 `.szs` 文件。

### Phase E1 - 导入导出核心

- 实现 `EditorDocument`。
- 实现 `.szs` 导入。
- 实现 `.szs` 导出。
- 建立黄金文件测试。

验收标准：

- 示例脚本导入导出后 `compile_script` 通过。
- choice、if/else、label/jump/call 有测试覆盖。

### Phase E2 - 节点编辑 UI

- 剧情列表视图。
- 属性检查器。
- 新增/删除/移动节点。
- 对白、背景、角色、选择、音频表单。
- 撤销/重做。

验收标准：

- 可不写 DSL 完成一段含背景、角色、对白和选择的剧本。
- 保存后示例运行正常。

### Phase E3 - 资源和诊断

- 资源扫描。
- 图片和音频选择器。
- 缺失资源诊断。
- 跳转和标签图诊断。
- 诊断面板定位节点。

验收标准：

- 删除被引用资源后能显示错误。
- 跳转到不存在标签时能显示错误并定位节点。

### Phase E4 - 分支图和预览

- 分支图视图。
- 从开头预览。
- 从选中节点预览。
- 编译输出面板。

验收标准：

- 分支剧情可以在图中查看。
- 当前节点预览能快速播放到目标片段。

### Phase E5 - 打包发布

- 打包脚本包含 `suzu-editor`。
- GitHub Release 上传编辑器二进制。
- README 和用户指南补充编辑器使用教程。

验收标准：

- Release 包内包含编辑器。
- 用户下载后可打开、编辑、保存、预览示例剧本。

## 十三、风险与对策

| 风险 | 影响 | 对策 |
|------|------|------|
| `.szs` 文本与可视化模型不能完全往返 | 用户手写脚本可能丢格式 | 使用 RawText 节点和旁路 `.editor.json` 保留信息 |
| 条件和自定义命令难以可视化 | 表单复杂度上升 | MVP 先提供文本字段，后续做插件表单 |
| 预览嵌入复杂 | 开发周期变长 | 首版用独立预览窗口 |
| 资源 ID 与文件路径规则不统一 | 引用错误 | 复用 `AssetManager` 扫描规则并提供诊断 |
| GUI 引入依赖导致包体增大 | Release 变大 | 编辑器作为独立工具打包，不影响运行时 crate |

## 十四、完成定义

当以下条件全部满足时，可视化剧本编辑器 MVP 视为完成：

- 能打开 Project Suzu 工程。
- 能导入、编辑、保存、导出 `.szs`。
- 能创建包含背景、角色、对白、选择、变量、条件、跳转和音频的剧本。
- 导出的脚本能通过 `suzu-compiler`。
- 能检测缺失资源和无效跳转。
- 能从编辑器启动预览。
- 支持撤销/重做。
- Windows Release 包中包含可双击启动的编辑器。
- 中文 README 和用户指南包含编辑器教程。

## 十五、开发检查命令

每个阶段完成后运行：

```powershell
C:\Users\方便面\.cargo\bin\cargo.exe fmt --all -- --check
C:\Users\方便面\.cargo\bin\cargo.exe clippy --workspace --all-targets -- -D warnings
C:\Users\方便面\.cargo\bin\cargo.exe test --workspace
.\scripts\package-desktop.ps1 -Check
```

编辑器核心新增后还应补充：

```powershell
C:\Users\方便面\.cargo\bin\cargo.exe test -p suzu-editor-core
C:\Users\方便面\.cargo\bin\cargo.exe run -p suzu-editor
```
