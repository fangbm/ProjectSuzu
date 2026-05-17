# XP3 支持

Project Suzu 包含实验性的 XP3 archive reader，用于资源预览和迁移工作流。它不是 TJS/KAG 项目的即插即用引擎。

## 已支持

- 在文件开头附近发现 XP3 header。
- Raw 和 zlib-compressed index。
- Chained index。
- Stored 和 zlib-compressed file segment。
- 多 entry archive。
- 从 XP3 index 读取 UTF-16LE entry name。
- 路径 lookup 不区分大小写，并规范化 `/` 与 `\`。
- 通过 `AssetManager` 和 `SuzuApp` 注册 XP3-backed asset。
- 明文图片、音频、字体、文本和 `.szs` 脚本资源。

## 实验性功能

- `suzu-xp3-viewer`：检查 archive 内容并预览明文资源。
- `suzu-launcher` KRKR package scan mode：用于清单盘点和转换实验。
- 由应用所有者提供的外部 XP3 plugin module。

## 外部 Plugin Hook

公开仓库不包含游戏专用 XP3 处理器或私有处理规则。有权处理特定资源包的应用可以提供外部 XP3 plugin module。使用或贡献 plugin 相关代码前请阅读 `LEGAL.zh-CN.md`，完整接口参考见 `docs/xp3-plugin-interface.zh-CN.md`。

```json
{
  "format": "suzu.xp3-plugin.v1",
  "name": "Local XP3 processor",
  "xp3": {
    "processors": [
      {
        "type": "external_process",
        "command": "D:\\tools\\xp3-plugin.exe",
        "args": ["--entry", "{entry}"],
        "stage": "after_inflate"
      }
    ]
  }
}
```

外部进程从 stdin 接收 bytes，并必须在 stdout 返回相同数量的 bytes。支持的 placeholder：

- `{entry}`
- `{checksum}` 和 `{checksum_hex}`
- `{original_size}` 和 `{packed_size}`
- `{segment_offset}`、`{segment_original_size}` 和 `{segment_packed_size}`

Plugin module 和 plugin binary 应放在本仓库之外，除非它们只处理项目有权再分发的数据。CLI 和 GUI 工具在加载外部 XP3 plugin 前都要求明确授权确认。公开示例必须限制为 synthetic fixture 和 identity-style processor。

## 不支持

- 完整 TJS 执行。
- 完整 KAG 兼容。
- Patch layering semantics。
- 私有 package 处理规则。
- 商业游戏 bundled processor。
- 保证 KRKR 游戏可以直接启动。

## 测试策略

仓库单元测试使用 synthetic XP3 fixture。不要提交受版权保护的游戏 archive。做本地兼容性检查时，把私有 fixture 放在仓库外，只记录可以公开分享的汇总结果或文件名。
