# XP3 Support

Project Suzu includes an experimental XP3 archive reader for asset preview and migration workflows. It is not a drop-in engine for TJS/KAG projects.

## Supported

- XP3 header discovery near the start of a file.
- Raw and zlib-compressed indexes.
- Chained indexes.
- Stored and zlib-compressed file segments.
- Multi-entry archives.
- UTF-16LE entry names from the XP3 index.
- Case-insensitive path lookup with `/` and `\` normalization.
- XP3-backed asset registration through `AssetManager` and `SuzuApp`.
- Plaintext image, audio, font, text, and `.szs` script assets.

## Experimental

- `suzu-xp3-viewer` for inspecting archive contents and previewing plaintext assets.
- `suzu-launcher` KRKR package scan mode for inventory and conversion experiments.
- External XP3 plugin modules supplied by the application owner.

## External Plugin Hook

The public repository does not include game-specific XP3 processors or proprietary handling rules. Applications that have the right to process a specific package can provide an external XP3 plugin module. See `LEGAL.md` before using or contributing plugin-related code, and see `docs/xp3-plugin-interface.md` for the full interface reference.

```json
{
  "format": "suzu.xp3-plugin.v1",
  "name": "Local XP3 processor",
  "xp3": {
    "processors": [
      {
        "type": "external_process",
        "command": "D:\\tools\\xp3-plugin.exe",
        "args": ["--entry", "{entry}"],
        "stage": "after_inflate"
      }
    ]
  }
}
```

The external process receives bytes on stdin and must return the same number of bytes on stdout. Supported placeholders are:

- `{entry}`
- `{checksum}` and `{checksum_hex}`
- `{original_size}` and `{packed_size}`
- `{segment_offset}`, `{segment_original_size}`, and `{segment_packed_size}`

Plugin modules and plugin binaries should live outside this repository unless they only process data that the project has the right to redistribute. CLI and GUI tools require an explicit authorization confirmation before loading an external XP3 plugin. Public examples must be limited to synthetic fixtures and identity-style processors.

## Not Supported

- Full TJS execution.
- Full KAG compatibility.
- Patch layering semantics.
- Proprietary package processing rules.
- Bundled processors for commercial games.
- Guarantees that a KRKR game can be launched directly.

## Testing Strategy

The repository uses synthetic XP3 fixtures in unit tests. Do not commit copyrighted game archives. For local compatibility checks, use private fixtures outside the repository and document only aggregate results or file names that are safe to share.
