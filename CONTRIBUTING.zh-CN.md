# 贡献指南

感谢你改进 Project Suzu。本仓库是 Rust workspace，请尽量把改动限制在拥有对应行为的 crate、工具或文档范围内。

## 开始前

- 阅读 `docs/user-guide.zh-CN.md` 或 `docs/user-guide.md`，了解项目结构。
- 修改脚本语法或 VM 命令前，阅读 `docs/scripting-reference.zh-CN.md`。
- 提交 PR 前，阅读 `docs/developer-checks.zh-CN.md`。
- 创建标签发布前，阅读 `docs/release-checklist.zh-CN.md`。
- 处理 XP3/KRKR 工具、外部插件模块或第三方资源前，阅读 `LEGAL.zh-CN.md`。

## 开发流程

1. 保持改动聚焦。
2. 在接近改动行为的位置新增或更新测试。
3. 运行必需开发门禁：

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

4. 如果改动脚本扩展能力，也运行：

```powershell
cargo test -p suzu-script --features lua
```

5. 如果改动发布打包，也运行：

```powershell
.\scripts\package-desktop.ps1 -Check
```

## 风格说明

- 优先沿用现有 crate 边界，不要随意增加跨 crate 捷径。
- 表示存档、脚本数据或配置的数据结构，新增公共 API 时应尽量保持可序列化。
- 示例应保持小巧、确定、可自动验证。
- 用户可见行为、工具、打包或文档变化应更新 `CHANGELOG.md` 和 `CHANGELOG.zh-CN.md`。

## 法律边界

- 不要提交商业游戏 archive、私有脚本、第三方 DLL、逆向产物或可识别的商业游戏样本。
- 不要提交解密密钥、DRM 绕过、许可证检查绕过或游戏专用 XP3 处理器。
- XP3 plugin 模块应保留在仓库外，除非它只处理项目有权再分发的数据。
