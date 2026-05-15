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
- XP3 plugin module JSON, external processor command resolution, and plugin stdin/stdout byte processing;
- Lua extension registration when the `lua` feature is enabled;
- save-game JSON loading;
- GitHub release and local packaging scripts.

## External XP3 Plugins

External XP3 plugins are intentionally not bundled with Project Suzu. A plugin module can launch an arbitrary executable, so only run modules from trusted sources and only for assets you are authorized to process. Keep plugin paths local, avoid shell wrappers when possible, and verify that processor output preserves byte counts unless the schema explicitly changes in a future version.
