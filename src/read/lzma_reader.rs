use std::io::{Error, ErrorKind, Read, Result};

/// Contains the state of the actual lzma reader to allow lazy reading
enum ReaderState<R> {
    Uninitialize {
        reader: Option<R>,
        uncompressed_size: u64,
    },
    Initialized {
        reader: Box<lzma_rust2::LzmaReader<R>>,
    },
}

pub struct LzmaReader<R> {
    state: ReaderState<R>,
}

impl<R: Read> LzmaReader<R> {
    pub fn new(reader: R, uncompressed_size: u64) -> Self {
        LzmaReader {
            state: ReaderState::Uninitialize {
                reader: Some(reader),
                uncompressed_size,
            },
        }
    }

    pub fn into_inner(self) -> R {
        match self.state {
            ReaderState::Uninitialize { mut reader, .. } => reader.take().unwrap(),
            ReaderState::Initialized { reader } => reader.into_inner(),
        }
    }
}

impl<R: Read> Read for LzmaReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match &mut self.state {
            ReaderState::Uninitialize {
                reader,
                uncompressed_size,
            } => {
                let mut reader = reader
                    .take()
                    .ok_or_else(|| Error::other("Reader was not set for LZMA"))?;
                // 5.8.8.1 LZMA Version Information & 5.8.8.2 LZMA Properties Size
                let mut header = [0; 4];
                reader.read_exact(&mut header)?;
                let _version_information = u16::from_le_bytes(header[0..2].try_into().unwrap());
                let properties_size = u16::from_le_bytes(header[2..4].try_into().unwrap());
                if properties_size != 5 {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!("unexpected LZMA properties size of {properties_size}"),
                    ));
                }

                let mut props_data = [0; 5];
                reader.read_exact(&mut props_data)?;
                let props = props_data[0];
                let dict_size = u32::from_le_bytes(props_data[1..5].try_into().unwrap());

                // We don't need to handle the end-of-stream marker here, since the LZMA reader
                // stops at the end-of-stream marker OR when it has decoded uncompressed_size bytes, whichever comes first.
                let mut lzma_reader = lzma_rust2::LzmaReader::new_with_props(
                    reader,
                    *uncompressed_size,
                    props,
                    dict_size,
                    None,
                )?;

                let res = lzma_reader.read(buf);
                self.state = ReaderState::Initialized {
                    reader: Box::new(lzma_reader),
                };

                res
            }
            ReaderState::Initialized { reader } => reader.read(buf),
        }
    }
}
