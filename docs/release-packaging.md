# Release Packaging

Project Suzu supports two release paths: local desktop packaging and GitHub tag releases.

## Local Desktop Package

Run:

```powershell
.\scripts\package-desktop.ps1
```

The script builds the workspace, copies example binaries and desktop tools including `suzu-player`, `suzu-launcher`, `suzu-editor`, and `suzu-xp3-viewer`, includes user documentation plus `templates/starter-vn` and `templates/minimal-vn`, packs hello-world and short-demo assets into `.suzupack`, writes JSON manifests, and creates `dist/project-suzu-desktop.zip`.
The package also includes the Project Suzu icon and branding notes, `CONTRIBUTING.md`, `SECURITY.md`, `LEGAL.md`, `LICENSE-MIT`, `LICENSE-APACHE`, `THIRD_PARTY_LICENSES.md`, and `CHANGELOG.md`.

Check package inputs without building:

```powershell
.\scripts\package-desktop.ps1 -Check
```

Custom output and asset root:

```powershell
.\scripts\package-desktop.ps1 -Output dist/my-build -AssetRoot examples/branching-story
```

## GitHub Release

`.github/workflows/release.yml` builds Linux and Windows artifacts on tags matching `v*`.

```powershell
git tag v0.2.1
git push origin v0.2.1
```

The workflow uploads per-platform archives containing tools, the zero-code player, the visual script editor, benchmark CLI, examples, packed hello-world and short-demo assets, README, legal notes, branding notes, third-party notices, the getting-started guide, framework guide, project layout guide, short-demo plan, low-friction template, minimal Rust template, XP3 interface docs, and core/developer documentation. Release asset filenames include the tag, for example `project-suzu-v0.2.1-windows-x64.tar.gz`.

Use `docs/release-checklist.md` as the final pre-tag checklist.

## Verification Before Release

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
cargo test -p suzu-script --features lua
cargo run -p suzu-launcher -- --check
cargo run -p suzu-player -- --check templates\starter-vn
cargo run -p suzu-xp3-viewer -- --check
cargo run -p suzu-editor -- --check
.\scripts\package-desktop.ps1 -Check
```
