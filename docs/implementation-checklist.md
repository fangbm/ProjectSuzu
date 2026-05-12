# Project Suzu Implementation Checklist

This checklist tracks the work completed to turn Project Suzu into a complete first visual novel framework slice. Status markers:

- `[x]` implemented and covered by tests
- `[~]` partially implemented, usable but incomplete
- `[ ]` not implemented yet

## Phase 0 - Infrastructure

- [x] Rust workspace with core crates, tools, and example project.
- [x] CI workflow for format, clippy, and tests.
- [x] Core math and error types: `Vec2`, `Color`, `Rect`, `Affine2`.
- [x] Typed module boundaries for script, render, text, audio, asset, save, input, platform, and app.
- [x] Release workflow builds desktop artifacts for Linux, Windows, and macOS on tags.

## Phase 1 - Rendering Core

- [x] Desktop `winit` window and `wgpu` surface/device initialization.
- [x] Sprite rendering with texture upload, tint, opacity, scale, rotation, and horizontal flip.
- [x] Blend modes: normal, add, multiply, screen.
- [x] Background transitions: instant, crossfade, fade-through-color.
- [x] Tween animations: move, zoom, shake, fade.
- [x] Frame-level text rendering through `cosmic-text`.
- [x] Retained layer model via `SpriteLayer` and a reusable `LayerStack` API.
- [x] Post-processing pipeline configuration: bloom, tone mapping, and user toggles.
- [x] User-defined WGSL shader loading and examples.

## Phase 2 - Text System

- [x] CJK horizontal text rendering and typewriter reveal.
- [x] Inline control tags: `[l]` click wait and `[r]` line break.
- [x] Dialogue wait/skip behavior on confirm input.
- [x] Dialogue history text normalization.
- [x] `WritingMode::VerticalRl` type and vertical glyph layout are implemented.
- [x] Ruby parser and annotation layout data are implemented.
- [x] Dialogue box rendering includes configurable style, speaker name area, and click prompt text.
- [x] Backlog/history UI with scroll and replay voice hooks.

## Phase 3 - Script And Audio

- [x] DSL parser for comments, speakers, labels, commands, quoted args, and inline comments.
- [x] VM command queue with labels, jump, call, return, insertion, and saved call stack.
- [x] Core script commands: `@bg`, `@char`, `@hidechar`, `@anim`, `@fx`, `@choice`, `@if/@else/@endif`, `@set`, `@jump`, `@call`, `@return`, `@wait`, `@savename`, `@autosave`, `@hidemsg`, `@showmsg`.
- [x] Character controls: face texture selection, position, size, layer, flip, show/update/hide.
- [x] Audio state model: BGM and voice channels with fade in/out and save snapshots.
- [x] Dialogue voice cue command: `@voice` binds the next text line to `VoiceSync`.
- [x] Audio backend interface and state backend command sync are implemented; `rodio`/`cpal` adapters can plug in through the backend trait.
- [x] Voice sync supports timestamp-driven reveal plans and applying voice elapsed time to dialogue reveal state.
- [x] Lua extension layer with optional `mlua` command-list bindings and custom command registration.
- [x] Script diagnostics with source spans, line/column errors, and command suggestions.
- [x] Script format versioning and migration rules.

## Phase 4 - System Features

- [x] Save manager with slots, quicksave, autosave, JSON read/write, script position, call stack, scene, variables, history, and audio state.
- [x] Asset manager can register textures, recursively discover PNG/JPEG/WebP files, register package manifests, load textures asynchronously, and cache textures with LRU eviction.
- [x] Input maps keyboard, mouse, wheel, and selection events on desktop, with configurable trigger bindings.
- [x] Save thumbnails.
- [x] Async asset loading and LRU cache.
- [x] Resource package manifest format, archive reader, compression metadata, and checksum validation.
- [x] `suzu-packer` CLI can scan an asset root and emit a sorted JSON manifest.
- [x] `suzu-packer` archive writing with compression, checksums, and package reader support.
- [x] System menu: settings, save, load, history, return title, quit.
- [x] Auto mode, skip mode for read dialogue, read-state persistence, and configurable text speed.
- [x] Config persistence for project window/script settings and user audio/text/window settings, with system menu integration points.

## Phase 5 - Platform, Polish, And Examples

- [x] Full desktop release packaging with local PowerShell bundle script and GitHub release workflow.
- [x] Android/iOS touch input abstractions and build target descriptors.
- [x] WebAssembly build target descriptor and browser example shell.
- [x] Live2D integration adapter boundary.
- [x] Video playback adapter boundary.
- [x] Performance benchmark CLI and stress scene script.
- [x] Complete user documentation for getting started, scripting, and release packaging.
- [x] Three examples: minimal hello-world, branching story, and full UI/save/load demo.

## Phase 6 - Visual Script Editor

- [x] Editor development plan covering MVP scope, architecture, document model, UI, diagnostics, preview, tests, and milestones.
- [x] `suzu-editor-core` crate with editor document model, `.szs` import/export, graph diagnostics, project scan, and undo command primitives.
- [x] Initial `suzu-editor` desktop binary with project scan, script open, visual node list, basic node inspector, export/save, and diagnostics panel.
- [x] Release packaging includes `suzu-editor`, `suzu-xp3-viewer`, and editor planning documentation.
- [ ] Rich node forms for all built-in commands.
- [ ] Branch graph visualization with editable edges.
- [ ] Asset picker previews for images and audio.
- [ ] Embedded or companion runtime preview from selected node.
- [ ] Editor sidecar `.editor.json` layout persistence.
- [ ] Golden file fixtures for import/export equivalence.

## Current Verification Gate

Run these after each completed slice:

```powershell
C:\Users\方便面\.cargo\bin\cargo.exe fmt --all -- --check
C:\Users\方便面\.cargo\bin\cargo.exe clippy --workspace --all-targets -- -D warnings
C:\Users\方便面\.cargo\bin\cargo.exe test --workspace
```
