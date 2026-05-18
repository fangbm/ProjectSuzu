# Release Checklist

Use this checklist before creating a `v*` tag or running a manual GitHub release workflow.

## v0.2.x Release Gate

- [ ] Update `CHANGELOG.md`.
- [ ] Update `CHANGELOG.zh-CN.md`.
- [ ] Confirm crate versions are intentional.
- [ ] Confirm `README.md` and `README.zh-CN.md` download links point at the release tag.
- [ ] Confirm package names use the same tag, for example `project-suzu-v0.2.1-windows-x64.tar.gz`.
- [ ] Confirm `Cargo.lock` reflects intentional workspace version changes.
- [ ] Confirm `README.md` describes the current feature set and keeps the author workflow as the main story.
- [ ] Confirm `docs/framework-guide.md` reflects the current project workflow.
- [ ] Confirm `docs/getting-started.md`, `docs/project-layout.md`, and `templates/starter-vn` still work as the recommended new-user entry point.
- [ ] Confirm `LEGAL.md`, XP3 support boundaries, and `docs/xp3-plugin-interface.md` are included.
- [ ] Regenerate and review `THIRD_PARTY_LICENSES.md` after dependency changes.
- [ ] Confirm `assets/branding/README.md` and `docs/api-stability.md` are current.
- [ ] Confirm the release package includes `suzu-player`, `suzu-editor`, `suzu-packer`, `suzu-compiler`, `templates/starter-vn`, and `examples/short-vn-demo`.

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo doc --workspace --no-deps`
- [ ] `cargo test -p suzu-script --features lua`
- [ ] `cargo run -p suzu-launcher -- --check`
- [ ] `cargo run -p suzu-player -- --check templates\starter-vn`
- [ ] `cargo run -p suzu-xp3-viewer -- --check`
- [ ] `cargo run -p suzu-editor -- --check`
- [ ] `cargo run -p suzu-bench -- 100`
- [ ] `.\scripts\package-desktop.ps1 -Check`
- [ ] `git status --short --branch`
- [ ] `git diff --check`

## Local Package

- [ ] `.\scripts\package-desktop.ps1`
- [ ] Confirm `dist/project-suzu-desktop.zip` contains tools, examples, docs, licenses, third-party notices, branding notes, changelog, and packed assets.
- [ ] Dry-run package inputs before tagging:

```powershell
.\scripts\package-desktop.ps1 -Check
cargo run -p suzu-player -- --check templates\starter-vn
cargo run -p suzu-compiler -- examples\short-vn-demo\script\main.szs
cargo run -p suzu-packer -- examples\short-vn-demo --pack target\short-vn-demo.suzupack
```

## Tag Release

```powershell
$tag = "v0.2.1"
git tag $tag
git push origin $tag
```

The tag-triggered GitHub release workflow builds platform artifacts and publishes archives for tags matching `v*`. Manual `workflow_dispatch` is for validation or recovery and must not be treated as a complete release unless a GitHub Release and downloadable assets are verified afterward.

## Post-Release Verification

```powershell
$tag = "v0.2.1"
gh run list --repo fangbm/ProjectSuzu --workflow Release --limit 10
gh release view $tag --repo fangbm/ProjectSuzu --json assets,isDraft,isPrerelease,publishedAt
```

- [ ] The Release workflow run for the tag completed successfully.
- [ ] The GitHub Release exists, is not a draft unless intentionally staged, and has the expected publish time.
- [ ] Windows and Linux assets both exist.
- [ ] Each asset is downloadable and has a non-zero size.
- [ ] Release notes or asset metadata include checksums/digests when available.
- [ ] The downloaded archive contains the expected tools, starter template, short VN demo, legal/security files, third-party notices, and user docs.
