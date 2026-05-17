# 开发检查

在打开 PR 或创建本地发布包之前，建议运行这些检查。

## 必需门禁

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

## GUI 检查门禁

```powershell
cargo run -p suzu-launcher -- --check
cargo run -p suzu-xp3-viewer -- --check
cargo run -p suzu-editor -- --check
```

这些命令验证无窗口启动路径，不会打开 GUI 窗口。

## 可选 feature 门禁

```powershell
cargo test -p suzu-script --features lua
```

## 打包门禁

```powershell
.\scripts\package-desktop.ps1 -Check
.\scripts\package-desktop.ps1
```

`-Check` 模式只验证打包输入，不执行构建。完整打包命令会创建 `dist/project-suzu-desktop.zip`。

依赖变更后重新生成第三方许可证声明：

```powershell
cargo about generate about-markdown.hbs --workspace --locked --fail -o THIRD_PARTY_LICENSES.md
```

## Benchmark Smoke Test

```powershell
cargo run -p suzu-bench -- 100
```

比较不同改动的性能时可以使用更大的迭代次数。发布包也包含 `suzu-bench`，方便对打包后的构建做基准测试。

## 短篇 Demo Smoke Test

```powershell
cargo run -p suzu-compiler -- examples\short-vn-demo\script\main.szs
cargo run -p suzu-packer -- examples\short-vn-demo --pack target\short-vn-demo.suzupack
```

完整 workspace test 已经会编译 `suzu-short-vn-demo`，smoke tests 也会编译 `examples/` 下的 `.szs` 脚本。
