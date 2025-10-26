ziperu
======
[![Crates.io version](https://img.shields.io/crates/v/ziperu.svg)](https://crates.io/crates/ziperu)

[Documentation](https://docs.rs/ziperu/)

*This is a fork of the original zip crate (version 0.6.6) before the new maintainer took over.
My goal is to a have more stable library without any frequent breaking changes.*

Info
----

A zip library for rust which supports reading and writing of simple ZIP files.

Supported compression formats:

* stored (i.e. none)
* deflate
* bzip2
* zstd
* lzma
* xz

Currently unsupported zip extensions:

* Encryption
* Multi-disk

Usage
-----

The features available are:

* `aes-crypto`: Enables decryption of files which were encrypted with AES. Supports AE-1 and AE-2 methods.
* `deflate`: Enables the deflate compression algorithm, which is the default for zip files.
* `deflate64`: Enables the deflate64 compression algorithm. Only decompression is supported.
* `bzip2`: Enables the BZip2 compression algorithm.
* `time`: Enables features using the [time](https://github.com/time-rs/time) crate.
* `zstd`: Enables the Zstandard compression algorithm.
* `lzma`: Enables the LZMA compression algorithm.
* `xz`: Enables the XZ compression algorithm.

By default the following features are enabled:

* `aes-crypto`
* `bzip2`
* `deflate`
* `time`
* `zstd`

MSRV
----

The MSRV is kept as low as possible. It will only be updated when a dependency
forces the update.
Every MSRV update will bump the minor version of this crate pre 1.0.

Examples
--------

See the [examples directory](examples) for:
   * How to write a file to a zip.
   * How to write a directory of files to a zip (using [walkdir](https://github.com/BurntSushi/walkdir)).
   * How to extract a zip file.
   * How to extract a single file from a zip.
   * How to read a zip from the standard input.

Fuzzing
-------

Fuzzing support is through [cargo fuzz](https://github.com/rust-fuzz/cargo-fuzz). To install cargo fuzz:

```bash
cargo install cargo-fuzz
```

To list fuzz targets:

```bash
cargo +nightly fuzz list
```

To start fuzzing zip extraction:

```bash
cargo +nightly fuzz run fuzz_read
```
