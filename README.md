# Project Suzu

[简体中文](README.zh-CN.md)

[![CI](https://github.com/fangbm/ProjectSuzu/actions/workflows/ci.yml/badge.svg)](https://github.com/fangbm/ProjectSuzu/actions/workflows/ci.yml)

![Project Suzu icon](assets/branding/Suzu_icon.png)

Project Suzu is a Rust visual novel framework based on the project plan in `docs/project-plan.md`.

The current repository includes a complete first framework slice with typed module boundaries for:

- core math and error types
- script AST, parser, and command queue
- retained-mode render layer descriptions and layer stack utilities
- post-processing configuration and user WGSL shader loading
- CJK-focused text model with ruby annotations and vertical layout data
- audio state, backend command sync, asset, save, input, platform, and app facades
- adapter boundaries for Live2D models and video playback
- optional Lua-backed script extension registration through `suzu-script`'s `lua` feature
- project branding assets documented in `assets/branding/README.md`

Input supports serializable trigger maps for keyboard and mouse bindings, with a default desktop map for confirm/cancel/selection controls. Audio state can be diffed into backend commands through the `AudioBackend` trait, with a built-in state backend for tests and headless integrations.

## Status

The desktop path creates a `winit` window, initializes a `wgpu` surface, uploads RGBA textures, renders sprite quads with tint, opacity, scale, rotation, horizontal flip, and blend modes, and rasterizes dialogue/choice text into GPU textures. Script files can be parsed and compiled into VM commands, and the app facade can advance scene/audio commands, BGM/voice playback commands, queued dialogue voice sync, timestamp-driven voice reveal plans, character show/update/hide commands with named or custom positions, sizes, and horizontal flips, message-box visibility commands, timed waits, subroutine calls, time-based move/zoom/fade tween animations, background crossfades, flash/quake visual effects, dialogue reveal timing, confirm input, auto mode, read-dialogue skip mode, backlog/history overlay with voice replay hooks, system menu actions, variable-gated choice branches with keyboard or wheel selection, and serializable game-state snapshots with optional RGBA thumbnails for autosave/slot save flows.

Desktop examples now start on a built-in title screen. The runtime title menu supports Start, Continue, Load, Settings, and Quit, and projects can enable it with `GameConfig.title_screen.enabled = true`.

| Feature | Status |
| --- | --- |
| `.szs` parser/compiler | Stable-ish |
| Runtime app facade | Experimental |
| Save/load snapshots | Experimental |
| Visual script editor | Preview |
| Plaintext XP3-backed asset loading | Preview |
| XP3 viewer and KRKR package scan | Preview |
| KRKR package scan mode | Limited preview |

XP3 support is limited to plaintext archive reading in the public repository. Applications may provide external XP3 plugin modules for packages they are authorized to process, but game-specific processors are intentionally kept out of this repository. See `docs/xp3-support.md` and `LEGAL.md`.

## Commands

```powershell
cargo run -p suzu-hello-world
cargo run -p suzu-branching-story
cargo run -p suzu-ui-save-load-demo
cargo run -p suzu-editor
start examples\web-browser-shell\index.html
cargo run -p suzu-bench -- 1000
cargo run -p suzu-compiler -- examples\hello-world\script\main.szs
cargo run -p suzu-packer -- examples\hello-world --output target\hello-world-assets.json
cargo run -p suzu-packer -- examples\hello-world --pack target\hello-world.suzupack
.\scripts\package-desktop.ps1 -Check
.\scripts\package-desktop.ps1
cargo test -p suzu-script --features lua
cargo test --workspace
```

On Windows, the desktop example binaries use the GUI subsystem, so launching them does not open an extra console window. CLI tools keep their console output; double-clicking `suzu-compiler` or `suzu-packer` shows usage and waits for Enter, while `suzu-bench` runs the default benchmark and waits for Enter.

`suzu-packer` emits a JSON asset manifest that can be registered through `AssetManager::register_manifest_file` or `SuzuApp::register_asset_manifest_file`, and can also write `.suzupack` archives with RLE compression, packed offsets, and checksum metadata. The asset manager supports synchronous texture loads, background texture loads, optional LRU texture caching, and package archive reads. `GameConfig` and `UserSettings` can be read from or written to JSON files for project and user preference persistence.
Script compile errors include line/column diagnostics, and unknown commands suggest the closest built-in command when possible. Scripts may declare `@script version=1`; the compiler validates the format version and exposes a migration entry point for future DSL upgrades.
User-facing documentation lives in `docs/user-guide.md`, `docs/scripting-reference.md`, `docs/xp3-support.md`, `docs/release-packaging.md`, `docs/developer-checks.md`, `docs/release-checklist.md`, and `docs/api-stability.md`. The visual script editor plan is tracked in `docs/visual-script-editor-development-plan.md`, and the first editor MVP can be launched with `cargo run -p suzu-editor`. Contribution, security, and legal notes live in `CONTRIBUTING.md`, `SECURITY.md`, and `LEGAL.md`. Project changes are summarized in `CHANGELOG.md`; licensing is `MIT OR Apache-2.0`, with dependency notices in `THIRD_PARTY_LICENSES.md`.
