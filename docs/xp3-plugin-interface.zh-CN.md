# XP3 Plugin 接口

本文档描述 Project Suzu 的外部 XP3 processor 接口。它是授权资源处理工作流的接口参考，不是解密教程。

Project Suzu 不随仓库发布商业游戏处理器、密钥、逆向笔记、DRM 绕过或访问控制规避逻辑。外部 processor 只应用于你拥有的资源、明确获得授权处理的资源、synthetic fixture，或不绕过技术保护措施的合法互操作研究。项目边界见 `LEGAL.zh-CN.md` 和 `docs/xp3-support.zh-CN.md`。

## 术语

推荐用语：

- XP3 external processor
- XP3 plugin module
- External XP3 processor
- Authorized resource-processing plugin

避免暗示破解、DRM 绕过、商业游戏适配或 bundled decryptor 的用语。公开仓库只暴露中性的处理 hook；应用所有者需自行负责所运行的私有 processor。

## 架构

接口由两部分组成：

- Plugin module：一个 JSON 文件，告诉 Project Suzu 何时运行哪个外部程序。
- External processor：独立可执行程序，从 stdin 读取 bytes，并把处理后的 bytes 写入 stdout。

Project Suzu 解析 XP3 元数据，选择 archive segment，必要时展开 zlib-compressed data，然后在配置指定的 stage 调用 processor。多个 processor 会组成 pipeline，并按数组顺序运行。

## Module Format

当前 format identifier：

```json
"suzu.xp3-plugin.v1"
```

最小 module：

```json
{
  "format": "suzu.xp3-plugin.v1",
  "name": "Local synthetic fixture processor",
  "xp3": {
    "processors": [
      {
        "type": "external_process",
        "command": "D:\\tools\\xp3-identity.exe",
        "args": ["--entry", "{entry}"],
        "stage": "after_inflate"
      }
    ]
  }
}
```

公开示例不要使用商业游戏名称。

## JSON 字段

`format`：解析层面可选，但强烈建议填写。存在时必须是 `suzu.xp3-plugin.v1`。未知 format 会被拒绝，方便未来 schema 安全演进。

`name`：可选的人类可读 module 名称。使用中性标签，例如 `Local synthetic fixture processor`。

`xp3.processors`：Processor 数组。当前公开 schema 只支持 `external_process` processor。空数组合法，行为等同无 plugin。

`type`：必须为 `external_process`。

`command`：外部可执行路径。绝对路径按原样使用。相对路径会基于 plugin module JSON 文件所在目录解析。

`args`：可选参数数组。进程启动前，每个参数都会展开受支持 placeholder。

`stage`：可选处理阶段。支持 `segment` 和 `after_inflate`。默认是 `after_inflate`。

## 处理阶段

`segment` 在 Project Suzu 展开 zlib-compressed segment 之前处理原始 segment bytes。

`after_inflate` 在 zlib inflate 之后运行。对 stored segment 来说，这也是 segment 被复制到输出 buffer 后的 file-byte 阶段。不确定时，对拥有权利的明文或 synthetic test fixture 使用 `after_inflate`。

## 外部进程协议

外部 processor 必须：

1. 从 stdin 读取全部 input bytes。
2. 将 processed bytes 写入 stdout。
3. 精确保留 byte length。
4. 成功时返回 exit code `0`。
5. 失败时返回非零 exit code。
6. 将失败详情写入 stderr。
7. 不要向 stdout 写日志、prompt、JSON 或额外换行。

Project Suzu 会检查 stdout 长度。byte-count 不匹配会被视为 plugin failure。

## Placeholder

`args` 支持这些 placeholder：

```text
{entry}
{checksum}
{checksum_hex}
{original_size}
{packed_size}
{segment_offset}
{segment_original_size}
{segment_packed_size}
```

示例：

```json
{
  "type": "external_process",
  "command": "D:\\tools\\xp3-processor.exe",
  "args": [
    "--entry",
    "{entry}",
    "--checksum",
    "{checksum_hex}",
    "--segment-offset",
    "{segment_offset}"
  ],
  "stage": "after_inflate"
}
```

这些 placeholder 只是元数据，不代表你获得了处理 archive 的许可。

## Identity Processor 示例

这个 processor 原样返回输入 bytes，适合 synthetic fixture test。

```rust
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;
    io::stdout().write_all(&input)?;
    Ok(())
}
```

匹配 module：

```json
{
  "format": "suzu.xp3-plugin.v1",
  "name": "Identity processor for synthetic tests",
  "xp3": {
    "processors": [
      {
        "type": "external_process",
        "command": "D:\\tools\\xp3-identity.exe",
        "args": ["--entry", "{entry}"],
        "stage": "after_inflate"
      }
    ]
  }
}
```

可用它验证 process startup、stdin/stdout byte flow、placeholder expansion、module-relative command resolution 和 authorization-confirmation path。

## 工具用法

CLI 工具在加载外部 XP3 plugin 前要求显式授权：

```powershell
cargo run -p suzu-launcher -- --check --xp3 "D:\fixtures\plain.xp3" --xp3-plugin "D:\plugins\xp3-plugin.json" --i-have-rights-to-process-these-assets
cargo run -p suzu-xp3-viewer -- --check --xp3 "D:\fixtures\plain.xp3" --xp3-plugin "D:\plugins\xp3-plugin.json" --i-have-rights-to-process-these-assets
cargo run -p suzu-launcher -- --krkr2suzu "D:\game" "D:\out" --xp3-plugin "D:\plugins\xp3-plugin.json" --i-have-rights-to-process-these-assets
```

没有 `--i-have-rights-to-process-these-assets` 时，Project Suzu 会拒绝 plugin。GUI 工具也必须勾选授权确认后才会进行 plugin-backed loading。

## 错误处理

外部 processor 应：

- 对 unsupported entry 或 invalid input 返回非零 exit code。
- 向 stderr 写简短解释。
- 除成功输出完整 byte output 外，不要使用 stdout。
- 不要改变 input length。
- 不要在公开日志中泄露密钥、私有路径、商业游戏名称或逆向细节。

良好的公开错误措辞：

```text
unsupported synthetic fixture variant for entry scenario/main.szs
```

避免包含商业作品、私有密钥或访问控制细节。

## 安全建议

外部 processor 是普通可执行程序，会以当前用户权限运行。

- 只运行可信本地 processor。
- 私有 processor 保留在公开 Project Suzu 仓库之外。
- 不要在公开 CI 中运行私有 processor。
- 不要让 processor 修改原始游戏目录。
- 不要把 processed output 写回原始 XP3 archive。
- 本地兼容性检查优先使用临时输出目录。

未来可能强化的方向包括 sandboxed execution、restricted environment variables、network isolation 和 command path allowlist。这些不属于 `suzu.xp3-plugin.v1`。

## 公开测试矩阵

公开仓库测试只应使用 synthetic fixture。

- Identity processor 返回相同 byte length。
- 非零 processor exit 会作为失败报告。
- Stderr 会包含在 error 中。
- Byte-count mismatch 会被拒绝。
- Relative processor path 从 module directory 解析。
- Placeholder 正确展开。
- Unsupported module format 被拒绝。
- 未授权时拒绝 plugin loading。

不要提交商业 XP3 文件、脚本、DLL、密钥、游戏专用 processor 或专有 plugin 配置。

## 建议本地布局

仓库外私有授权 processor 布局：

```text
local-tools/
  xp3-plugin.json
  xp3-processor.exe
  README-private.md
```

如需 synthetic public fixture，可使用：

```text
examples/
  synthetic-xp3-plugin/
    identity-processor/
    README.md
```

公开示例必须声明它们只处理可再分发的 synthetic data。

## Review 清单

- [ ] 没有商业游戏名称。
- [ ] 没有真实 XP3 archive、DLL、脚本、图片、音频或字体。
- [ ] 没有密钥。
- [ ] 没有逆向步骤。
- [ ] 没有 DRM、license-check 或访问控制绕过说明。
- [ ] 示例只使用 synthetic fixture。
- [ ] 链接 `LEGAL.zh-CN.md`。
- [ ] Plugin 用户必须确认自己有处理权利。
- [ ] CLI 或 GUI 路径要求授权确认。
- [ ] Release 包包含 `LEGAL.md`、`SECURITY.md`、`docs/xp3-support.md` 和本接口文档。
