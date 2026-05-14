# Project Suzu Continuation Context

Last updated: 2026-05-14, Asia/Shanghai

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
- Current local and remote HEAD before the continuation-context refresh: `44e00ee Prepare v0.1.4 release boundaries`.
- `main` is clean and aligned with `origin/main` before this document refresh.
- `CHANGELOG.md` already contains `0.1.4 - 2026-05-14`.
- `v0.1.4` has not existed yet in GitHub Releases or remote tags at the time of this refresh.
- Existing preview tags/releases: `v0.1.4-xp3-preview.1` through `v0.1.4-xp3-preview.5`.
- One old stash remains: `stash@{0}: On feature/xp3-archive-support: local xp3 legal cleanup before main merge`. It was retained from earlier cleanup work; do not drop it unless the user asks.

## Release Plan

For the official `v0.1.4` release:

1. Commit this continuation-context refresh on `main`.
2. Push `main`.
3. Create tag `v0.1.4` on the pushed `main` commit.
4. Push tag `v0.1.4` to trigger `.github/workflows/release.yml`.
5. Before reporting success, verify:
   - GitHub Actions release run status.
   - Workflow build artifacts.
   - GitHub Release `v0.1.4` exists.
   - Release assets are uploaded.

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

## Commands

```powershell
git status --short --branch
gh release list --repo fangbm/ProjectSuzu --limit 20
git tag --list "v0.1.4*" --sort=-creatordate
git push origin main
git tag v0.1.4
git push origin v0.1.4
gh run list --repo fangbm/ProjectSuzu --workflow Release --limit 5
gh release view v0.1.4 --repo fangbm/ProjectSuzu
```
