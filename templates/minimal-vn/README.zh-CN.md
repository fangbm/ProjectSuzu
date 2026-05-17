# 最小 VN 模板

这个模板是最小可用的 Project Suzu 桌面项目。它加载 `script/main.szs`，扫描 `assets/`，当没有可加载图片时注册 fallback texture，并启动标准桌面运行时。

从仓库根目录运行：

```powershell
cargo run --manifest-path templates\minimal-vn\Cargo.toml
```

可以从这些位置开始改成自己的项目：

- `Cargo.toml`：包名和依赖路径。
- `script/main.szs`：标签、对白、选项和命令。
- `src/main.rs`：标题界面文本、资源路径、fallback texture。
- `assets/`：项目图片、音频和 manifest。

模板存放在 Project Suzu 仓库内，因此使用本地 path dependency。如果复制到仓库外，请先更新 `suzu-app` 和 `suzu-platform` 的依赖路径。
