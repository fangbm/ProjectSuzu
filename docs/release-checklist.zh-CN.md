# Release 检查清单

创建 `v*` tag 或手动运行 GitHub release workflow 前，使用这份清单做最后确认。

## 版本说明

- [ ] 更新 `CHANGELOG.md` 和 `CHANGELOG.zh-CN.md`。
- [ ] 确认 crate 版本符合预期。
- [ ] 确认 `README.md` 和 `README.zh-CN.md` 描述当前功能集。
- [ ] 确认 `docs/framework-guide.md` 反映当前项目流程。
- [ ] 确认 `docs/getting-started.md`、`docs/project-layout.md` 和 `templates/krkr-like-vn` 仍是推荐的新用户入口。
- [ ] 确认 `LEGAL.md`、`LEGAL.zh-CN.md`、XP3 支持边界和 XP3 plugin 接口文档都已包含。
- [ ] 依赖变更后重新生成并检查 `THIRD_PARTY_LICENSES.md`。
- [ ] 确认 `assets/branding/README.md`、`assets/branding/README.zh-CN.md` 和 `docs/api-stability.md` 是最新状态。

## 验证

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo doc --workspace --no-deps`
- [ ] `cargo test -p suzu-script --features lua`
- [ ] `cargo run -p suzu-launcher -- --check`
- [ ] `cargo run -p suzu-player -- --check templates\krkr-like-vn`
- [ ] `cargo run -p suzu-xp3-viewer -- --check`
- [ ] `cargo run -p suzu-editor -- --check`
- [ ] `cargo run -p suzu-bench -- 100`
- [ ] `.\scripts\package-desktop.ps1 -Check`

## 本地发布包

- [ ] `.\scripts\package-desktop.ps1`
- [ ] 确认 `dist/project-suzu-desktop.zip` 包含工具、示例、英文和中文文档、许可证、第三方声明、品牌说明、changelog 和打包资源。

## Tag Release

```powershell
git tag v0.2.0
git push origin v0.2.0
```

GitHub release workflow 会为匹配 `v*` 的 tag 构建平台产物并发布 archive。
