# Galgame 框架项目规划书

> **项目名称**：Project Suzu（鈴）
> **技术栈**：Rust + wgpu + winit + mlua + cosmic-text
> **目标平台**：Windows / macOS / Linux / Android / iOS / WebAssembly
> **框架定位**：面向 Galgame（视觉小说）的专用 2D 渲染与叙事框架

---

## 一、项目愿景与核心原则

### 1.1 愿景
打造一个专为 Galgame 设计的现代渲染框架，具备以下特性：
- **跨平台原生**：单一代码库覆盖桌面、移动端与 Web
- **高性能 2D 渲染**：基于 GPU 的图层合成、后处理特效、Live2D 集成
- **中文特化**：竖排文本、Ruby 注音、换行禁则、CJK 排版
- **可扩展脚本**：类 KAG 的声明式 DSL，同时支持 Lua 脚本扩展
- **状态快照存档**：完整冻结/恢复游戏状态，支持无限存档位

### 1.2 核心原则
| 原则 | 说明 |
|------|------|
| **零 GC 卡顿** | Rust 所有权系统 + wgpu 显存管理，杜绝运行时垃圾回收 |
| **显式优于隐式** | 所有状态变更通过脚本指令驱动，无魔法黑箱 |
| **延迟加载** | 资源按需异步加载，支持流式语音与背景图 |
| **向后兼容** | 脚本格式版本化，旧存档可迁移至新版本 |

---

## 二、技术选型与依赖

### 2.1 核心依赖

| 模块 | Crate | 版本策略 | 选型理由 |
|------|-------|---------|---------|
| **GPU 渲染** | `wgpu` | 跟踪最新稳定版 | 跨平台 GPU 抽象，支持 Vulkan/Metal/DX12/WebGPU |
| **窗口管理** | `winit` | 跟踪最新稳定版 | Rust 生态标准，跨平台窗口与输入事件 |
| **脚本引擎** | `mlua` | 跟踪最新稳定版 | Lua 5.4 / LuaJIT 双后端，高性能脚本绑定 |
| **文本渲染** | `cosmic-text` | 跟踪最新稳定版 | 现代文本 shaping，支持复杂脚本与竖排布局 |
| **字体解析** | `skrifa` / `fontdue` | 跟踪最新稳定版 | 字体光栅化与 metrics 提取 |
| **图像解码** | `image` | 跟踪最新稳定版 | PNG/JPEG/WebP/AVIF 解码 |
| **音频播放** | `rodio` / `cpal` | 跟踪最新稳定版 | `rodio` 高层 API，`cpal` 底层音频设备访问 |
| **序列化** | `serde` + `ron` | 跟踪最新稳定版 | 人类可读配置 + 二进制存档 |
| **异步运行时** | `tokio` (桌面) / `wasm-bindgen-futures` (Web) | 条件编译 | 资源异步加载与网络请求 |
| **日志** | `tracing` | 跟踪最新稳定版 | 结构化日志，支持 span 追踪 |
| **错误处理** | `thiserror` + `anyhow` | 跟踪最新稳定版 | 框架层/用户层差异化错误处理 |

### 2.2 可选依赖

| 模块 | Crate | 用途 |
|------|-------|------|
| **Live2D** | Live2D Cubism Core SDK (Rust FFI) | 角色立绘动态表情 |
| **视频播放** | `ffmpeg-next` / `webm` | OP/ED 视频、剧情过场 |
| **压缩** | `zstd` / `lz4` | 资源包压缩与存档压缩 |
| **加密** | `aes-gcm` / `chacha20poly1305` | 资源包加密（可选） |
| **HTTP** | `reqwest` | DLC 下载、云端存档同步 |

---

## 三、架构设计

### 3.1 整体架构图

```
┌─────────────────────────────────────────────────────────────────────┐
│                         用户层 (User Layer)                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │
│  │  脚本文件    │  │  资源文件    │  │  配置文件    │                │
│  │  (.szs/.lua)│  │  (.png/.ogg)│  │  (.ron/.toml)│                │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                │
└─────────┼────────────────┼────────────────┼───────────────────────┘
          │                │                │
          ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        应用层 (Application Layer)                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │
│  │  脚本解析器   │  │  资源管理器   │  │  配置管理器   │                │
│  │  (Parser)    │  │  (AssetMgr)  │  │  (Config)    │                │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                │
│         │                │                │                       │
│         └────────────────┼────────────────┘                       │
│                          ▼                                         │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                    场景状态机 (Scene State Machine)            │ │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐           │ │
│  │  │ 背景层   │ │ 立绘层   │ │ 特效层   │ │ UI层    │           │ │
│  │  │ (BgLayer)│ │(CharLayer)│ │(FxLayer) │ │(UiLayer) │           │ │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘           │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        系统层 (System Layer)                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐│
│  │   渲染管线   │  │   音频系统   │  │   输入系统   │  │   存档系统   ││
│  │ (Renderer)  │  │  (Audio)    │  │  (Input)    │  │  (SaveSys)  ││
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘│
└─────────┼────────────────┼────────────────┼────────────────┼───────┘
          │                │                │                │
          ▼                ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        平台层 (Platform Layer)                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐│
│  │    wgpu     │  │    winit    │  │    rodio    │  │    文件系统   ││
│  │  (GPU抽象)   │  │ (窗口/输入)  │  │  (音频输出)  │  │  (std/fs)   ││
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 核心模块职责

#### 3.2.1 脚本系统 (Script Engine)

**设计目标**：
- 原生 DSL（`.szs` 文件）覆盖 90% 常用指令
- Lua 脚本覆盖复杂逻辑（变量计算、自定义动画、小游戏）
- 指令队列驱动，支持断点、快进、历史回溯

**DSL 语法示例**：
```szs
; 场景初始化
@bg file="bg_school_evening" time=800 method=crossfade
@playbgm file="bgm_school" loop=true fadein=2000

; 角色登场
@char name="eileen" face="happy" pos=center time=300
@char name="eileen" layer=2  ; 图层深度

; 对话
# 艾琳
你好，[player_name]！今天过得怎么样？[l][r]

; 选项分支
@choice "去教室" goto=label_classroom
@choice "去天台" goto=label_rooftop
@choice "回家" goto=label_home cond=flag_homework_done==true

; 条件分支
@if var= affection_eileen op=gt value=50
    @char name="eileen" face="blush"
    # 艾琳
    那个...能和你一起走吗？[l]
@endif

; 动画指令
@anim target="eileen" type=shake duration=500 intensity=3
@anim target="bg" type=zoom center=(0.5,0.5) scale=1.2 duration=1000 easing=ease_out_quad

; 特效
@fx type=flash color=#FFFFFF duration=200
@fx type=quake intensity=2 duration=800

; 系统指令
@savename text="第一章-相遇"
@autosave
```

**Lua 扩展接口**：
```lua
-- 自定义动画
function custom_animation(char_name, params)
    local char = suzu.char.get(char_name)
    char:move_to(params.x, params.y, params.duration, "ease_in_out")
    char:set_effect("glow", params.intensity)
end

-- 注册为脚本指令
suzu.script.register_command("custom_anim", custom_animation)
```

**核心数据结构**：
```rust
// 指令类型枚举
enum Command {
    Bg { file: String, time: u32, method: Transition },
    Char { name: String, face: String, pos: Position, layer: i32 },
    Text { speaker: String, content: Vec<TextSegment> },
    Choice { options: Vec<ChoiceOption> },
    If { var: String, op: CompareOp, value: Value, then_block: Vec<Command> },
    Anim { target: String, animation: Animation },
    Fx { effect: VisualEffect },
    // ... 共约 30-40 种基础指令
}

// 指令队列（支持跳转、调用、返回）
struct CommandQueue {
    commands: Vec<Command>,
    pc: usize,              // 程序计数器
    call_stack: Vec<usize>, // 子程序调用栈
    labels: HashMap<String, usize>, // 标签表
}
```

#### 3.2.2 渲染系统 (Renderer)

**渲染管线设计**：

Galgame 画面本质是 **2D 图层合成**，采用 **Retained Mode** 架构：

```
┌────────────────────────────────────────────┐
│              最终合成输出                    │
│  ┌────────────────────────────────────────┐ │
│  │           后处理阶段                     │ │
│  │  (泛光/色调映射/抗锯齿/自定义滤镜)        │ │
│  └────────────────────────────────────────┘ │
│              ↑                              │
│  ┌────────────────────────────────────────┐ │
│  │           UI / 文本层                   │ │
│  │  (对话框/选择支/历史记录/系统菜单)       │ │
│  └────────────────────────────────────────┘ │
│              ↑                              │
│  ┌────────────────────────────────────────┐ │
│  │           特效层 (FxLayer)              │ │
│  │  (粒子/全屏滤镜/闪白/震动/过渡动画)      │ │
│  └────────────────────────────────────────┘ │
│              ↑                              │
│  ┌────────────────────────────────────────┐ │
│  │           角色立绘层 (CharLayer)        │ │
│  │  (多角色/表情差分/Live2D/图层深度排序)   │ │
│  └────────────────────────────────────────┘ │
│              ↑                              │
│  ┌────────────────────────────────────────┐ │
│  │           背景层 (BgLayer)              │ │
│  │  (静态背景/动态背景/视差滚动)            │ │
│  └────────────────────────────────────────┘ │
└────────────────────────────────────────────┘
```

**图层节点设计**：
```rust
// 图层树节点
trait LayerNode {
    fn render(&self, ctx: &mut RenderContext, pass: &mut RenderPass);
    fn bounds(&self) -> Rect;
    fn opacity(&self) -> f32;
    fn transform(&self) -> Affine2;
    fn blend_mode(&self) -> BlendMode;
    fn z_index(&self) -> i32;
}

// 具体图层实现
struct SpriteLayer {
    texture: GpuTexture,
    position: Vec2,
    scale: Vec2,
    rotation: f32,
    opacity: f32,
    color_matrix: Mat4,     // 颜色调整（黑白/ sepia / 色调偏移）
    mask_texture: Option<GpuTexture>, // 遮罩
    tween: Option<Tween>,   // 当前动画插值
}

struct TextLayer {
    text_block: cosmic_text::Buffer,
    writing_mode: WritingMode,  // Horizontal / VerticalRl
    ruby_annotations: Vec<Ruby>,
    reveal_progress: f32,       // 0.0 ~ 1.0 渐显进度
    speaker_name: String,
    // ...
}
```

**过渡动画系统**：
```rust
enum Transition {
    CrossFade { duration: u32 },
    FadeThroughColor { color: Color, duration: u32 },
    Slide { direction: Direction, duration: u32 },
    Wipe { pattern: WipePattern, duration: u32 },
    Pixelate { block_size: u32, duration: u32 },
    Custom { shader: String, uniforms: HashMap<String, Value> },
}

// 过渡动画通过双缓冲 + 自定义 shader 实现
struct TransitionRenderer {
    source_texture: RenderTarget,
    target_texture: RenderTarget,
    progress: f32,  // 0.0 -> 1.0
    shader: wgpu::RenderPipeline,
}
```

**着色器架构**：
- **基础 2D 着色器**：顶点变换 + 纹理采样 + 颜色矩阵
- **文本着色器**：SDF 字体渲染 + 亚像素抗锯齿
- **后处理着色器**：泛光（Bloom）、色调映射、CRT 扫描线、老电影颗粒
- **过渡着色器**：像素化、波纹、百叶窗等（用户可自定义 WGSL）

#### 3.2.3 文本渲染系统 (Text Renderer)

**中文特化需求**：

| 特性 | 实现方案 | 状态 |
|------|---------|------|
| **横排文本** | `cosmic-text` 默认布局 | ✅ 原生支持 |
| **竖排文本** | `cosmic-text` + 自定义 `WritingMode::VerticalRl` | ⚠️ 需适配 |
| **Ruby 注音** | 在 `cosmic-text` 上层自建 Ruby 布局 | 🔧 需自研 |
| **逐字渐显** | 字形级裁剪 + 时间驱动 | 🔧 需自研 |
| **换行禁则** | ICU4X line breaker + CJK 规则扩展 | ⚠️ 需扩展 |
| **字体回退** | `fontdb` 管理字体栈（指定字体 → 系统字体 → Noto CJK） | ✅ 原生支持 |

**文本数据结构**：
```rust
struct TextBlock {
    // 原始文本（含标记）
    raw: String,
    // 解析后的文本段
    segments: Vec<TextSegment>,
    // 书写模式
    writing_mode: WritingMode,
    // 容器尺寸
    bounds: Rect,
    // 排版结果（由 cosmic-text 生成）
    layout: cosmic_text::Buffer,
    // Ruby 注音映射
    ruby_map: Vec<RubyAnnotation>,
    // 渐显状态
    reveal: RevealState,
}

enum TextSegment {
    Plain(String),
    Ruby { base: String, ruby: String },
    Color(Color),
    Size(f32),
    Bold,
    Italic,
    // 内联变量
    Variable(String),
    // 语音同步标记
    VoiceSync { char_index: usize, voice_file: String },
}

struct RevealState {
    // 已显示字符数（支持按字/按词）
    revealed_chars: usize,
    // 总字符数
    total_chars: usize,
    // 当前速度（字符/秒）
    speed: f32,
    // 是否等待点击
    waiting_click: bool,
}
```

#### 3.2.4 音频系统 (Audio)

```rust
struct AudioSystem {
    // BGM 通道（支持交叉淡入淡出）
    bgm: AudioChannel,
    // 语音通道（独占，播放时自动降低 BGM 音量）
    voice: AudioChannel,
    // SE 通道（多并发）
    se: Vec<AudioChannel>,
    // 环境音通道
    ambient: AudioChannel,
    // 主音量控制
    master_volume: Volume,
}

struct AudioChannel {
    sink: rodio::Sink,
    current: Option<AudioSource>,
    next: Option<AudioSource>,     // 用于无缝循环
    fade_state: FadeState,
    ducking: bool,                 // 被语音压制的状态
}

// 音频源（支持流式加载）
enum AudioSource {
    File { path: String, loop: bool },
    Memory { data: Vec<u8>, loop: bool },
}
```

**特性**：
- 语音播放时自动 Ducking（BGM 降至 30%）
- 无缝循环 BGM（预读下一段开头）
- 3D 定位音效（HRTF，用于环境沉浸）
- 音频可视化（波形/频谱，用于 UI 反馈）

#### 3.2.5 存档系统 (Save System)

**设计理念**：存档 = 虚拟机状态快照

```rust
// 完整游戏状态（可序列化）
#[derive(Serialize, Deserialize)]
struct GameState {
    // 元数据
    metadata: SaveMetadata,
    // 脚本执行状态
    script: ScriptState,
    // 场景图层状态
    scene: SceneState,
    // 音频状态
    audio: AudioState,
    // 变量表
    variables: VariableTable,
    // 历史记录
    history: HistoryLog,
    // 系统设置
    settings: GameSettings,
}

struct ScriptState {
    current_file: String,
    line_number: usize,
    call_stack: Vec<CallFrame>,
    // 指令队列中待执行的指令
    pending_commands: Vec<Command>,
}

struct SceneState {
    bg: Option<BgState>,
    characters: HashMap<String, CharState>,
    fx: Vec<FxState>,
    ui: UiState,
}

// 存档版本化，支持迁移
const SAVE_FORMAT_VERSION: u32 = 1;
```

**存档管理器**：
```rust
struct SaveManager {
    slots: Vec<Option<SaveData>>,  // 99 个存档位
    quicksave_slot: Option<SaveData>,
    autosave_slot: Option<SaveData>,
    // 缩略图生成（渲染当前画面到 320x180）
    thumbnail_renderer: ThumbnailRenderer,
}
```

#### 3.2.6 资源管理系统 (Asset Manager)

```rust
struct AssetManager {
    // 资源加载器注册表
    loaders: HashMap<AssetType, Box<dyn AssetLoader>>,
    // 已加载资源缓存（LRU）
    cache: LruCache<AssetId, LoadedAsset>,
    // 异步加载队列
    load_queue: LoadQueue,
    // 资源包（可选加密）
    pak_files: Vec<PakFile>,
}

enum AssetType {
    Texture,
    Audio,
    Script,
    Font,
    Live2DModel,
    Shader,
    Video,
}

// 资源引用（弱引用 + 自动卸载）
struct Handle<T> {
    id: AssetId,
    _phantom: PhantomData<T>,
}
```

**加载策略**：
- **预加载**：场景切换时预加载下一章资源
- **流式加载**：语音文件边播边读
- **内存预算**：设定上限，LRU 淘汰不活跃资源
- **异步解压**：`zstd` 解包不阻塞主线程

---

## 四、项目目录结构

```
suzu-framework/
├── Cargo.toml                    # Workspace 根配置
├── rust-toolchain.toml           # Rust 工具链锁定
├── .cargo/
│   └── config.toml               # 编译配置（target 别名、链接器优化）
│
├── crates/                       # 核心 Crate 分层
│   ├── suzu-core/                # 核心抽象层（平台无关）
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs          # 基础类型（Vec2, Color, Rect, Affine2）
│   │   │   ├── error.rs          # 错误类型定义
│   │   │   └── math.rs           # 数学工具
│   │   └── Cargo.toml
│   │
│   ├── suzu-script/              # 脚本系统
│   │   ├── src/
│   │   │   ├── lib.rs
││   │   │   ├── parser/           # DSL 解析器
│   │   │   │   ├── lexer.rs
│   │   │   │   ├── parser.rs
│   │   │   │   └── ast.rs
│   │   │   ├── vm/               # 虚拟机
│   │   │   │   ├── mod.rs
│   │   │   │   ├── executor.rs
│   │   │   │   ├── state.rs
│   │   │   │   └── commands.rs   # 指令定义
│   │   │   ├── lua/              # Lua 绑定
│   │   │   │   ├── mod.rs
│   │   │   │   └── bindings.rs
│   │   │   └── macros.rs         # 指令宏
│   │   └── Cargo.toml
│   │
│   ├── suzu-render/              # 渲染系统
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── renderer.rs       # 主渲染器
│   │   │   ├── context.rs        # 渲染上下文
│   │   │   ├── layer/            # 图层系统
│   │   │   │   ├── mod.rs
│   │   │   │   ├── sprite.rs
│   │   │   │   ├── text.rs
│   │   │   │   └── fx.rs
│   │   │   ├── pipeline/         # 渲染管线
│   │   │   │   ├── mod.rs
│   │   │   │   ├── 2d.rs
│   │   │   │   ├── postprocess.rs
│   │   │   │   └── transition.rs
│   │   │   ├── shader/           # 着色器管理
│   │   │   │   ├── mod.rs
│   │   │   │   ├── compiler.rs   # WGSL 编译/缓存
│   │   │   │   └── builtins/     # 内置 shader
│   │   │   │       ├── sprite.wgsl
│   │   │   │       ├── text.wgsl
│   │   │   │       ├── postprocess.wgsl
│   │   │   │       └── transition/
│   │   │   │           ├── crossfade.wgsl
│   │   │   │           ├── wipe.wgsl
│   │   │   │           └── pixelate.wgsl
│   │   │   ├── texture.rs        # GPU 纹理管理
│   │   │   └── tween.rs          # 动画插值
│   │   └── Cargo.toml
│   │
│   ├── suzu-text/                # 文本渲染（中文特化）
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── layout.rs         # 排版引擎
│   │   │   ├── vertical.rs       # 竖排布局
│   │   │   ├── ruby.rs           # Ruby 注音
│   │   │   ├── reveal.rs         # 逐字渐显
│   │   │   ├── font.rs           # 字体管理
│   │   │   └── shaping.rs        # 字形 shaping
│   │   └── Cargo.toml
│   │
│   ├── suzu-audio/               # 音频系统
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── mixer.rs          # 混音器
│   │   │   ├── channel.rs        # 音频通道
│   │   │   ├── source.rs         # 音频源
│   │   │   └── ducking.rs        # 自动压制
│   │   └── Cargo.toml
│   │
│   ├── suzu-asset/               # 资源管理
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── manager.rs
│   │   │   ├── loader.rs
│   │   │   ├── cache.rs
│   │   │   ├── pak.rs            # 资源包格式
│   │   │   └── types.rs
│   │   └── Cargo.toml
│   │
│   ├── suzu-save/                # 存档系统
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── state.rs          # 状态快照
│   │   │   ├── manager.rs
│   │   │   ├── thumbnail.rs      # 缩略图生成
│   │   │   └── migrate.rs        # 版本迁移
│   │   └── Cargo.toml
│   │
│   ├── suzu-input/               # 输入系统
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── keyboard.rs
│   │   │   ├── mouse.rs
│   │   │   ├── touch.rs
│   │   │   └── gesture.rs        # 手势识别
│   │   └── Cargo.toml
│   │
│   ├── suzu-platform/            # 平台抽象层
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── desktop.rs        # Windows/macOS/Linux
│   │   │   ├── mobile.rs         # Android/iOS
│   │   │   └── web.rs            # WebAssembly
│   │   └── Cargo.toml
│   │
│   └── suzu-app/                 # 应用层（整合所有模块）
│       ├── src/
│       │   ├── lib.rs
│       │   ├── app.rs            # 主应用循环
│       │   ├── scene.rs          # 场景管理
│       │   ├── config.rs         # 游戏配置
│       │   └── plugin.rs         # 插件系统
│       └── Cargo.toml
│
├── tools/                        # 开发工具
│   ├── suzu-packer/              # 资源打包工具
│   │   └── src/main.rs
│   ├── suzu-compiler/            # 脚本编译器（DSL -> 字节码）
│   │   └── src/main.rs
│   └── suzu-editor/              # （远期）可视化脚本编辑器
│       └── src/main.rs
│
├── examples/                     # 示例项目
│   ├── hello-world/              # 最小可运行示例
│   │   ├── script/
│   │   │   └── main.szs
│   │   ├── assets/
│   │   │   ├── bg/
│   │   │   ├── char/
│   │   │   └── audio/
│   │   └── main.rs
│   ├── demo-visual-novel/        # 完整演示
│   └── demo-live2d/              # Live2D 集成演示
│
├── tests/                        # 集成测试
│   ├── script_tests/
│   ├── render_tests/
│   └── save_tests/
│
├── docs/                         # 文档
│   ├── architecture.md           # 架构文档
│   ├── script-reference.md       # 脚本语言参考
│   ├── api-guide.md              # Rust API 指南
│   └── tutorial/                 # 教程
│       ├── 01-getting-started.md
│       ├── 02-script-basics.md
│       ├── 03-advanced-animations.md
│       └── 04-custom-shaders.md
│
├── assets/                       # 框架自带默认资源
│   ├── fonts/
│   │   └── NotoSansCJK-Regular.ttc
│   ├── shaders/
│   │   └── builtins/
│   └── textures/
│       └── default_white.png
│
└── .github/
    └── workflows/
        ├── ci.yml                # CI/CD
        └── release.yml           # 发布流程
```

---

## 五、开发路线图

### Phase 0：基础设施（Week 1-2）

| 任务 | 产出 | 验收标准 |
|------|------|---------|
| Workspace 搭建 | `Cargo.toml` workspace 配置 | `cargo check` 全 crate 通过 |
| CI/CD 配置 | GitHub Actions workflow | 每次 PR 自动跑 `cargo test` + `cargo clippy` |
| 基础类型库 | `suzu-core` crate | Vec2/Color/Rect/Affine2 实现 + 单元测试 |
| 错误处理框架 | 统一 Error 类型 | 支持 `thiserror` + `anyhow` 分层 |

### Phase 1：渲染核心（Week 3-6）

| 任务 | 产出 | 验收标准 |
|------|------|---------|
| wgpu 初始化 | `suzu-platform` 窗口 + GPU 设备 | 显示纯色背景窗口 |
| 2D 精灵渲染 | `SpriteLayer` | 加载 PNG 显示在屏幕指定位置 |
| 图层系统 | `LayerStack` | 多层叠加、透明度、混合模式 |
| 基础过渡动画 | CrossFade / FadeThroughColor | `@bg` 切换时平滑过渡 |
| Tween 系统 | 插值动画引擎 | 位置/缩放/旋转/透明度动画 |
| 后处理管线 | Bloom / 色调映射 | 可开关的全屏效果 |

### Phase 2：文本系统（Week 7-9）

| 任务 | 产出 | 验收标准 |
|------|------|---------|
| cosmic-text 集成 | `suzu-text` 基础排版 | 显示横排中文文本 |
| 竖排布局 | `WritingMode::VerticalRl` | 竖排文本正确显示，标点旋转 |
| Ruby 注音 | Ruby 布局引擎 | `<ruby>漢字<rt>かんじ</rt></ruby>` 正确渲染 |
| 逐字渐显 | `RevealState` | 文本按速度逐字出现，支持点击跳过 |
| 对话框 UI | `DialogLayer` | 带 speaker 名称、文本区、点击提示 |

### Phase 3：脚本与音频（Week 10-13）

| 任务 | 产出 | 验收标准 |
|------|------|---------|
| DSL 解析器 | `suzu-script` lexer + parser | 解析示例脚本无错误 |
| 虚拟机执行器 | `ScriptVM` | 顺序执行指令，支持 `@jump` `@label` |
| 核心指令集 | ~30 种基础指令 | `@bg` `@char` `@text` `@choice` `@if` `@anim` `@fx` |
| Lua 绑定 | `mlua` 集成 | Lua 脚本可调用框架 API |
| 音频系统 | `suzu-audio` | BGM 播放/淡入淡出/循环 |
| 语音同步 | 文本渐显与语音时间戳对齐 | 播放语音时文本同步出现 |

### Phase 4：系统功能（Week 14-16）

| 任务 | 产出 | 验收标准 |
|------|------|---------|
| 存档系统 | `SaveManager` | 保存/读取完整游戏状态，缩略图生成 |
| 资源管理器 | `AssetManager` | 异步加载、LRU 缓存、资源包读取 |
| 输入系统 | 键盘/鼠标/触摸 | 点击推进文本、右键菜单、滚轮历史 |
| 历史回溯 | `HistoryLog` | 可回看已读文本，点击跳转到对应位置 |
| 系统菜单 | 设置/存档/读档/退出 | 完整的游戏内菜单 |

### Phase 5： polish 与扩展（Week 17-20）

| 任务 | 产出 | 验收标准 |
|------|------|---------|
| 自定义 Shader | 用户可加载 WGSL | 文档 + 示例 shader |
| Live2D 集成 | Live2D Cubism SDK FFI | 角色立绘支持动态表情 |
| 移动端适配 | Android/iOS 构建 | 触摸手势、屏幕适配 |
| WebAssembly | WASM 构建 | 浏览器中运行基础示例 |
| 性能优化 | 性能基准测试 | 1080p 60fps，内存占用 < 200MB |
| 文档与示例 | 完整文档 + 3 个示例 | 新人可 30 分钟上手 |

---

## 六、关键技术决策记录 (ADR)

### ADR-001：为何不用 Bevy 而是自研渲染？

**决策**：基于 `wgpu` 自研渲染层，而非使用 Bevy 引擎。

**理由**：
1. **控制权**：Galgame 渲染模式高度特化（2D 图层合成），Bevy 的 3D ECS 架构是过度设计
2. **包体**：自研可控制依赖，最终包体目标 < 5MB；Bevy 最小构建也 > 10MB
3. **学习曲线**：Bevy 的 ECS 对 Galgame 开发者是认知负担，Retained Mode 图层栈更直观
4. **中文文本**：Bevy 的文本渲染不支持竖排与 Ruby，仍需自研

### ADR-002：DSL + Lua 双脚本策略

**决策**：原生 DSL 覆盖常用指令，Lua 覆盖扩展逻辑。

**理由**：
1. DSL 对编剧友好，类自然语言语法
2. Lua 对程序员友好，可写复杂逻辑（如小游戏、数值系统）
3. 避免 DSL 过度复杂化（不图灵完备到极致）
4. Lua 沙箱化，防止用户脚本破坏框架

### ADR-003：为何选择 `cosmic-text` 而非 `fontdue`？

**决策**：`cosmic-text` 为主，`fontdue` 为光栅化后备。

**理由**：
1. `cosmic-text` 基于 `swash` 提供完整的 shaping 管线，支持复杂脚本
2. `fontdue` 仅提供光栅化，无 shaping，不适合多语言混合文本
3. `cosmic-text` 的 `Buffer` 架构适合我们的图层渲染模式
4. 竖排支持需在上层自建，但 shaping 基础由 `cosmic-text` 提供

---

## 七、风险与缓解策略

| 风险 | 影响 | 缓解策略 |
|------|------|---------|
| `cosmic-text` 竖排支持不完善 | 高 | 预留自定义 shaping 路径，必要时直接调 HarfBuzz |
| wgpu Web 后端性能不足 | 中 | 桌面端为主，Web 为辅助分发渠道 |
| 资源包加密被破解 | 低 | 加密为可选功能，不承诺绝对安全 |
| 开发周期超预期 | 中 | Phase 1-4 为 MVP，Phase 5 可裁剪；每 Phase 结束可发布 |
| Lua 绑定性能瓶颈 | 中 | 热点路径（渲染指令）不走 Lua，仅逻辑层使用 |

---

## 八、性能目标

| 指标 | 目标值 | 测试场景 |
|------|--------|---------|
| 帧率 | 60 FPS (VSync) | 1080p，3 角色立绘 + 粒子特效 + 全屏后处理 |
| 内存占用 | < 200 MB | 加载完整章节资源后 |
| 启动时间 | < 2 秒 | 从点击到显示主菜单 |
| 存档大小 | < 500 KB | 包含缩略图的完整存档 |
| 脚本解析 | < 10 ms | 1000 行脚本文件 |
| 资源加载 | 无阻塞 | 场景切换时预加载，语音流式播放 |

---

## 九、开源与社区策略

- **许可证**：框架 MIT/Apache-2.0 双许可；示例游戏 CC-BY-4.0
- **发布节奏**：
  - `0.1.0`：Phase 2 结束（基础渲染 + 文本）
  - `0.2.0`：Phase 4 结束（完整 MVP，可做游戏）
  - `0.3.0`：Phase 5 结束（Live2D + 移动端 + Web）
  - `1.0.0`：生产就绪，API 稳定
- **社区建设**：
  - Discord / QQ 群技术支持
  - 每月发布开发日志
  - 接受脚本指令扩展的 PR

---

## 十、附录

### A. 参考项目
- **Koharu**：Rust 漫画翻译工具，含竖排 CJK 文本渲染实现
- **Ren'Py**：Galgame 行业标准，DSL 设计参考
- **KiriKiri/KAG3**：日式 Galgame 引擎，脚本语法参考
- **Bevy**：Rust 游戏引擎，wgpu 使用模式参考

### B. 术语表
| 术语 | 说明 |
|------|------|
| **Shaping** | 将字符序列转换为可渲染字形序列的过程（含连字、重排、变体选择） |
| **Tween** | 补间动画，在两个状态间自动插值 |
| **Retained Mode** | 保留模式渲染，维护场景图，每帧只更新变化部分 |
| **Ducking** | 音频自动压制，当高优先级音频播放时降低低优先级音频音量 |
| **WGSL** | WebGPU Shading Language，WebGPU 标准着色器语言 |

---

> **文档版本**：v1.1
> **最后更新**：2026-05-10
> **作者**：Project Suzu Team
