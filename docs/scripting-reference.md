# Project Suzu Scripting Reference

Project Suzu scripts use `.szs` files. The default syntax is the classic line-oriented Project Suzu style: lines beginning with `#` set the current speaker, labels begin with `*`, and commands begin with `@`.

## Metadata

```text
@script version=1
```

The current script format version is `1`. Scripts may also specify a surface syntax:

```text
@script version=1 syntax=classic
@script version=1 syntax=indent
@script version=1 syntax=braces
@script version=1 syntax=markup
```

If `syntax` is omitted, Project Suzu uses `classic`. All styles compile into the same command model, so runtime behavior, saves, packaging, and asset loading stay shared.

## Choosing A Syntax

Use `syntax=indent` for new hand-written story scripts. It keeps the script readable for writers, maps directly onto choices and conditional branches, and is the style used by the recommended starter workflow.

Use `syntax=classic` when maintaining older scripts, writing low-level examples, or needing the most stable compatibility surface. Treat `syntax=braces` and `syntax=markup` as tool-facing front ends for generated scripts, import/export, and structured editors. During the editor MVP phase, Project Suzu aims for full editing support for `indent` and `classic`; `braces` and `markup` may be read or exported by tools before they receive full form-editing support.

## Syntax Styles

Use one style per file. Mixed style is only intended for small escape hatches, such as writing a classic `@custom` command inside an indent script while experimenting with custom extension commands.

| Style | Best For | Notes |
| --- | --- | --- |
| `indent` | Recommended new projects | Python-like blocks; indentation closes `if` blocks. |
| `classic` | Stable examples and compatibility | Original line-oriented Project Suzu syntax. |
| `braces` | Programmers and generators | C-like call statements and brace blocks; experimental for author editing. |
| `markup` | Editor export and structured tools | Tag syntax with quoted attributes; experimental for author editing. |

Classic style is the original `.szs` form:

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

Indent style is friendlier for authors who prefer Python-like blocks:

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

Brace style is useful for programmers and code generators:

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

Markup style is useful for editor export and tag-oriented workflows:

```html
<script version="1" syntax="markup" />
<scene>
  <bg file="school" method="crossfade" time="500" />
  <say speaker="Suzu">Hello from markup.</say>
  <choice text="Library" goto="library" />
  <label name="library" />
</scene>
```

The non-classic styles are parser front ends. They are intended for core VN flow commands first; custom or unusual commands can still use classic `@command` lines inside indent and markup documents.

### Style Rules

- `classic` labels use `*label`; `indent` can use `label name:`; `braces` can use `label("name");`; `markup` can use `<label name="name" />`.
- `classic` speaker lines use `# Speaker` followed by text; `indent` and `braces` also accept `Speaker: text`; `markup` uses `<say speaker="Speaker">text</say>`.
- `indent` closes `if` blocks when indentation returns to the parent level.
- `braces` closes `if` blocks with `}` and can pair `} else {` as expected.
- `markup` closes conditional blocks with `</if>` and can place `<else />` inside the block.
- All attribute names are the same across styles: `file`, `goto`, `cond`, `time`, `duration`, `type`, `name`, `face`, and so on.

Equivalent choice syntax:

```text
@choice "Library" goto=library cond=trust>=50
choice "Library" goto=library cond=trust>=50
choice("Library", goto=library, cond="trust>=50");
<choice text="Library" goto="library" cond="trust>=50" />
```

Equivalent condition syntax:

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

## Dialogue

```text
# Eileen
Hello.[l][r]This appears after a click wait.
```

Supported inline tags:

- `[l]`: click wait point.
- `[r]`: line break.
- `[ruby=text]base[/ruby]`: ruby annotation data.

## Scene Commands

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

`@bg` supports `method=crossfade`, `method=fade_color`, `method=fade-through-color`, and instant fallback when `method` is omitted or unknown. Character `face=neutral` maps to the texture id in `name`; other faces map to `name_face`, for example `name=eileen face=smile` uses `eileen_smile`.

## Flow Commands

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

## Variables

```text
@set name=affection value=52
```

The app facade stores values as booleans, numbers, or strings when possible.

## Audio Commands

```text
@playbgm file="music/theme.ogg" loop=true fadein=1000
@stopbgm fadeout=500
@playvoice file="voice/eileen_001.ogg" fadein=0
@voice file="voice/eileen_002.ogg"
@stopvoice fadeout=80
```

`@voice` attaches the voice file to the next dialogue line. `suzu-text` can build timestamp-driven reveal plans so displayed characters follow voice elapsed time.

## System Commands

```text
@wait time=750
@hidemsg
@showmsg
@savename text="Chapter 1"
@autosave
```

## Custom Commands

Register custom command names through `ExtensionRegistry`:

```rust
let mut registry = suzu_script::ExtensionRegistry::new();
registry.register_command_name("shakeui");
let document = suzu_script::parse_script("@shakeui \"dialogue\" power=8");
let commands = suzu_script::compile_document_with_extensions(&document, Some(&registry))?;
```

With the `lua` feature enabled, Lua can return a command-name list:

```rust
registry.register_lua_command_list(r#"return { "shakeui", "unlock_gallery" }"#)?;
```
