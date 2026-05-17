# KRKR 风格 Project Suzu 模板

这个模板展示低门槛 Project Suzu 工程布局。它可以直接由 `suzu-player` 或 `suzu-launcher` 打开，不需要编写 Rust `main.rs`。

```powershell
cargo run -p suzu-player -- templates\krkr-like-vn
cargo run -p suzu-player -- --check templates\krkr-like-vn
```

目录说明：

- `game.suzu.toml`：工程标题、入口脚本、窗口设置、资源目录和可选资源包。
- `scenario/main.szs`：默认脚本入口。
- `assets/`：图片、音频、字体和生成的 manifest。
- `saves/`：本地测试时的运行时存档。

新作者推荐使用 `syntax=indent`。高级 Rust 集成仍然可以通过 `SuzuApp` 使用，但这个模板的目标是更接近 KRKR 工程文件夹：编辑脚本和资源，然后运行目录。
