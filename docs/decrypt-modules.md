# 解密模块

Project Suzu 可以通过 JSON 解密模块为 XP3 读取器加载游戏专用解密配置。模块文件只描述资源读取时的解密链路，不会修改原始游戏文件。

## 格式

```json
{
  "format": "suzu.decrypt-module.v1",
  "name": "XP3 XOR 5A",
  "xp3": {
    "decryptors": [
      {
        "type": "xor",
        "key": "5A"
      }
    ]
  }
}
```

`decryptors` 会按顺序组成管线。当前支持：

- `xor`: 对 XP3 加密段在解压前执行单字节 XOR。
- `xor_after_inflate`: 对 XP3 加密段在解压后执行单字节 XOR。
- `name_xor`: 对 XP3 索引里的文件名执行单字节 XOR。

`key` 可以写十六进制字符串，例如 `5A` 或 `0x5A`。

## 使用

在 Launcher 或 XP3 Viewer 的 `Decrypt module` 输入框填入模块 JSON 路径，再加载 XP3 或扫描 KRKR 目录。

命令行转换 KRKR 包时也可以指定模块：

```powershell
cargo run -p suzu-launcher -- --krkr2suzu "D:\game" "D:\out" --decrypt-module examples\decrypt-modules\xor-5a.json
```

## PackinOne 状态

`PackinOne.dll` / ChaCha 类保护还没有实现完整解密算法。当前版本会检测这种保护并阻止生成乱码工程；等 PackinOne 的 `seed`、`cryptmode`、`iv`、`outeriv` 解密链路确认后，可以把它作为新的解密模块类型接入。
