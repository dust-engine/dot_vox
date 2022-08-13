# dot_vox

![](https://img.shields.io/crates/v/dot_vox.svg)
[![CI](https://github.com/dust-engine/dot_vox/actions/workflows/ci.yml/badge.svg)](https://github.com/dust-engine/dot_vox/actions/workflows/ci.yml)
![](https://docs.rs/dot_vox/badge.svg)

Rust parser for [MagicaVoxel](https://ephtracy.github.io/) .vox files, using
[Nom](https://github.com/Geal/nom).

## Current status

As of version 3.0.0, we no longer support files with the old `MATT` chunk. If you wish to use 
v3.0.0, please open your file in the latest version of MagicaVoxel and resave - this should 
switch you over to the newer dictionary-based `MATL` chunks. Alternatively, continue to use 
v2.0.0.

The [Dust Engine](https://github.com/dust-engine) project is currently maintaining this crate.
If there are any changes in the MagicaVoxel file format, feel free to open an issue or a PR, and we'll work to get them implemented.

## RustDoc

Kindly hosted over at https://docs.rs/dot_vox/.

## Thanks

`dot_vox` was originally developed by [@davidedmonds](https://github.com/davidedmonds) and many people have contributed to its development.

- [@davidedmonds](https://github.com/davidedmonds) (The original author of this crate)
- [@jice-nospam](https://github.com/jice-nospam) (first crack at palette parsing)
- [@expenses](https://github.com/expenses) (bug report on palette indexing)
- [@xMAC94x](https://github.com/xMAC94x) (bug report on material parsing)
- [@bonsairobo](https://github.com/bonsairobo) (implementation of `write_vox` functionality)
- [@Sixmorphugus](https://github.com/Sixmorphugus) (implementation of Scene Graph parsing)
- [@nickelc](https://github.com/nickelc) (migrating to nom 7 and modernizing the code base)
- [@InBetweenNames](https://github.com/InBetweenNames) (Rolled up changes in multiple PRs and added various helper methods)
- [@virtualritz](https://github.com/virtualritz) (Upgrading the codebase to Rust 2021)
