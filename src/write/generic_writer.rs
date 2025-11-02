use std::io::{Error, ErrorKind, Seek, Write};
use crate::{result::{ZipError, ZipResult}, write::MaybeEncrypted, CompressionMethod};

#[cfg(any(
    feature = "deflate",
    feature = "deflate-miniz",
    feature = "deflate-zlib"
))]
use flate2::write::DeflateEncoder;

#[cfg(feature = "bzip2")]
use bzip2::write::BzEncoder;

#[cfg(feature = "zstd")]
use zstd::stream::write::Encoder as ZstdEncoder;

pub(crate) enum GenericZipWriter<W: Write + Seek> {
    Closed,
    Storer(MaybeEncrypted<W>),
    #[cfg(any(
        feature = "deflate",
        feature = "deflate-miniz",
        feature = "deflate-zlib"
    ))]
    Deflater(DeflateEncoder<MaybeEncrypted<W>>),
    #[cfg(feature = "bzip2")]
    Bzip2(BzEncoder<MaybeEncrypted<W>>),
    #[cfg(feature = "zstd")]
    Zstd(ZstdEncoder<'static, MaybeEncrypted<W>>),
    #[cfg(feature = "lzma")]
    Lzma(Box<lzma_rust2::LzmaWriter<MaybeEncrypted<W>>>),
    #[cfg(feature = "xz")]
    Xz(Box<lzma_rust2::XzWriter<MaybeEncrypted<W>>>),
}

impl<W: Write + Seek> GenericZipWriter<W> {
    pub(super) fn switch_to(
        &mut self,
        compression: CompressionMethod,
        compression_level: Option<i32>,
    ) -> ZipResult<()> {
        match self.current_compression() {
            Some(method) if method == compression => return Ok(()),
            None => {
                return Err(Error::new(
                    ErrorKind::BrokenPipe,
                    "ZipWriter was already closed",
                )
                .into());
            }
            _ => {}
        }

        let bare = match std::mem::replace(self, GenericZipWriter::Closed) {
            GenericZipWriter::Storer(w) => w,
            #[cfg(any(
                feature = "deflate",
                feature = "deflate-miniz",
                feature = "deflate-zlib"
            ))]
            GenericZipWriter::Deflater(w) => w.finish()?,
            #[cfg(feature = "bzip2")]
            GenericZipWriter::Bzip2(w) => w.finish()?,
            #[cfg(feature = "zstd")]
            GenericZipWriter::Zstd(w) => w.finish()?,
            #[cfg(feature = "lzma")]
            GenericZipWriter::Lzma(w) => w.finish()?,
            #[cfg(feature = "xz")]
            GenericZipWriter::Xz(w) => w.finish()?,
            GenericZipWriter::Closed => {
                return Err(Error::new(
                    ErrorKind::BrokenPipe,
                    "ZipWriter was already closed",
                )
                .into());
            }
        };

        *self = {
            match compression {
                CompressionMethod::Stored => {
                    if compression_level.is_some() {
                        return Err(ZipError::UnsupportedArchive(
                            "Unsupported compression level",
                        ));
                    }

                    GenericZipWriter::Storer(bare)
                }
                #[cfg(any(
                    feature = "deflate",
                    feature = "deflate-miniz",
                    feature = "deflate-zlib"
                ))]
                CompressionMethod::Deflated => GenericZipWriter::Deflater(DeflateEncoder::new(
                    bare,
                    flate2::Compression::new(
                        clamp_opt(
                            compression_level
                                .unwrap_or(flate2::Compression::default().level() as i32),
                            deflate_compression_level_range(),
                        )
                        .ok_or(ZipError::UnsupportedArchive(
                            "Unsupported compression level",
                        ))? as u32,
                    ),
                )),
                #[cfg(feature = "deflate64")]
                CompressionMethod::Deflate64 => {
                    return Err(ZipError::UnsupportedArchive(
                        "Compression using deflate64 is currently not supported",
                    ));
                }
                #[cfg(feature = "bzip2")]
                CompressionMethod::Bzip2 => GenericZipWriter::Bzip2(BzEncoder::new(
                    bare,
                    bzip2::Compression::new(
                        clamp_opt(
                            compression_level
                                .unwrap_or(bzip2::Compression::default().level() as i32),
                            bzip2_compression_level_range(),
                        )
                        .ok_or(ZipError::UnsupportedArchive(
                            "Unsupported compression level",
                        ))? as u32,
                    ),
                )),
                CompressionMethod::AES => {
                    return Err(ZipError::UnsupportedArchive(
                        "AES compression is not supported for writing",
                    ));
                }
                #[cfg(feature = "zstd")]
                CompressionMethod::Zstd => GenericZipWriter::Zstd(
                    ZstdEncoder::new(
                        bare,
                        clamp_opt(
                            compression_level.unwrap_or(zstd::DEFAULT_COMPRESSION_LEVEL),
                            zstd::compression_level_range(),
                        )
                        .ok_or(ZipError::UnsupportedArchive(
                            "Unsupported compression level",
                        ))?,
                    )
                    .map_err(ZipError::Io)?,
                ),
                #[cfg(feature = "lzma")]
                CompressionMethod::Lzma => GenericZipWriter::Lzma(Box::new(
                    lzma_rust2::LzmaWriter::new_no_header(
                        bare,
                        &lzma_rust2::LzmaOptions::with_preset(
                            clamp_opt(compression_level.unwrap_or(6), 0..=9)
                                .ok_or(ZipError::UnsupportedArchive(
                                    "Unsupported compression level",
                                ))? as u32,
                        ),
                        false,
                    )
                    .map_err(ZipError::Io)?,
                )),
                #[cfg(feature = "xz")]
                CompressionMethod::Xz => GenericZipWriter::Xz(Box::new(
                    lzma_rust2::XzWriter::new(
                        bare,
                        lzma_rust2::XzOptions::with_preset(
                            clamp_opt(compression_level.unwrap_or(6), 0..=9)
                                .ok_or(ZipError::UnsupportedArchive(
                                    "Unsupported compression level",
                                ))? as u32,
                        ),
                    )
                    .map_err(ZipError::Io)?,
                )),
                CompressionMethod::Unsupported(..) => {
                    return Err(ZipError::UnsupportedArchive("Unsupported compression"));
                }
            }
        };

        Ok(())
    }

    pub(super) fn ref_mut(&mut self) -> Option<&mut dyn Write> {
        match *self {
            GenericZipWriter::Storer(ref mut w) => Some(w as &mut dyn Write),
            #[cfg(any(
                feature = "deflate",
                feature = "deflate-miniz",
                feature = "deflate-zlib"
            ))]
            GenericZipWriter::Deflater(ref mut w) => Some(w as &mut dyn Write),
            #[cfg(feature = "bzip2")]
            GenericZipWriter::Bzip2(ref mut w) => Some(w as &mut dyn Write),
            #[cfg(feature = "zstd")]
            GenericZipWriter::Zstd(ref mut w) => Some(w as &mut dyn Write),
            #[cfg(feature = "lzma")]
            GenericZipWriter::Lzma(ref mut w) => Some(w as &mut dyn Write),
            #[cfg(feature = "xz")]
            GenericZipWriter::Xz(ref mut w) => Some(w as &mut dyn Write),
            GenericZipWriter::Closed => None,
        }
    }

    pub(super) fn is_closed(&self) -> bool {
        matches!(*self, GenericZipWriter::Closed)
    }

    pub(super) fn get_plain(&mut self) -> &mut W {
        match *self {
            GenericZipWriter::Storer(MaybeEncrypted::Unencrypted(ref mut w)) => w,
            _ => panic!("Should have switched to stored and unencrypted beforehand"),
        }
    }

    fn current_compression(&self) -> Option<CompressionMethod> {
        match *self {
            GenericZipWriter::Storer(..) => Some(CompressionMethod::Stored),
            #[cfg(any(
                feature = "deflate",
                feature = "deflate-miniz",
                feature = "deflate-zlib"
            ))]
            GenericZipWriter::Deflater(..) => Some(CompressionMethod::Deflated),
            #[cfg(feature = "bzip2")]
            GenericZipWriter::Bzip2(..) => Some(CompressionMethod::Bzip2),
            #[cfg(feature = "zstd")]
            GenericZipWriter::Zstd(..) => Some(CompressionMethod::Zstd),
            #[cfg(feature = "lzma")]
            GenericZipWriter::Lzma(..) => Some(CompressionMethod::Lzma),
            #[cfg(feature = "xz")]
            GenericZipWriter::Xz(..) => Some(CompressionMethod::Xz),
            GenericZipWriter::Closed => None,
        }
    }

    pub(super) fn unwrap(self) -> W {
        match self {
            GenericZipWriter::Storer(MaybeEncrypted::Unencrypted(w)) => w,
            _ => panic!("Should have switched to stored and unencrypted beforehand"),
        }
    }
}

#[cfg(any(
    feature = "deflate",
    feature = "deflate-miniz",
    feature = "deflate-zlib"
))]
fn deflate_compression_level_range() -> std::ops::RangeInclusive<i32> {
    let min = flate2::Compression::none().level() as i32;
    let max = flate2::Compression::best().level() as i32;
    min..=max
}

#[cfg(feature = "bzip2")]
fn bzip2_compression_level_range() -> std::ops::RangeInclusive<i32> {
    let min = bzip2::Compression::fast().level() as i32;
    let max = bzip2::Compression::best().level() as i32;
    min..=max
}

#[cfg(any(
    feature = "deflate",
    feature = "deflate-miniz",
    feature = "deflate-zlib",
    feature = "bzip2",
    feature = "zstd"
))]
fn clamp_opt<T: Ord + Copy>(value: T, range: std::ops::RangeInclusive<T>) -> Option<T> {
    if range.contains(&value) {
        Some(value)
    } else {
        None
    }
}
