# Short VN Demo Plan

`examples/short-vn-demo` is the planned proof that Project Suzu can ship a small, complete visual novel loop without relying on private or game-specific assets.

## Current Slice

The first checked-in slice is intentionally lightweight:

- a workspace example binary: `suzu-short-vn-demo`
- a title screen and seeded save slot
- one `.szs` script covering background changes, character display, dialogue, choices, variables, conditionals, BGM/voice placeholders, autosave, effects, waits, and character hide
- runtime fallback textures generated in Rust code
- packageable script/assets through `suzu-packer`

This keeps CI fast while making the demo a real executable instead of a design note.

## Completion Target

Before `v0.2.0`, the demo should become a short tutorial-ready VN:

- 3 to 5 minutes of play time
- at least two meaningful routes and one converged ending
- original or CC0 background art
- original or CC0 character art
- optional open-licensed BGM and short voice placeholders
- a README that explains how to edit the script, replace assets, pack resources, and ship the example
- release packages that include the demo binary and packed demo resources

## Asset Rules

Allowed:

- original assets made for Project Suzu
- CC0 assets
- assets with a clear license that allows redistribution in this repository and release archives

Not allowed:

- extracted commercial game assets
- decrypted XP3 outputs
- plugin configurations for specific games
- assets with unclear redistribution terms

## Acceptance Checklist

- `cargo run -p suzu-short-vn-demo` starts at the title screen.
- `cargo run -p suzu-compiler -- examples\short-vn-demo\script\main.szs` passes.
- `cargo run -p suzu-packer -- examples\short-vn-demo --pack target\short-vn-demo.suzupack` succeeds.
- `cargo test --workspace` compiles the example and smoke-tests the script.
- release packages include `suzu-short-vn-demo` plus packed short-demo resources.
