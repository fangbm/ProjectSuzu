# Project Suzu User Guide

Project Suzu is a Rust visual novel framework. It provides a script compiler, a retained scene model, text reveal and history, save/load state, audio state synchronization, resource packing, and desktop examples.

For a first project walkthrough, see `docs/getting-started.md`. For a full end-to-end developer guide, see `docs/framework-guide.md`.

## Quick Start

Run the hello-world example:

```powershell
cargo run -p suzu-hello-world
```

Run the minimal project template:

```powershell
cargo run --manifest-path templates\minimal-vn\Cargo.toml
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

Launch the unified project entry app:

```powershell
cargo run -p suzu-launcher
```

Inspect and preview a KiriKiri XP3 archive:

```powershell
cargo run -p suzu-xp3-viewer -- D:\game\data.xp3
```

## Project Layout

- `crates/suzu-app`: high-level visual novel app facade.
- `crates/suzu-script`: DSL parser, compiler, VM command queue, diagnostics, and extension registration.
- `crates/suzu-render`: render layer data, transitions, post-process configuration, and shader loading.
- `crates/suzu-text`: markup normalization, reveal state, ruby data, vertical layout, and voice reveal sync.
- `crates/suzu-audio`: audio channel state, fades, snapshots, and backend command synchronization.
- `crates/suzu-save`: JSON save slots, quicksave, autosave, thumbnails, history, and audio state.
- `crates/suzu-asset`: texture discovery, async loading, LRU cache, manifests, `.suzupack` archive reads, and experimental KiriKiri XP3 archive reads.
- `crates/suzu-input`: keyboard, mouse, wheel, and selection trigger maps.
- `crates/suzu-platform`: desktop `winit`/`wgpu` integration and platform configuration types.
- `crates/suzu-editor-core`: visual script editor document model, import/export, graph diagnostics, project scan, and undo commands.
- `tools/suzu-launcher`: unified desktop entry app for opening Suzu projects, importing XP3 archives, and running previews.
- `tools/suzu-xp3-viewer`: desktop XP3 inspection and image/text preview tool.

## Runtime Flow

1. Parse `.szs` source into a `ScriptDocument`. The parser detects `syntax=classic`, `syntax=indent`, `syntax=braces`, or `syntax=markup` from the script header.
2. Compile the document into VM commands.
3. Feed commands into `SuzuApp`.
4. Optionally show the title screen when `GameConfig.title_screen.enabled` is true.
5. Advance the app with frame deltas and input events.
6. Render the app scene using the platform renderer.
7. Capture saves through the save manager.

## XP3 Resources

`suzu-asset` includes experimental KiriKiri XP3 archive parsing through `Xp3Archive`. The reader indexes XP3 `File` entries, extracts stored or zlib-compressed segments, and can be registered directly into `AssetManager`:

```rust
let mut app = SuzuApp::default();
app.register_xp3_file("data.xp3")?;
app.load_script_asset("main")?;
```

When an XP3 is registered, supported entries are exposed as normal assets by file stem. For example, `scenario/main.szs` becomes the script asset `main`, and `image/bg_school.png` becomes the texture asset `bg_school`. Scripts can then reference those ids normally, such as `@bg file="bg_school"`.

Protected XP3 entries are indexed for reporting, but Project Suzu only reads plaintext entries by default. Projects that own a compatible external XP3 processor can pass it through the plugin hook:

```rust
use suzu_asset::Xp3PluginModule;

let module = Xp3PluginModule::from_json_file("xp3-plugin.json")?;
app.register_xp3_file_with_options("data.xp3", module.xp3_options())?;
```

The repository does not include game-specific XP3 processors or proprietary handling rules. Those should live outside the public repository and be supplied by the application that has the right to use them.

For manual testing, run `suzu-xp3-viewer` with an XP3 path. It lists indexed entries, marks protected entries, previews plaintext image assets, and previews UTF-8 script/text files. Select a `.szs` script entry and press Start Game to register the XP3, load the script, and run an embedded game preview.

## Title Screen

Set `GameConfig.title_screen.enabled = true` to start on a title menu instead of immediately advancing the script. The built-in title menu supports Start, Continue, Load, Settings, and Quit. Start resets the runtime and advances the script to the first waiting point; Continue restores the autosave or slot 0 when available; Load opens a small load page for autosave plus the first five slots; Settings opens a title settings page for text speed, auto-advance delay, and master volume. `TitleScreenConfig.background_texture` can point at a registered texture id for a branded title background, and `TitleScreenConfig.labels` can override menu text. Return Title in the system menu resets the runtime and shows the title screen again.

## Examples

- `examples/hello-world`: minimal script, title screen, and asset packing flow.
- `examples/branching-story`: title screen, choices, labels, and conditional variables.
- `examples/ui-save-load-demo`: title screen, save/load, settings, history, and menu flows.
- `examples/short-vn-demo`: first complete short VN slice covering title screen, choices, variables, autosave, effects, and packaging.
- `examples/stress-scene`: script-level stress scene for benchmark inputs.
- `examples/web-browser-shell`: static browser canvas shell for future Wasm bundles.

## Script Writing Styles

Classic `.szs` remains the default:

```text
@script version=1
@bg file="school"
# Suzu
Hello.
```

For hand-written stories, the indentation style is often easier to scan:

```text
@script version=1 syntax=indent
bg file="school"
Suzu: Hello.
choice "Library" goto=library
label library:
Suzu: Let us begin.
```

Programmatic tools can choose `syntax=braces` or `syntax=markup` when that shape is easier to generate. See `docs/scripting-reference.md` for the full style guide and equivalent examples.

## Visual Script Editor

The planned visual script editor is documented in `docs/visual-script-editor-development-plan.md`. It covers the editor MVP scope, native Rust desktop architecture, `.szs` import/export model, node graph design, resource picker, diagnostics, preview workflow, tests, and development milestones.

The initial editor binary is available as `suzu-editor`. It can scan a Project Suzu folder, open `.szs` files, show imported visual nodes, edit common node fields, export back to `.szs`, and run graph/compile diagnostics.
