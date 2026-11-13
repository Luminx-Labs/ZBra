# Changelog

All notable changes to this project are documented here.

## Unreleased

### Added

- add the basic parsing capabilities in `zbra_parser` ([46ef75d](https://github.com/Luminx-Labs/ZBra/commit/46ef75df229edaa279d4f77ec880f2cfb503e79d)
- add [`ParserError`] for `zbra_parser` for proper error handling ([0aa06fa](https://github.com/Luminx-Labs/ZBra/commit/0aa06faae931f83a73e7a4a96c8d203d4b0302de)

### Fixed

- move the FEN strings parsing from `zbra_parser` to `zbra_chess` for efficiency ([abe45a1](https://github.com/Luminx-Labs/ZBra/commit/abe45a1bb386067c7f3bd84c586151b65272f97d)
- add the `pre-release` flag to `version` subcommand and improve the beheaviour of xtask ([b7f64c6](https://github.com/Luminx-Labs/ZBra/commit/b7f64c69ec8961591e4f7ef1ab1d6274d43d0e03)

### Documentation

- configure mdbook and write the initial docs ([c33af8c](https://github.com/Luminx-Labs/ZBra/commit/c33af8cfdebae84f51667a9f3421c0e435fff25b)

### Other

- bump to v0.2.0-alpha.0 ([ba4f6d7](https://github.com/Luminx-Labs/ZBra/commit/ba4f6d72ab915733adc9d5983e56034dee6bfa23)
- rust formating ([ed6094a](https://github.com/Luminx-Labs/ZBra/commit/ed6094aed123ea7bebf82478602a030f532f110a)
- configure `cargo-deny` ([89579f5](https://github.com/Luminx-Labs/ZBra/commit/89579f517311b0fcc938cbdf7c09f67243a0d636)
- add the Python-Rust co-ordination and update the packages manifests according to it ([ab12332](https://github.com/Luminx-Labs/ZBra/commit/ab1233200bc8c33fabd1d4ce67c7a24bf0e59338)
- move the parsing code from `zbra_chess` to `zbra_parser` ([bbd7753](https://github.com/Luminx-Labs/ZBra/commit/bbd77539753c3bf986b048aff4a2116f3faad63b)
- remove unused deps and add deps that will be used ([46ca8a5](https://github.com/Luminx-Labs/ZBra/commit/46ca8a54adf2a0d30e1a2e676eb28eb6a1a8a2d0)

## v0.1.0-alpha.1 - 2026-11-03

### Fixed

- fix the changelog generation of xtask ([88833a7](https://github.com/Luminx-Labs/ZBra/commit/88833a7d339166a01507b26ad7321d2f3fbd983e)

### Other

- bump the version ([7f3caad](https://github.com/Luminx-Labs/ZBra/commit/7f3caad9b5ee727eafb047782d580916d34a4750)
- rustfmt ([76935b3](https://github.com/Luminx-Labs/ZBra/commit/76935b3d376072a71ab457ac3346ded35cf3cc57)

## v0.1.0-alpha.0 - 2026-11-03

### Added

- add all essential features to `zbra_chess` ([32adccd](https://github.com/Luminx-Labs/ZBra/commit/32adccde3a80d6f36295afb972f95ad0d453d73d)
- add a complete working xtask to workspace ([0451379](https://github.com/Luminx-Labs/ZBra/commit/0451379fca078d3127e1aaab34124be764e247ca)
- add a custom logging module for xtask ([57a40f0](https://github.com/Luminx-Labs/ZBra/commit/57a40f051f1e0c3569f2795e3cbc6073229591b4)
- create a Cargo Workspace and make `xtask` ([c3a5968](https://github.com/Luminx-Labs/ZBra/commit/c3a596889541cdd09d0b11ba3b2aedee59899a64)

### Fixed

- add appropriate macros and change the naming style ([fb8ef6c](https://github.com/Luminx-Labs/ZBra/commit/fb8ef6c8ce1e7437b65dd62ee3bc96bf1cf984cf)

### Documentation

- create a `mdbook` on developmental guide ([a18ca56](https://github.com/Luminx-Labs/ZBra/commit/a18ca56dec8462e36ef103c697a13cff3dab0a98)

### Other

- rewrite the correct tag ([a1b03ac](https://github.com/Luminx-Labs/ZBra/commit/a1b03accae90d349bd2196facf1558225b5781b5)
- add a crate named `zbra_py` to export the python bindings ([07e19e9](https://github.com/Luminx-Labs/ZBra/commit/07e19e93fa711e6f20d0dce7c748c9843c66ead4)
- Add `AGENTS.md` for agents ([e52cd3e](https://github.com/Luminx-Labs/ZBra/commit/e52cd3e089972c4e03c078ef01cff2833e6db441)
- add Rust & Python workspace and organize the workspace ([ef7a4a0](https://github.com/Luminx-Labs/ZBra/commit/ef7a4a099bb5e40fd8f0ac0e3498bb90c5bbde7a)
- configure clippy ([7eda367](https://github.com/Luminx-Labs/ZBra/commit/7eda367098cbd0e7369b79df341902469bf7e5ac)
- Initial Commit ([b627010](https://github.com/Luminx-Labs/ZBra/commit/b6270108cc9a2ba1a9ebec8b5fccefd4f4ebb32f)
