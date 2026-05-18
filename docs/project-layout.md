# Project Layout

Project Suzu now supports a script-first project folder for authors who want to write scripts and place resources without creating a Rust binary.

## Standard Folder

```text
my-game/
  game.suzu.toml
  scenario/main.szs
  assets/
    bg/
    chara/
    bgm/
    voice/
  saves/
```

`scenario/main.szs` is the default entry. If `game.suzu.toml` is absent, `suzu-player` and `suzu-launcher` still try `scenario/main.szs`, then `script/main.szs` for older templates.

## game.suzu.toml

```toml
title = "My Game"
subtitle = "A Project Suzu visual novel"
entry = "scenario/main.szs"

[title_screen]
enabled = true
background = "title_bg"

[window]
title = "My Game"
width = 1280
height = 720
resizable = true

[assets]
roots = ["assets"]

[package]
files = ["data.suzupack"]
```

The `package.files` list is optional. Use it when the project has generated `.suzupack` files. Keep it empty while working directly from loose files.

## Resource IDs

Loose resources are registered from each configured asset root:

- `assets/bg/school.png` becomes `bg/school`.
- `assets/chara/suzu_normal.png` becomes `chara/suzu_normal`.
- Texture files also receive a short file-stem alias, such as `school` or `suzu_normal`, for compact scripts.

The path-based ID is best for larger projects because it avoids collisions. The short alias is meant for small projects and script-first quick prototypes.

## Running And Checking

```powershell
suzu-player my-game
suzu-player --check my-game
suzu-launcher --check --project-root my-game
suzu-editor --check --project-root my-game
```

When `suzu-player` starts without a project root and the current folder is not a Suzu project, it looks for the bundled `templates/starter-vn` project beside the executable or under the current folder. This makes double-clicking `suzu-player` in a release package open the included starter project instead of failing on `.\scenario\main.szs`.

During development, prefer `syntax=indent` in new scripts:

```text
@script version=1 syntax=indent
bg school
Suzu: Hello.
choice "Library" goto=library
label library:
Suzu: Let us begin.
```

This layout does not make Project Suzu a full KRKR/TJS/KAG runtime. KRKR package scanning and KAG conversion remain migration tools; new projects should target `.szs` directly.
