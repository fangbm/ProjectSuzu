# Project Suzu Starter Template

This template shows the low-friction Project Suzu layout. It can be opened by `suzu-player` or `suzu-launcher` without writing a Rust `main.rs`.

```powershell
cargo run -p suzu-player -- templates\starter-vn
cargo run -p suzu-player -- --check templates\starter-vn
```

Layout:

- `game.suzu.toml`: project title, entry script, window settings, asset roots, and optional packages.
- `scenario/main.szs`: the default script entry.
- `assets/`: images, audio, fonts, and generated manifests.
- `saves/`: runtime save data for local playtests.

The recommended authoring style for new writers is `syntax=indent`. Advanced Rust integration is still available through `SuzuApp`, but this template is meant to be a script-first project folder: edit scripts and resources, then run the folder.
