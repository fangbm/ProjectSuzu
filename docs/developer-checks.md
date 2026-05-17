# Developer Checks

Use these checks before opening a pull request or creating a local release package.

## Required Gate

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

## GUI Check Gate

```powershell
cargo run -p suzu-launcher -- --check
cargo run -p suzu-xp3-viewer -- --check
cargo run -p suzu-editor -- --check
```

These commands validate headless startup paths without opening windows.

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

Regenerate third-party notices after dependency changes:

```powershell
cargo about generate about-markdown.hbs --workspace --locked --fail -o THIRD_PARTY_LICENSES.md
```

## Benchmark Smoke Test

```powershell
cargo run -p suzu-bench -- 100
```

Use larger iteration counts when comparing performance across changes. Release packages include `suzu-bench` so benchmarks can be run against packaged builds too.

## Short Demo Smoke Test

```powershell
cargo run -p suzu-compiler -- examples\short-vn-demo\script\main.szs
cargo run -p suzu-packer -- examples\short-vn-demo --pack target\short-vn-demo.suzupack
```

The full workspace test already compiles `suzu-short-vn-demo` and the smoke tests compile `.szs` scripts under `examples/`.
