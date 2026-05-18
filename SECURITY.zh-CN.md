# 安全策略

Project Suzu 是本地游戏框架和工具链。它仍会处理不完全可信的输入，例如脚本、资源 manifest、包 archive 和 Lua 扩展片段。若你发现能导致工具崩溃、逃出预期路径、破坏输出包或执行非预期代码的问题，请报告。

## 支持版本

| 版本 | 是否支持 |
| --- | --- |
| 0.2.x | 是 |

## 报告方式

目前请先私下向项目维护者报告安全问题，再公开细节。报告中请包含：

- 受影响的命令、crate 或工作流；
- 最小复现步骤；
- 预期结果和实际结果；
- 操作系统和 Rust toolchain 版本；
- 复现所需的生成包、manifest 或脚本。

## 敏感区域

- `.suzupack` archive 解析和 checksum 校验；
- 递归资源发现和包输出路径；
- XP3 plugin 模块 JSON、外部处理器命令解析、plugin stdin/stdout 字节处理；
- 启用 `lua` feature 时的 Lua 扩展注册；
- 存档 JSON 加载；
- GitHub release 和本地打包脚本。

## 外部 XP3 Plugin

Project Suzu 有意不内置外部 XP3 plugin。plugin 模块可以启动任意可执行程序，因此只运行可信来源的模块，并且只用于你被授权处理的资源。保持 plugin 路径本地化，尽可能避免 shell wrapper，并确认处理器输出字节数保持不变，除非未来 schema 明确改变这一规则。

运行外部 plugin 前，请审查：

- command path 是否指向可信的本地可执行文件；
- processor 是否来自可审计源码或已知发布者；
- module 或 binary 是否内嵌密钥、私有规则或游戏专用处理逻辑；
- module 是否说明其授权处理的资源范围；
- processor 是否会上传本地文件、联网或通过 shell wrapper 运行。
