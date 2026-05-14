# Project Suzu Continuation Context

Last updated: 2026-05-14 23:03:09 +08:00

This is the compressed handoff for continuing Project Suzu. Update this file at the end of each future Codex conversation on this repository before sending the final reply. Treat live `git status`, GitHub Releases, and GitHub Actions as the source of truth when they differ from this note.

## Project Identity

- Product name: `Project Suzu`.
- Repository: `fangbm/ProjectSuzu`.
- Local path: `D:\codex\projectSuzu`.
- Integration branch: `main`.
- Historical XP3 preview branch: `feature/xp3-archive-support`.
- License: `MIT OR Apache-2.0`.

## Current Repo State

- Current branch: `main`.
- `main` was pushed through `78df87b Add continuation context handoff` before the `v0.1.4` release tag was created.
- `v0.1.4` points at `78df87b`.
- This post-release note is intentionally after the `v0.1.4` tag and should be pushed to `main` only; it does not change the published release tag.
- `CHANGELOG.md` already contains `0.1.4 - 2026-05-14`.
- Existing preview tags/releases: `v0.1.4-xp3-preview.1` through `v0.1.4-xp3-preview.5`.
- One old stash remains: `stash@{0}: On feature/xp3-archive-support: local xp3 legal cleanup before main merge`. It was retained from earlier cleanup work; do not drop it unless the user asks.

## Release Status

Official `v0.1.4` is published.

- Release URL: `https://github.com/fangbm/ProjectSuzu/releases/tag/v0.1.4`.
- Release workflow run: `25866883593`, conclusion `success`.
- Quality gate passed: fmt, clippy, workspace tests, docs, Lua feature tests, and packaging precheck.
- Build jobs passed: `Build ubuntu-latest` and `Build windows-latest`.
- Publish job passed: `Publish GitHub Release`.
- Workflow artifacts confirmed:
  - `project-suzu-linux-x64`
  - `project-suzu-windows-x64`
- Release assets confirmed uploaded:
  - `project-suzu-v0.1.4-linux-x64.tar.gz`
  - `project-suzu-v0.1.4-windows-x64.tar.gz`
- CI for both `main` push and `v0.1.4` tag push also completed successfully.

The release workflow runs on pushed `v*` tags. Manual workflow dispatch can build artifacts but does not publish a GitHub Release because the publish job only runs for tag refs.

## User Preferences

- Keep the GitHub repository public.
- Use versioned release asset filenames.
- Release notes should show changes between versions.
- Keep a meaningful local commit before release automation.
- Verify release completion from Actions, artifacts, release existence, and release assets.
- Do not guess preview numbering; check GitHub Releases first.
- Prefer Windows and Linux assets unless the user asks for macOS again.
- If only workflow or documentation changes, do not bump the project version unless explicitly requested.

## XP3 And Legal Boundary

- Repository code supports plain, unencrypted XP3 archive reading.
- Repository code must not ship built-in XOR, ChaCha, PackinOne, game-specific decryptors, or decryptor examples.
- Keep only a neutral external XP3 plugin hook.
- Do not upload future decryption plugins to the repository.
- Game-specific or decryption-related experiments are local only.
- Avoid public-facing claims of full KRKR compatibility.
- Preferred wording: XP3 archive reader, XP3-backed asset loading, XP3 asset preview, KRKR package scan mode.

## Recent Work

- Merged XP3 archive support into `main`.
- Removed repo-shipped concrete decryptor functionality and examples.
- Added explicit XP3 support boundaries and neutral plugin-hook documentation.
- Added XP3 viewer and unified launcher preview tooling.
- Added release quality gates, tag-triggered publish, and versioned Windows/Linux artifact packaging.
- Fixed workspace repository metadata.
- Latest successful CI on `main` before this refresh: run `25861280447`, commit `44e00ee`.
- Added and tracked this continuation-context file in `78df87b`.
- Pushed `v0.1.4` and verified the official release.

## Commands

```powershell
git status --short --branch
gh release list --repo fangbm/ProjectSuzu --limit 20
git tag --list "v0.1.4*" --sort=-creatordate
gh run list --repo fangbm/ProjectSuzu --workflow Release --limit 5
gh release view v0.1.4 --repo fangbm/ProjectSuzu
```
