# Contributing

Thanks for improving Project Suzu. This repository is organized as a Rust workspace, so keep changes scoped to the crate or tool that owns the behavior.

## Before You Start

- Read `docs/user-guide.md` for the project shape.
- Read `docs/scripting-reference.md` when changing script syntax or VM commands.
- Read `docs/developer-checks.md` before opening a pull request.
- Read `docs/release-checklist.md` before creating a tag release.
- Read `LEGAL.md` before working with XP3/KRKR tooling, external plugin modules, or third-party assets.

## Development Flow

1. Make focused changes.
2. Add or update tests near the changed behavior.
3. Run the required developer gate:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

4. If script extensions changed, also run:

```powershell
cargo test -p suzu-script --features lua
```

5. If release packaging changed, run:

```powershell
.\scripts\package-desktop.ps1 -Check
```

## Style Notes

- Prefer existing crate boundaries over adding cross-crate shortcuts.
- Keep public API additions serializable when they represent save data, script data, or config.
- Keep examples small and deterministic.
- Update `CHANGELOG.md` for user-visible behavior, tooling, packaging, or docs changes.

## Legal Boundaries

- Do not submit commercial game archives, proprietary scripts, third-party DLLs, reverse-engineered artifacts, or recognizable commercial game samples.
- Do not submit decryption keys, DRM bypasses, license-check bypasses, or game-specific XP3 processors.
- Keep XP3 plugin modules outside this repository unless they only process data the project has the right to redistribute.
