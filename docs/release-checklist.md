# Release Checklist

Use this checklist before creating a `v*` tag or running a manual GitHub release workflow.

## Version Notes

- [ ] Update `CHANGELOG.md`.
- [ ] Confirm crate versions are intentional.
- [ ] Confirm `README.md` describes the current feature set.

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo doc --workspace --no-deps`
- [ ] `cargo test -p suzu-script --features lua`
- [ ] `cargo run -p suzu-bench -- 100`
- [ ] `.\scripts\package-desktop.ps1 -Check`

## Local Package

- [ ] `.\scripts\package-desktop.ps1`
- [ ] Confirm `dist/project-suzu-desktop.zip` contains tools, examples, docs, licenses, changelog, and packed assets.

## Tag Release

```powershell
git tag v0.1.0
git push origin v0.1.0
```

The GitHub release workflow builds platform artifacts and publishes archives for tags matching `v*`.
