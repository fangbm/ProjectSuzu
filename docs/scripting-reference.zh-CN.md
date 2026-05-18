# Project Suzu 脚本参考

Project Suzu 脚本使用 `.szs` 文件。默认语法是 classic 行式 Project Suzu 风格：以 `#` 开头的行设置当前说话人，标签以 `*` 开头，命令以 `@` 开头。

## 元数据

```text
@script version=1
```

当前脚本格式版本是 `1`。脚本也可以指定表层语法：

```text
@script version=1 syntax=classic
@script version=1 syntax=indent
@script version=1 syntax=braces
@script version=1 syntax=markup
```

如果省略 `syntax`，Project Suzu 使用 `classic`。所有风格都会编译到同一套命令模型，因此运行时行为、存档、打包和资源加载保持一致。

## 如何选择语法

新手写剧情脚本优先使用 `syntax=indent`。它对作者更易读，能自然表达选择项和条件分支，也是推荐 starter 工作流采用的语法。

维护旧脚本、编写底层示例或需要最稳定兼容面时使用 `syntax=classic`。`syntax=braces` 和 `syntax=markup` 定位为工具生成、导入导出和结构化编辑器的前端。编辑器 MVP 阶段优先完整支持 `indent` 和 `classic` 的编辑；`braces` 和 `markup` 可以先作为只读、导入或导出格式存在，之后再补齐完整表单编辑。

## 语法风格

每个文件建议只使用一种风格。混用只适合小型 escape hatch，例如在 indent 脚本中临时写 classic `@custom` 命令来实验自定义扩展命令。

| 风格 | 适合场景 | 说明 |
| --- | --- | --- |
| `indent` | 推荐新项目 | 类 Python block；缩进结束 `if` block。 |
| `classic` | 稳定示例和兼容性 | Project Suzu 原始行式语法。 |
| `braces` | 程序员和生成器 | 类 C call statement 和 brace block；作者编辑仍是实验定位。 |
| `markup` | 编辑器导出和结构化工具 | 带 quoted attribute 的 tag 语法；作者编辑仍是实验定位。 |

Classic 风格是原始 `.szs` 形式：

```text
@script version=1 syntax=classic
@bg file="school" method=crossfade time=500
# Suzu
Hello.[l][r]
@choice "Library" goto=library
*library
# Suzu
This is the same runtime route.
```

Indent 风格更适合喜欢类 Python block 的作者：

```text
@script version=1 syntax=indent
bg file="school" method=crossfade time=500
Suzu: Hello.[l][r]
choice "Library" goto=library
label library:
if cond=flag:
    Suzu: The route is open.
else:
    Suzu: The route is closed.
```

Brace 风格适合程序员和代码生成器：

```text
script(version=1, syntax=braces);
bg(file="school", method=crossfade, time=500);
Suzu: Hello from braces;
choice("Library", goto=library);
label("library");
if(cond=flag) {
    Suzu: The route is open;
} else {
    Suzu: The route is closed;
}
```

Markup 风格适合编辑器导出和 tag-oriented workflow：

```html
<script version="1" syntax="markup" />
<scene>
  <bg file="school" method="crossfade" time="500" />
  <say speaker="Suzu">Hello from markup.</say>
  <choice text="Library" goto="library" />
  <label name="library" />
</scene>
```

非 classic 风格是 parser front end。它们优先覆盖核心 VN flow 命令；custom 或不常见命令仍可在 indent 和 markup 文档中使用 classic `@command` 行。

### 风格规则

- `classic` 标签使用 `*label`；`indent` 可用 `label name:`；`braces` 可用 `label("name");`；`markup` 可用 `<label name="name" />`。
- `classic` 说话人行使用 `# Speaker` 后接文本；`indent` 和 `braces` 也接受 `Speaker: text`；`markup` 使用 `<say speaker="Speaker">text</say>`。
- `indent` 在缩进回到父级时关闭 `if` block。
- `braces` 用 `}` 关闭 `if` block，并支持常见的 `} else {`。
- `markup` 用 `</if>` 关闭条件 block，并可在其中放置 `<else />`。
- 各风格的 attribute name 保持一致：`file`、`goto`、`cond`、`time`、`duration`、`type`、`name`、`face` 等。

等价选择项语法：

```text
@choice "Library" goto=library cond=trust>=50
choice "Library" goto=library cond=trust>=50
choice("Library", goto=library, cond="trust>=50");
<choice text="Library" goto="library" cond="trust>=50" />
```

等价条件语法：

```text
@if cond=trust>=50
Unlocked route.
@else
Locked route.
@endif
```

```text
if cond=trust>=50:
    Unlocked route.
else:
    Locked route.
```

```text
if(cond="trust>=50") {
    Unlocked route;
} else {
    Locked route;
}
```

```html
<if cond="trust>=50">
  Unlocked route.
  <else />
  Locked route.
</if>
```

## 对白

```text
# Eileen
Hello.[l][r]This appears after a click wait.
```

支持的内联标签：

- `[l]`：点击等待点。
- `[r]`：换行。
- `[ruby=text]base[/ruby]`：ruby 注音数据。

## 场景命令

```text
@bg file="classroom.png" method=crossfade time=500
@bg file="platform.png" method=fade_color color=#101828 time=800
@char name=eileen face=smile pos=left layer=10 flip=false
@char name=eileen face=smile x=460 y=32 width=360 height=720 layer=10
@hidechar name=eileen
@anim target=eileen type=move_to x=520 y=32 duration=500
@anim target=eileen type=fade opacity=0 duration=400
@fx type=flash color=#ffffff duration=120
@fx type=quake intensity=8 duration=300
```

`@bg` 支持 `method=crossfade`、`method=fade_color`、`method=fade-through-color`；省略或未知 `method` 时使用 instant fallback。角色 `face=neutral` 映射到 `name` 的 texture id；其他 face 映射到 `name_face`，例如 `name=eileen face=smile` 使用 `eileen_smile`。

## 流程命令

```text
*start
@choice "Go home" goto=home cond=affection>=10
@jump goto=start
@call goto=common
@return
@if cond=affection>=50
Unlocked text
@else
Fallback text
@endif
```

## 变量

```text
@set name=affection value=52
```

App facade 会尽量把值存为 boolean、number 或 string。

## 音频命令

```text
@playbgm file="music/theme.ogg" loop=true fadein=1000
@stopbgm fadeout=500
@playvoice file="voice/eileen_001.ogg" fadein=0
@voice file="voice/eileen_002.ogg"
@stopvoice fadeout=80
```

`@voice` 会把语音文件绑定到下一行对白。`suzu-text` 可以构建基于 timestamp 的 reveal plan，让显示字符跟随语音播放时间。

## 系统命令

```text
@wait time=750
@hidemsg
@showmsg
@savename text="Chapter 1"
@autosave
```

## 自定义命令

通过 `ExtensionRegistry` 注册自定义命令名：

```rust
let mut registry = suzu_script::ExtensionRegistry::new();
registry.register_command_name("shakeui");
let document = suzu_script::parse_script("@shakeui \"dialogue\" power=8");
let commands = suzu_script::compile_document_with_extensions(&document, Some(&registry))?;
```

启用 `lua` feature 后，Lua 可以返回 command-name list：

```rust
registry.register_lua_command_list(r#"return { "shakeui", "unlock_gallery" }"#)?;
```
