# XP3 Plugin Interface

This document describes Project Suzu's external XP3 processor interface. It is an interface reference for authorized asset-processing workflows, not a decryption tutorial.

Project Suzu does not ship commercial game processors, keys, reverse-engineering notes, DRM bypasses, or access-control circumvention logic. Use external processors only for resources you own, resources you are explicitly authorized to process, synthetic fixtures, or lawful interoperability research that does not bypass technical protection measures. See `LEGAL.md` and `docs/xp3-support.md` for the project boundary.

## Terms

Preferred terms:

- XP3 external processor
- XP3 plugin module
- External XP3 processor
- Authorized resource-processing plugin

Avoid terms that imply cracking, DRM bypass, commercial-game adaptation, or bundled decryptors. The public repository exposes a neutral processing hook; application owners are responsible for any private processor they choose to run.

## Architecture

The interface has two pieces:

- Plugin module: a JSON file that tells Project Suzu which external program to run and when.
- External processor: a standalone executable that reads bytes from stdin and writes processed bytes to stdout.

Project Suzu parses XP3 metadata, selects an archive segment, optionally inflates zlib-compressed data, and then invokes configured processors at the requested stage. Multiple processors form a pipeline and run in array order.

## Module Format

Current format identifier:

```json
"suzu.xp3-plugin.v1"
```

Minimal module:

```json
{
  "format": "suzu.xp3-plugin.v1",
  "name": "Local synthetic fixture processor",
  "xp3": {
    "processors": [
      {
        "type": "external_process",
        "command": "D:\\tools\\xp3-identity.exe",
        "args": ["--entry", "{entry}"],
        "stage": "after_inflate"
      }
    ]
  }
}
```

Do not use commercial game names in public examples.

## JSON Fields

`format`: Optional in parser terms but strongly recommended. When present, it must be `suzu.xp3-plugin.v1`. Unknown formats are rejected so future schemas can evolve safely.

`name`: Optional human-readable module name. Use neutral labels such as `Local synthetic fixture processor`.

`xp3.processors`: Processor array. The current public schema supports only `external_process` processors. An empty array is valid and behaves like no plugin.

`type`: Must be `external_process`.

`command`: External executable path. Absolute paths are used as-is. Relative paths are resolved relative to the plugin module JSON file's directory.

`args`: Optional argument array. Each argument is expanded for supported placeholders before the process starts.

`stage`: Optional processing stage. Supported values are `segment` and `after_inflate`. The default is `after_inflate`.

## Processing Stages

`segment` runs on raw segment bytes before Project Suzu inflates zlib-compressed segments.

`after_inflate` runs after zlib inflation. For stored segments, this is still the file-byte stage after the segment is copied into the output buffer. If unsure, use `after_inflate` for owned plaintext or synthetic test fixtures.

## External Process Protocol

An external processor must:

1. Read all input bytes from stdin.
2. Write the processed bytes to stdout.
3. Preserve byte length exactly.
4. Return exit code `0` on success.
5. Return a non-zero exit code on failure.
6. Write failure details to stderr.
7. Never write logs, prompts, JSON, or extra newlines to stdout.

Project Suzu checks stdout length. A byte-count mismatch is treated as plugin failure.

## Placeholders

`args` supports these placeholders:

```text
{entry}
{checksum}
{checksum_hex}
{original_size}
{packed_size}
{segment_offset}
{segment_original_size}
{segment_packed_size}
```

Example:

```json
{
  "type": "external_process",
  "command": "D:\\tools\\xp3-processor.exe",
  "args": [
    "--entry",
    "{entry}",
    "--checksum",
    "{checksum_hex}",
    "--segment-offset",
    "{segment_offset}"
  ],
  "stage": "after_inflate"
}
```

These placeholders are metadata only. They do not grant permission to process an archive.

## Identity Processor Example

This processor returns input bytes unchanged and is suitable for synthetic fixture tests.

```rust
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;
    io::stdout().write_all(&input)?;
    Ok(())
}
```

Matching module:

```json
{
  "format": "suzu.xp3-plugin.v1",
  "name": "Identity processor for synthetic tests",
  "xp3": {
    "processors": [
      {
        "type": "external_process",
        "command": "D:\\tools\\xp3-identity.exe",
        "args": ["--entry", "{entry}"],
        "stage": "after_inflate"
      }
    ]
  }
}
```

Use this example to verify process startup, stdin/stdout byte flow, placeholder expansion, module-relative command resolution, and authorization-confirmation paths.

## Tool Usage

CLI tools require explicit authorization before loading an external XP3 plugin:

```powershell
cargo run -p suzu-launcher -- --check --xp3 "D:\fixtures\plain.xp3" --xp3-plugin "D:\plugins\xp3-plugin.json" --i-have-rights-to-process-these-assets
cargo run -p suzu-xp3-viewer -- --check --xp3 "D:\fixtures\plain.xp3" --xp3-plugin "D:\plugins\xp3-plugin.json" --i-have-rights-to-process-these-assets
cargo run -p suzu-launcher -- --krkr2suzu "D:\game" "D:\out" --xp3-plugin "D:\plugins\xp3-plugin.json" --i-have-rights-to-process-these-assets
```

Without `--i-have-rights-to-process-these-assets`, Project Suzu rejects the plugin. GUI tools also require the authorization checkbox before plugin-backed loading.

## Error Handling

External processors should:

- Return a non-zero exit code for unsupported entries or invalid input.
- Write a concise explanation to stderr.
- Avoid stdout unless writing the full successful byte output.
- Avoid changing input length.
- Avoid leaking keys, private paths, commercial game names, or reverse-engineering details in public logs.

Good public error wording:

```text
unsupported synthetic fixture variant for entry scenario/main.szs
```

Avoid wording that identifies commercial works, private keys, or access-control details.

## Security Guidance

External processors are ordinary executables and run with the current user's permissions.

- Run only trusted local processors.
- Keep private processors outside the public Project Suzu repository.
- Do not run private processors in public CI.
- Do not let processors modify original game directories.
- Do not write processed output back into the original XP3 archive.
- Prefer temporary output directories for local compatibility checks.

Possible future hardening areas include sandboxed execution, restricted environment variables, network isolation, and command path allowlists. Those are not part of `suzu.xp3-plugin.v1`.

## Public Test Matrix

Public repository tests should use synthetic fixtures only.

- Identity processor returns the same byte length.
- Non-zero processor exit is reported as failure.
- Stderr is included in the error.
- Byte-count mismatch is rejected.
- Relative processor paths resolve from the module directory.
- Placeholders expand correctly.
- Unsupported module formats are rejected.
- Missing authorization rejects plugin loading.

Never commit commercial XP3 files, scripts, DLLs, keys, game-specific processors, or proprietary plugin configurations.

## Suggested Local Layout

Private authorized processor layout outside the repository:

```text
local-tools/
  xp3-plugin.json
  xp3-processor.exe
  README-private.md
```

Synthetic public fixture layout, if needed:

```text
examples/
  synthetic-xp3-plugin/
    identity-processor/
    README.md
```

Public examples must state that they process only redistributable synthetic data.

## Review Checklist

- [ ] No commercial game names.
- [ ] No real XP3 archives, DLLs, scripts, images, audio, or fonts.
- [ ] No keys.
- [ ] No reverse-engineering steps.
- [ ] No DRM, license-check, or access-control bypass instructions.
- [ ] Examples use only synthetic fixtures.
- [ ] `LEGAL.md` is linked.
- [ ] Plugin users must confirm they have processing rights.
- [ ] CLI or GUI paths require authorization confirmation.
- [ ] Release packages include `LEGAL.md`, `SECURITY.md`, `docs/xp3-support.md`, and this interface document.
