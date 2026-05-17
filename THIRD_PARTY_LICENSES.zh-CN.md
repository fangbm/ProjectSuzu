# 第三方许可证说明

`THIRD_PARTY_LICENSES.md` 由 `cargo-about` 根据 Rust workspace 的依赖自动生成，是发布包中的正式第三方许可证声明。

本文件是中文导读，不替代各依赖的原始许可证文本，也不构成法律意见。遇到解释差异时，请以 `THIRD_PARTY_LICENSES.md` 中的原文许可证和各依赖上游许可证为准。

## 生成方式

依赖发生变化后运行：

```powershell
cargo about generate about-markdown.hbs --workspace --locked --fail -o THIRD_PARTY_LICENSES.md
```

生成后请检查：

- 许可证汇总是否合理。
- 是否出现未知或不兼容许可证。
- 新依赖是否符合项目的 `MIT OR Apache-2.0` 分发边界。
- Release 包是否同时包含 `THIRD_PARTY_LICENSES.md` 和本中文导读。

## 当前收录内容

正式清单位于 `THIRD_PARTY_LICENSES.md`，其中包含：

- 许可证概览。
- 每类许可证的原文条款。
- 使用该许可证的依赖名称、版本和上游链接。

请不要手工编辑 `THIRD_PARTY_LICENSES.md` 的许可证正文；如需更新，重新运行生成命令。
