# Security Policy

Project Suzu is a local game framework and toolchain. It still processes untrusted-ish inputs such as scripts, asset manifests, package archives, and Lua extension snippets, so report issues that can crash tools, escape expected paths, corrupt output packages, or execute unintended code.

## Supported Versions

| Version | Supported |
| --- | --- |
| 0.1.x | yes |

## Reporting

For now, report security issues privately to the project maintainers before publishing details. Include:

- affected command, crate, or workflow;
- minimal reproduction steps;
- expected result and actual result;
- operating system and Rust toolchain version;
- any generated package, manifest, or script needed to reproduce the issue.

## Security-Sensitive Areas

- `.suzupack` archive parsing and checksum validation;
- recursive asset discovery and package output paths;
- Lua extension registration when the `lua` feature is enabled;
- save-game JSON loading;
- GitHub release and local packaging scripts.
