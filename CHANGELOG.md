4.1.0
=====

- Feature: Added the ability to write out data in `.vox` format.

4.0.0
=====

- Migrated to use Nom 4 which involved a bit of a rewrite.

3.1.0
=====

- Feature: Can load `.vox` data from a `byte[]`.

3.0.0
=====

- Breaking Change - MagicaVoxel changed the way they output material chunks.
  This version of `dot_vox` removes the ability to parse the old material format
  as it is possible to migrate files by re-saving them in a recent version of 
  MagicaVoxel.

2.0.0
=====

- Migrated to use Nom 3, and improved the parsing method.

1.0.1
=====

- Bugfix: When parsing some materials, a panic would occur. This now emits an
  `Unknown` material type instead.

1.0.0
=====

- First stable release.