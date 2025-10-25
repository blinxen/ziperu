Changelog
=========

0.7.0
-----

### Changed

- Update all dependencies to their latest versions
- Add `ziperu::read::ZipArchive::file_exists` for checking if a file exists in a archive or not
- Bump MSRV to `1.85.0`
- Use target_has_atomic instead of hardcoding arches
- Relax methods that take in a `Path` to require on `AsRef<Path`
- Undeprecate `start_file_by_path` and `add_directory_by_path`
- Undeprecate `CompressionMethod::from_u16` and `CompressionMethod::to_u16`
- Add decompression support for deflate64
- Decrease minimum version from 46 (bzip2) to 10 (stored)

### Bugfixes

- https://github.com/zip-rs/zip-old/issues/154
- https://github.com/zip-rs/zip-old/issues/280
- https://github.com/zip-rs/zip-old/issues/432
- Fixed a bug where `..` was not considered when paths were normalized

[0.6.6]
-------

### Changed

- Updated `aes` dependency to `0.8.2` (https://github.com/zip-rs/zip/pull/354)

[0.6.5]
-------

### Changed

- Added experimental [`zip::unstable::write::FileOptions::with_deprecated_encryption`] API to enable encrypting files with PKWARE encryption.

[0.6.4]
-------

### Changed

 - [#333](https://github.com/zip-rs/zip/pull/333): disabled the default features of the `time` dependency, and also `formatting` and `macros`, as they were enabled by mistake.
 - Deprecated [`DateTime::from_time`](https://docs.rs/zip/0.6/zip/struct.DateTime.html#method.from_time) in favor of [`DateTime::try_from`](https://docs.rs/zip/0.6/zip/struct.DateTime.html#impl-TryFrom-for-DateTime)
