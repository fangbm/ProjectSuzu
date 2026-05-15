# Minimal VN Template

This template is the smallest practical Project Suzu desktop project. It loads `script/main.szs`, scans `assets/`, registers fallback textures when no image files are present, and starts the standard desktop runtime.

Run it from the repository root:

```powershell
cargo run --manifest-path templates\minimal-vn\Cargo.toml
```

Use it as a starting point by changing:

- `Cargo.toml`: package name and dependency paths.
- `script/main.szs`: labels, dialogue, choices, and commands.
- `src/main.rs`: title screen text, asset paths, fallback textures.
- `assets/`: images, audio, and manifests for your project.

The template is stored inside the Project Suzu repository and therefore uses local path dependencies. If you copy it outside the repository, update the `suzu-app` and `suzu-platform` dependency paths first.
