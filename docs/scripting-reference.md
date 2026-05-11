# Project Suzu Scripting Reference

Project Suzu scripts use `.szs` files. Lines beginning with `#` set the current speaker, labels begin with `*`, and commands begin with `@`.

## Metadata

```text
@script version=1
```

The current script format version is `1`.

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
@char name=eileen face=smile pos=left layer=10 flip=false
@hidechar name=eileen
@anim target=eileen kind=shake intensity=8 time=300
@fx kind=flash color=#ffffff time=120
```

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
