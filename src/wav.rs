use core::convert::TryFrom;

use crate::chunk::{parse_chunks, Chunk, ChunkTag};
use crate::error::Error;
use crate::fmt::Fmt;
use embedded_sdmmc::{BlockDevice, File, TimeSource};
use heapless::Vec;

pub(crate) const HEADER_SIZE: usize = 44;
pub(crate) const MAX_CHUNKS: usize = 20;

/// Enum to hold samples for different bit depths
#[derive(Debug)]
pub enum Data {
    /// 8 bit audio
    BitDepth8(u8),
    /// 16 bit audio
    BitDepth16(i16),
    /// 24 bit audio
    BitDepth24(i32),
}

/// Struct representing a WAV file
pub struct Wav<
    'a,
    BD: BlockDevice,
    TS: TimeSource,
    const MAX_DIRS: usize,
    const MAX_FILES: usize,
    const MAX_VOLUMES: usize,
> {
    pub file: File<'a, BD, TS, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    /// The Audio sample data
    pub data: Chunk,
    /// Contains data from the fmt chunk / header part of the file
    pub fmt: Fmt,
    /// Contains raw chunk data that is either unimplemented or unknown
    pub chunks: Vec<Chunk, MAX_CHUNKS>,
}

impl<
        'a,
        BD: BlockDevice,
        TS: TimeSource,
        const MAX_DIRS: usize,
        const MAX_FILES: usize,
        const MAX_VOLUMES: usize,
    > Wav<'a, BD, TS, MAX_DIRS, MAX_FILES, MAX_VOLUMES>
{
    /// Create new [`Wav`] instance from a embedded_sdmmc File
    ///
    pub fn new(
        mut file: File<'a, BD, TS, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    ) -> Result<Self, Error> {
        let mut bytes: [u8; HEADER_SIZE] = [0; HEADER_SIZE];
        let read = file.read(&mut bytes).unwrap();
        assert!(bytes.len() == read);
        let parsed_chunks = parse_chunks(&bytes)?;

        let fmt = parsed_chunks
            .iter()
            .find(|c| c.id == ChunkTag::Fmt)
            .ok_or(Error::NoFmtChunkFound)
            .and_then(|c| {
                let (start, end) = (c.start, c.end);
                Fmt::from_chunk(&bytes[start..end])
            })?;

        let data = parsed_chunks
            .iter()
            .find(|c| c.id == ChunkTag::Data)
            .ok_or(Error::NoDataChunkFound)?
            .clone();

        let chunks = parsed_chunks
            .into_iter()
            .filter(|c| c.id != ChunkTag::Data && c.id != ChunkTag::Fmt)
            .collect();

        file.seek_from_start(HEADER_SIZE as u32 + 1).unwrap();

        let wave = Wav {
            file,
            data,
            fmt,
            chunks,
        };

        Ok(wave)
    }

    pub fn is_end(&self) -> bool {
        self.file.offset() >= self.data.end as u32
    }

    pub fn next(&mut self) -> Result<Data, Error> {
        assert!(!self.is_end());

        match self.fmt.bit_depth {
            8 => {
                let mut buf: [u8; 1] = [0; 1];
                assert!(self.file.read(&mut buf).unwrap() == 1);
                Ok(Data::BitDepth8(buf[0]))
            }
            16 => {
                let mut buf: [u8; 2] = [0; 2];
                assert!(self.file.read(&mut buf).unwrap() == 2);
                Ok(Data::BitDepth16(i16::from_le_bytes([buf[0], buf[1]])))
            }
            24 => {
                let mut buf: [u8; 3] = [0; 3];
                assert!(self.file.read(&mut buf).unwrap() == 3);

                let sign = buf[2] >> 7;
                let sign_byte = if sign == 1 { 0xff } else { 0x0 };

                Ok(Data::BitDepth24(i32::from_le_bytes([
                    buf[0], buf[1], buf[2], sign_byte,
                ])))
            }
            _ => Err(Error::UnsupportedBitDepth(self.fmt.bit_depth)),
        }
    }
}
