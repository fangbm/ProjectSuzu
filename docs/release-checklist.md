# Release Checklist

Use this checklist before creating a `v*` tag or running a manual GitHub release workflow.

## Version Notes

- [ ] Update `CHANGELOG.md`.
- [ ] Confirm crate versions are intentional.
- [ ] Confirm `README.md` describes the current feature set.
- [ ] Confirm `docs/framework-guide.md` reflects the current project workflow.
- [ ] Confirm `docs/getting-started.md`, `docs/project-layout.md`, and `templates/starter-vn` still work as the recommended new-user entry point.
- [ ] Confirm `LEGAL.md`, XP3 support boundaries, and `docs/xp3-plugin-interface.md` are included.
- [ ] Regenerate and review `THIRD_PARTY_LICENSES.md` after dependency changes.
- [ ] Confirm `assets/branding/README.md` and `docs/api-stability.md` are current.

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo doc --workspace --no-deps`
- [ ] `cargo test -p suzu-script --features lua`
- [ ] `cargo run -p suzu-launcher -- --check`
- [ ] `cargo run -p suzu-player -- --check templates\starter-vn`
- [ ] `cargo run -p suzu-xp3-viewer -- --check`
- [ ] `cargo run -p suzu-editor -- --check`
- [ ] `cargo run -p suzu-bench -- 100`
- [ ] `.\scripts\package-desktop.ps1 -Check`

## Local Package

- [ ] `.\scripts\package-desktop.ps1`
- [ ] Confirm `dist/project-suzu-desktop.zip` contains tools, examples, docs, licenses, third-party notices, branding notes, changelog, and packed assets.

## Tag Release

```powershell
git tag v0.2.1
git push origin v0.2.1
```

The GitHub release workflow builds platform artifacts and publishes archives for tags matching `v*`.
