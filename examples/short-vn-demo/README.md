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
