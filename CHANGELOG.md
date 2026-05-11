# Changelog

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
