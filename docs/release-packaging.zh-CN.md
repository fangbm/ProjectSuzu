# Release 打包

Project Suzu 支持两条发布路径：本地桌面打包和 GitHub tag release。

## 本地桌面包

运行：

```powershell
.\scripts\package-desktop.ps1
```

脚本会构建整个 workspace，复制示例 binary 和桌面工具，包括 `suzu-player`、`suzu-launcher`、`suzu-editor`、`suzu-xp3-viewer`，收录英文和中文用户文档、`templates/krkr-like-vn`、`templates/minimal-vn`、hello-world 和 short-demo 的 `.suzupack` 资源、JSON manifest，并生成 `dist/project-suzu-desktop.zip`。

发布包也包含 Project Suzu 图标与品牌说明、`CONTRIBUTING.md` / `CONTRIBUTING.zh-CN.md`、`SECURITY.md` / `SECURITY.zh-CN.md`、`LEGAL.md` / `LEGAL.zh-CN.md`、`LICENSE-MIT`、`LICENSE-APACHE`、`THIRD_PARTY_LICENSES.md`、`THIRD_PARTY_LICENSES.zh-CN.md`、`CHANGELOG.md` 和 `CHANGELOG.zh-CN.md`。

不构建，只检查打包输入：

```powershell
.\scripts\package-desktop.ps1 -Check
```

自定义输出目录和 asset root：

```powershell
.\scripts\package-desktop.ps1 -Output dist/my-build -AssetRoot examples/branching-story
```

## GitHub Release

`.github/workflows/release.yml` 会在匹配 `v*` 的 tag 上构建 Linux 和 Windows 产物。

```powershell
git tag v0.2.0
git push origin v0.2.0
```

workflow 上传的每个平台 archive 会包含工具、零代码 player、可视化剧本编辑器、benchmark CLI、示例、打包后的 hello-world 与 short-demo 资源、README、法律说明、品牌说明、第三方许可证声明、快速上手、框架指南、工程布局指南、短篇 demo 计划、低门槛模板、Rust 集成模板、XP3 接口文档和核心/开发文档。Release asset 文件名会包含 tag，例如 `project-suzu-v0.2.0-windows-x64.tar.gz`。

最后打 tag 前使用 `docs/release-checklist.zh-CN.md`。

## 发布前验证

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
cargo test -p suzu-script --features lua
cargo run -p suzu-launcher -- --check
cargo run -p suzu-player -- --check templates\krkr-like-vn
cargo run -p suzu-xp3-viewer -- --check
cargo run -p suzu-editor -- --check
.\scripts\package-desktop.ps1 -Check
```
