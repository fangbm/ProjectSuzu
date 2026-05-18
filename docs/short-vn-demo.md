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

## Next Completion Target

During the `0.2.x` line, the demo should continue toward a short tutorial-ready VN:

- 3 to 5 minutes of play time
- at least two meaningful routes and one converged ending
- original or CC0 background art
- original or CC0 character art
- optional open-licensed BGM and short voice placeholders
- a README that explains how to edit the script, replace assets, pack resources, and ship the example
- release packages that include the demo binary and packed demo resources

## Completion Script Outline

The completed demo should stay small enough for CI and release packages, but large enough to prove a full author loop:

1. Title screen into a station opening scene.
2. A first choice that changes the explanation route and sets a variable.
3. A middle scene that demonstrates autosave, wait, visual effects, and character animation.
4. A second choice that changes one later line or route note without multiplying the whole script.
5. A converged ending that points authors to replacing assets and packaging the project.

Keep the script in the public repository synthetic and tutorial-oriented. It should not reference private projects, commercial games, or local compatibility experiments.

## Asset Sourcing Plan

- Backgrounds: use original simple illustrations, generated placeholders with a redistribution note, or CC0 images whose source and license can be recorded in `examples/short-vn-demo/assets/README.md`.
- Characters: use original placeholder portraits or redistributable CC0/open-licensed art. Prefer simple static images that exercise `@char`, `@anim`, and `@hidechar`.
- Audio: use silent placeholders, original short tones, or open-licensed BGM/voice clips with license text. Audio is optional until the final tutorial polish pass.
- Packaging: keep source assets and the packed `.suzupack` output reproducible through `suzu-packer`; do not commit generated archives unless release packaging expects them.

## Asset Rules

Allowed:

- original assets made for Project Suzu
- CC0 assets
- assets with a clear license that allows redistribution in this repository and release archives
- generated placeholders with an explicit redistribution note

Not allowed:

- extracted commercial game assets
- processed XP3 outputs from third-party archives
- plugin configurations for specific games
- assets with unclear redistribution terms

## Acceptance Checklist

- `cargo run -p suzu-short-vn-demo` starts at the title screen.
- `cargo run -p suzu-compiler -- examples\short-vn-demo\script\main.szs` passes.
- `cargo run -p suzu-packer -- examples\short-vn-demo --pack target\short-vn-demo.suzupack` succeeds.
- `cargo test --workspace` compiles the example and smoke-tests the script.
- release packages include `suzu-short-vn-demo` plus packed short-demo resources.
