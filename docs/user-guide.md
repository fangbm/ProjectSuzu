# Project Suzu User Guide

Project Suzu is a Rust visual novel framework. It provides a script compiler, a retained scene model, text reveal and history, save/load state, audio state synchronization, resource packing, and desktop examples.

## Quick Start

Run the hello-world example:

```powershell
cargo run -p suzu-hello-world
```

Compile a script:

```powershell
cargo run -p suzu-compiler -- examples\hello-world\script\main.szs
```

Pack assets into an archive:

```powershell
cargo run -p suzu-packer -- examples\hello-world --pack target\hello-world.suzupack
```

Run all verification gates:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p suzu-script --features lua
.\scripts\package-desktop.ps1 -Check
```

Launch the first visual script editor MVP:

```powershell
cargo run -p suzu-editor
```

## Project Layout

- `crates/suzu-app`: high-level visual novel app facade.
- `crates/suzu-script`: DSL parser, compiler, VM command queue, diagnostics, and extension registration.
- `crates/suzu-render`: render layer data, transitions, post-process configuration, and shader loading.
- `crates/suzu-text`: markup normalization, reveal state, ruby data, vertical layout, and voice reveal sync.
- `crates/suzu-audio`: audio channel state, fades, snapshots, and backend command synchronization.
- `crates/suzu-save`: JSON save slots, quicksave, autosave, thumbnails, history, and audio state.
- `crates/suzu-asset`: texture discovery, async loading, LRU cache, manifests, and `.suzupack` archive reads.
- `crates/suzu-input`: keyboard, mouse, wheel, and selection trigger maps.
- `crates/suzu-platform`: desktop `winit`/`wgpu` integration and platform configuration types.
- `crates/suzu-editor-core`: visual script editor document model, import/export, graph diagnostics, project scan, and undo commands.

## Runtime Flow

1. Parse `.szs` source into a `ScriptDocument`.
2. Compile the document into VM commands.
3. Feed commands into `SuzuApp`.
4. Optionally show the title screen when `GameConfig.title_screen.enabled` is true.
5. Advance the app with frame deltas and input events.
6. Render the app scene using the platform renderer.
7. Capture saves through the save manager.

## Title Screen

Set `GameConfig.title_screen.enabled = true` to start on a title menu instead of immediately advancing the script. The built-in title menu supports Start, Continue, Load, Settings, and Quit. Start resets the runtime and advances the script to the first waiting point; Continue restores the autosave or slot 0 when available; Load restores slot 0; Return Title in the system menu resets the runtime and shows the title screen again.

## Examples

- `examples/hello-world`: minimal script, title screen, and asset packing flow.
- `examples/branching-story`: title screen, choices, labels, and conditional variables.
- `examples/ui-save-load-demo`: title screen, save/load, settings, history, and menu flows.
- `examples/stress-scene`: script-level stress scene for benchmark inputs.
- `examples/web-browser-shell`: static browser canvas shell for future Wasm bundles.

## Visual Script Editor

The planned visual script editor is documented in `docs/visual-script-editor-development-plan.md`. It covers the editor MVP scope, native Rust desktop architecture, `.szs` import/export model, node graph design, resource picker, diagnostics, preview workflow, tests, and development milestones.

The initial editor binary is available as `suzu-editor`. It can scan a Project Suzu folder, open `.szs` files, show imported visual nodes, edit common node fields, export back to `.szs`, and run graph/compile diagnostics.
