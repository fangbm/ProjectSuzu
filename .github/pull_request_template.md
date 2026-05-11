## Summary

- 

## Changed Areas

- [ ] app/runtime
- [ ] script/compiler
- [ ] render/platform
- [ ] text/audio
- [ ] assets/save/input
- [ ] docs/packaging/CI

## Checks

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo doc --workspace --no-deps`
- [ ] `cargo test -p suzu-script --features lua` if script extension behavior changed
- [ ] `.\scripts\package-desktop.ps1 -Check` if packaging or release files changed

## Notes

- 
