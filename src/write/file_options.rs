use std::convert::TryInto;

use crate::{compression::CompressionMethod, DateTime};
#[cfg(feature = "time")]
use time::OffsetDateTime;

/// Metadata for a file to be written
#[derive(Copy, Clone)]
pub struct FileOptions {
    pub(super) compression_method: CompressionMethod,
    pub(super) compression_level: Option<i32>,
    pub(super) last_modified_time: DateTime,
    pub(super) permissions: Option<u32>,
    pub(super) large_file: bool,
    pub(super) encrypt_with: Option<crate::zipcrypto::ZipCryptoKeys>,
}

impl FileOptions {
    /// Set the compression method for the new file
    ///
    /// The default is `CompressionMethod::Deflated`. If the deflate compression feature is
    /// disabled, `CompressionMethod::Stored` becomes the default.
    #[must_use]
    pub fn compression_method(mut self, method: CompressionMethod) -> FileOptions {
        self.compression_method = method;
        self
    }

    /// Set the compression level for the new file
    ///
    /// `None` value specifies default compression level.
    ///
    /// Range of values depends on compression method:
    /// * `Deflated`: 0 - 9. Default is 6
    /// * `Bzip2`: 1 - 9. Default is 6
    /// * `Zstd`: -7 - 22, with zero being mapped to default level. Default is 3
    /// * others: only `None` is allowed
    #[must_use]
    pub fn compression_level(mut self, level: Option<i32>) -> FileOptions {
        self.compression_level = level;
        self
    }

    /// Set the last modified time
    ///
    /// The default is the current timestamp if the 'time' feature is enabled, and 1980-01-01
    /// otherwise
    #[must_use]
    pub fn last_modified_time(mut self, mod_time: DateTime) -> FileOptions {
        self.last_modified_time = mod_time;
        self
    }

    /// Set the permissions for the new file.
    ///
    /// The format is represented with unix-style permissions.
    /// The default is `0o644`, which represents `rw-r--r--` for files,
    /// and `0o755`, which represents `rwxr-xr-x` for directories.
    ///
    /// This method only preserves the file permissions bits (via a `& 0o777`) and discards
    /// higher file mode bits. So it cannot be used to denote an entry as a directory,
    /// symlink, or other special file type.
    #[must_use]
    pub fn unix_permissions(mut self, mode: u32) -> FileOptions {
        self.permissions = Some(mode & 0o777);
        self
    }

    /// Set whether the new file's compressed and uncompressed size is less than 4 GiB.
    ///
    /// If set to `false` and the file exceeds the limit, an I/O error is thrown. If set to `true`,
    /// readers will require ZIP64 support and if the file does not exceed the limit, 20 B are
    /// wasted. The default is `false`.
    #[must_use]
    pub fn large_file(mut self, large: bool) -> FileOptions {
        self.large_file = large;
        self
    }
    pub(crate) fn with_deprecated_encryption(mut self, password: &[u8]) -> FileOptions {
        self.encrypt_with = Some(crate::zipcrypto::ZipCryptoKeys::derive(password));
        self
    }
}

impl Default for FileOptions {
    /// Construct a new FileOptions object
    fn default() -> Self {
        Self {
            #[cfg(any(
                feature = "deflate",
                feature = "deflate-miniz",
                feature = "deflate-zlib"
            ))]
            compression_method: CompressionMethod::Deflated,
            #[cfg(not(any(
                feature = "deflate",
                feature = "deflate-miniz",
                feature = "deflate-zlib"
            )))]
            compression_method: CompressionMethod::Stored,
            compression_level: None,
            #[cfg(feature = "time")]
            last_modified_time: OffsetDateTime::now_utc().try_into().unwrap_or_default(),
            #[cfg(not(feature = "time"))]
            last_modified_time: DateTime::default(),
            permissions: None,
            large_file: false,
            encrypt_with: None,
        }
    }
}
