# Changelog

## Unreleased

- Split the desktop platform layer, script compiler, and app runtime tests into focused modules for the v0.1.7 convergence pass.
- Added the first `examples/short-vn-demo` slice and planning documentation for the complete short VN proof.
- Updated scripting reference examples to match current `@anim` and `@fx` attribute names.
- Added parser front ends for `syntax=indent`, `syntax=braces`, and `syntax=markup` while keeping the existing classic `.szs` syntax as the default.

## 0.1.6 - 2026-05-15

- Added a getting-started guide and a minimal visual novel template for new projects.
- Improved GUI `--check` diagnostics so launcher, XP3 viewer, and editor report validated paths and archive/plugin status without opening windows.
- Made XP3 viewer archive indexing and entry previews run in the background to keep the UI responsive on large archives.
- Hid external XP3 processor console windows on Windows during preview and asset loading.
- Added text previews for decoded XP3 script/text entries with UTF-8, UTF-16, and Shift_JIS detection.
- Kept local project continuation context out of the Git repository.

## 0.1.5 - 2026-05-15

- Added a detailed framework usage guide covering project setup, scripts, assets, runtime APIs, tools, packaging, and troubleshooting.
- Split the runtime app facade into focused modules while keeping `SuzuApp`, `TitleMenuAction`, and `SystemMenuAction` public exports stable.
- Split launcher and XP3 viewer GUI entry points into smaller app/UI/helper modules and added headless `--check` commands for launcher, XP3 viewer, and editor.
- Added a workspace smoke-test crate covering script compilation, runtime progression, package archive loading, save/restore, plaintext XP3 loading, and KAG conversion.
- Added a minimal `suzu-packer` library entry and package archive registration through `AssetManager`.
- Added `cargo-about` third-party notices, API stability notes, branding guidance, stronger legal/security plugin guidance, and release package checks for trust documentation.
- Added XP3 external processor interface documentation for `suzu.xp3-plugin.v1`.
- Updated CI and release quality gates to run GUI check commands.

## 0.1.4 - 2026-05-14

- Merged experimental XP3 archive reader and XP3-backed asset loading.
- Added XP3 viewer and unified launcher preview tools.
- Added KRKR package scan and limited KAG-to-Suzu conversion experiments.
- Added explicit XP3 support boundaries and an external XP3 plugin hook.
- Expanded CI triggers to feature branches and version tags.
- Added release quality gates, legal guidance, and versioned Windows/Linux artifact preparation.
- Fixed workspace repository metadata.

## 0.1.3 - 2026-05-11

- Added the first visual script editor MVP with `suzu-editor-core` and the `suzu-editor` desktop tool.
- Added `.szs` import/export, graph diagnostics, project scanning, undo command primitives, node editing UI, and editor packaging.
- Updated release documentation so desktop packages include the visual script editor.

## 0.1.0 - 2026-05-11

- Created the Project Suzu Rust workspace with typed crates for app, script, render, text, audio, assets, save, input, and platform boundaries.
- Added desktop `winit`/`wgpu` rendering, retained sprite layers, transitions, tween animations, text rendering, post-process configuration, and WGSL shader loading.
- Added the `.szs` script parser/compiler, VM queue, labels, choices, variables, conditionals, calls, waits, save commands, message visibility, diagnostics, versioning, and optional Lua extension registration.
- Added dialogue reveal, history UI, ruby annotation data, vertical glyph layout, voice cue markers, and timestamp-driven voice reveal plans.
- Added audio channel state, fades, save snapshots, and backend command synchronization.
- Added save/load slots, thumbnails, autosave/quicksave support, config persistence, input maps, system menu, auto mode, skip mode, and read-state persistence.
- Added asset discovery, async texture loading, LRU cache, manifests, `.suzupack` archive writing/reading, compression metadata, and checksum validation.
- Added desktop examples, web shell, stress scene, benchmark CLI, local desktop packaging, GitHub CI/release workflows, and user/developer documentation.
- Added repository metadata for licensing, contribution guidance, security reporting, and release notes.
