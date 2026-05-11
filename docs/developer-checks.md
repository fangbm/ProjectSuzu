# Developer Checks

Use these checks before opening a pull request or creating a local release package.

## Required Gate

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

## Optional Feature Gate

```powershell
cargo test -p suzu-script --features lua
```

## Packaging Gate

```powershell
.\scripts\package-desktop.ps1 -Check
.\scripts\package-desktop.ps1
```

The `-Check` mode validates package inputs without building. The full packaging command creates `dist/project-suzu-desktop.zip`.

## Benchmark Smoke Test

```powershell
cargo run -p suzu-bench -- 100
```

Use larger iteration counts when comparing performance across changes. Release packages include `suzu-bench` so benchmarks can be run against packaged builds too.
