# Release Packaging

Project Suzu supports two release paths: local desktop packaging and GitHub tag releases.

## Local Desktop Package

Run:

```powershell
.\scripts\package-desktop.ps1
```

The script builds the workspace, copies example binaries and desktop tools including `suzu-launcher`, `suzu-editor`, and `suzu-xp3-viewer`, includes user documentation, packs hello-world assets into `.suzupack`, writes a JSON manifest, and creates `dist/project-suzu-desktop.zip`.
The package also includes the Project Suzu icon, `CONTRIBUTING.md`, `SECURITY.md`, `LEGAL.md`, `LICENSE-MIT`, `LICENSE-APACHE`, and `CHANGELOG.md`.

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
git tag v0.1.4
git push origin v0.1.4
```

The workflow uploads per-platform archives containing tools, the visual script editor, benchmark CLI, examples, packed hello-world assets, README, legal notes, and core/developer documentation. Release asset filenames include the tag, for example `project-suzu-v0.1.4-windows-x64.tar.gz`.

Use `docs/release-checklist.md` as the final pre-tag checklist.

## Verification Before Release

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
cargo test -p suzu-script --features lua
.\scripts\package-desktop.ps1 -Check
```
