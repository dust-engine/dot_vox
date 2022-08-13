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

I'm not currently using MagicaVoxel, so am not keeping up with developments in the file format. If there are any
missing features, or problems loading more recent files, please don't hesitate to open an issue. I'll try to get back
to you within a day or two!

## RustDoc

Kindly hosted over at https://docs.rs/dot_vox/.

## Still to implement

* `nTRN` chunk
* `nGRP` chunk
* `nSHP` chunk

## Thanks

As a maintainer, its always nice to get bug reports and (even better) pull requests. Thanks go to all of the following
for doing just that!

- @jice-nospam (first crack at palette parsing)
- @expenses (bug report on palette indexing)
- @xMAC94x (bug report on material parsing)
- @bonsairobo (implementation of `write_vox` functionality)