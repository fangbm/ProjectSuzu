# API Stability

Project Suzu is pre-1.0. Public APIs are kept steady when practical, but crates and tools may still change as the runtime converges.

## Current Stability Promise

- `suzu-app::SuzuApp`, `TitleMenuAction`, and `SystemMenuAction` are the primary runtime facade types.
- Existing public `SuzuApp` methods should not be renamed or removed during the `0.1.x` line without a changelog note.
- Script format `@script version=1` remains the current stable script format. The classic syntax remains the default when no `syntax` field is present.
- Script syntax front ends `syntax=classic`, `syntax=indent`, `syntax=braces`, and `syntax=markup` compile into the same command model. In `0.1.x`, non-classic styles are newer and may receive additive parsing improvements, but existing documented examples should keep compiling unless a changelog calls out a break.
- `.suzupack` format version `1` remains readable by the asset crate.
- GUI `--check` interfaces are intended for CI smoke usage and should remain scriptable.
- `TitleScreenConfig` additive fields such as labels and background texture are intended to stay backward-compatible through serde defaults.

## Experimental Areas

- Renderer internals and frame-building details.
- Desktop GUI layout and editor data model.
- XP3 plugin module schema beyond `suzu.xp3-plugin.v1`; the current interface is documented in `docs/xp3-plugin-interface.md`.
- KRKR/KAG conversion heuristics.
- Lua extension registration under the optional `lua` feature.

## Compatibility Guidance

For applications, prefer the facade methods on `SuzuApp` and asset-manager registration methods over reaching into crate internals. For tools, prefer `--check` for automation and treat visual GUI behavior as preview-quality until the project reaches `0.2.x`.
