## 摘要

- 

## 改动区域

- [ ] app/runtime
- [ ] script/compiler
- [ ] render/platform
- [ ] text/audio
- [ ] assets/save/input
- [ ] docs/packaging/CI

## 检查

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo doc --workspace --no-deps`
- [ ] `cargo test -p suzu-script --features lua`，如果改动了脚本扩展行为
- [ ] `.\scripts\package-desktop.ps1 -Check`，如果改动了打包或 release 文件

## 备注

- 
