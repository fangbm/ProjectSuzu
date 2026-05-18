# Release 检查清单

创建 `v*` tag 或手动运行 GitHub release workflow 前，使用这份清单做最后确认。

## v0.2.x 发布门禁

- [ ] 更新 `CHANGELOG.md` 和 `CHANGELOG.zh-CN.md`。
- [ ] 确认 crate 版本符合预期。
- [ ] 确认 `README.md` 和 `README.zh-CN.md` 的下载链接指向本次 release tag。
- [ ] 确认包名使用同一个 tag，例如 `project-suzu-v0.2.1-windows-x64.tar.gz`。
- [ ] 确认 `Cargo.lock` 反映了有意的 workspace 版本变化。
- [ ] 确认 `README.md` 和 `README.zh-CN.md` 描述当前功能集，并把作者工作流放在主叙事。
- [ ] 确认 `docs/framework-guide.md` 反映当前项目流程。
- [ ] 确认 `docs/getting-started.md`、`docs/project-layout.md` 和 `templates/starter-vn` 仍是推荐的新用户入口。
- [ ] 确认 `LEGAL.md`、`LEGAL.zh-CN.md`、XP3 支持边界和 XP3 plugin 接口文档都已包含。
- [ ] 依赖变更后重新生成并检查 `THIRD_PARTY_LICENSES.md`。
- [ ] 确认 `assets/branding/README.md`、`assets/branding/README.zh-CN.md` 和 `docs/api-stability.md` 是最新状态。
- [ ] 确认 release 包包含 `suzu-player`、`suzu-editor`、`suzu-packer`、`suzu-compiler`、`templates/starter-vn` 和 `examples/short-vn-demo`。

## 验证

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
- [ ] `git status --short --branch`
- [ ] `git diff --check`

## 本地发布包

- [ ] `.\scripts\package-desktop.ps1`
- [ ] 确认 `dist/project-suzu-desktop.zip` 包含工具、示例、英文和中文文档、许可证、第三方声明、品牌说明、changelog 和打包资源。
- [ ] tag 前先 dry-run 包输入：

```powershell
.\scripts\package-desktop.ps1 -Check
cargo run -p suzu-player -- --check templates\starter-vn
cargo run -p suzu-compiler -- examples\short-vn-demo\script\main.szs
cargo run -p suzu-packer -- examples\short-vn-demo --pack target\short-vn-demo.suzupack
```

## Tag Release

```powershell
$tag = "v0.2.1"
git tag $tag
git push origin $tag
```

Tag 触发的 GitHub release workflow 会为匹配 `v*` 的 tag 构建平台产物并发布 archive。手动 `workflow_dispatch` 只用于验证或恢复；除非后续确认 GitHub Release 和可下载 assets 已存在，否则不能把手动 workflow 成功当作正式发布完成。

## 发布后验证

```powershell
$tag = "v0.2.1"
gh run list --repo fangbm/ProjectSuzu --workflow Release --limit 10
gh release view $tag --repo fangbm/ProjectSuzu --json assets,isDraft,isPrerelease,publishedAt
```

- [ ] 对应 tag 的 Release workflow run 成功完成。
- [ ] GitHub Release 存在，除非有意暂存，否则不是 draft，并且发布时间符合预期。
- [ ] Windows 和 Linux assets 都存在。
- [ ] 每个 asset 都可下载且大小非 0。
- [ ] Release notes 或 asset metadata 在可用时包含 checksum/digest。
- [ ] 下载后的 archive 包含预期工具、starter 模板、short VN demo、法律/安全文件、第三方声明和用户文档。
