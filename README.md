# Project Suzu

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
- project branding asset in `assets/branding/Suzu_icon.png`

Input supports serializable trigger maps for keyboard and mouse bindings, with a default desktop map for confirm/cancel/selection controls. Audio state can be diffed into backend commands through the `AudioBackend` trait, with a built-in state backend for tests and headless integrations.

## Status

The desktop path creates a `winit` window, initializes a `wgpu` surface, uploads RGBA textures, renders sprite quads with tint, opacity, scale, rotation, horizontal flip, and blend modes, and rasterizes dialogue/choice text into GPU textures. Script files can be parsed and compiled into VM commands, and the app facade can advance scene/audio commands, BGM/voice playback commands, queued dialogue voice sync, timestamp-driven voice reveal plans, character show/update/hide commands with named or custom positions, sizes, and horizontal flips, message-box visibility commands, timed waits, subroutine calls, time-based move/zoom/fade tween animations, background crossfades, flash/quake visual effects, dialogue reveal timing, confirm input, auto mode, read-dialogue skip mode, backlog/history overlay with voice replay hooks, system menu actions, variable-gated choice branches with keyboard or wheel selection, and serializable game-state snapshots with optional RGBA thumbnails for autosave/slot save flows.

## Commands

```powershell
cargo run -p suzu-hello-world
cargo run -p suzu-branching-story
cargo run -p suzu-ui-save-load-demo
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

`suzu-packer` emits a JSON asset manifest that can be registered through `AssetManager::register_manifest_file` or `SuzuApp::register_asset_manifest_file`, and can also write `.suzupack` archives with RLE compression, packed offsets, and checksum metadata. The asset manager supports synchronous texture loads, background texture loads, optional LRU texture caching, and package archive reads. `GameConfig` and `UserSettings` can be read from or written to JSON files for project and user preference persistence.
Script compile errors include line/column diagnostics, and unknown commands suggest the closest built-in command when possible. Scripts may declare `@script version=1`; the compiler validates the format version and exposes a migration entry point for future DSL upgrades.
User-facing documentation lives in `docs/user-guide.md`, `docs/scripting-reference.md`, `docs/release-packaging.md`, `docs/developer-checks.md`, and `docs/release-checklist.md`. Contribution and security notes live in `CONTRIBUTING.md` and `SECURITY.md`. Project changes are summarized in `CHANGELOG.md`; licensing is `MIT OR Apache-2.0`.
