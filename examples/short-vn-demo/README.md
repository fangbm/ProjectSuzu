# Project Suzu Short VN Demo

This example is the first pass at the complete short visual novel demo planned for the `0.2.x` development direction.

Run it from the repository root:

```powershell
cargo run -p suzu-short-vn-demo
```

The demo currently covers:

- title screen startup
- background transitions
- character show/update/hide
- dialogue and click waits
- choices
- variables and `@if/@else/@endif`
- BGM and voice command placeholders
- autosave plus a seeded load slot
- history, system menu, and auto mode through the shared runtime UI
- packageable script/assets through `suzu-packer`

The visual textures are generated fallback colors in code. Replace them with original or openly licensed assets before treating this as a polished sample.

## Edit The Script

The story source lives at `script/main.szs`. It is intentionally small and uses core commands that the editor MVP should support: background changes, character display, dialogue, choices, variables, conditionals, waits, autosave, voice placeholders, animation, and effects.

After edits, check the script from the repository root:

```powershell
cargo run -p suzu-compiler -- examples\short-vn-demo\script\main.szs
```

## Replace Assets

Put redistributable assets under `assets/`. Use original work, CC0 files, or assets whose license explicitly allows redistribution in this repository and release archives. Record source and license notes in `assets/README.md`.

Do not add commercial game resources, third-party archive outputs, private plugin configs, or files with unclear redistribution terms.

## Pack And Run

```powershell
cargo run -p suzu-packer -- examples\short-vn-demo --pack target\short-vn-demo.suzupack
cargo run -p suzu-short-vn-demo
```

The release package should include the demo binary and packed demo resources once the sample is promoted from fallback textures to redistributable art/audio.
