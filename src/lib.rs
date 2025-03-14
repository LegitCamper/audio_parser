#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

use embedded_sdmmc_async::{BlockDevice, Controller, File, TimeSource, Volume};
use heapless::Vec;

mod wav;

/// Enum to hold samples for different bit depths
#[derive(Debug)]
pub enum BitDepth {
    /// 8 bit audio
    BitDepth8,
    /// 16 bit audio
    BitDepth16,
    /// 24 bit audio
    BitDepth24,
}

/// Represents Audio Format. Anything other than PCM needs to be decoded
#[derive(Debug)]
pub enum AudioFormat {
    Pcm,
}

/// Struct representing an audio file
pub struct AudioFile<const CHUNK_LEN: usize = 512> {
    file: File,
    /// How much read of the Audio section
    pub read: usize,
    /// The start of the audio section
    pub start: usize,
    /// The end of the audio section
    pub end: usize,
    pub audio_type: AudioFormat,
    /// sample rate, typical values are `44_100`, `48_000` or `96_000`
    pub sample_rate: u32,
    /// number of audio channels in the sample data, channels are interleaved
    pub num_channels: u16,
    /// bit depth for each sample, typical values are `16` or `24`
    pub bit_depth: u16,
}

impl<const CHUNK_LEN: usize> AudioFile<CHUNK_LEN> {
    pub async fn new_wav<'a, D: BlockDevice, TS: TimeSource>(
        mut file: File,
        sd_controller: &mut Controller<D, TS>,
        volume: &Volume,
    ) -> Result<Self, wav::error::Error> {
        let mut bytes: [u8; wav::HEADER_SIZE] = [0; wav::HEADER_SIZE];
        let read = sd_controller
            .read(volume, &mut file, &mut bytes)
            .await
            .unwrap();
        assert!(bytes.len() == read);
        let parsed_chunks = wav::chunk::parse_chunks(&bytes)?;

        let fmt = parsed_chunks
            .iter()
            .find(|c| c.id == wav::chunk::ChunkTag::Fmt)
            .ok_or(wav::error::Error::NoFmtChunkFound)
            .and_then(|c| {
                let (start, end) = (c.start, c.end);
                wav::fmt::Fmt::from_chunk(&bytes[start..end])
            })?;

        let data = parsed_chunks
            .iter()
            .find(|c| c.id == wav::chunk::ChunkTag::Data)
            .ok_or(wav::error::Error::NoDataChunkFound)?
            .clone();

        let chunks: Vec<wav::chunk::Chunk, 5> = parsed_chunks
            .into_iter()
            .filter(|c| c.id != wav::chunk::ChunkTag::Data && c.id != wav::chunk::ChunkTag::Fmt)
            .collect();

        file.seek_from_start(wav::HEADER_SIZE as u32 + 1).unwrap();

        Ok(AudioFile {
            file,
            read: data.start,
            start: data.start,
            end: data.end,
            audio_type: fmt.audio_format,
            sample_rate: fmt.sample_rate,
            num_channels: fmt.num_channels,
            bit_depth: fmt.bit_depth,
        })
    }

    pub async fn read_exact<'a, D: BlockDevice, TS: TimeSource>(
        &mut self,
        sd_controller: &mut Controller<D, TS>,
        volume: &Volume,
        into_buf: &mut [u8],
    ) -> usize {
        self.read += into_buf.len();
        // fill the file_buffer
        sd_controller
            .read(volume, &mut self.file, into_buf)
            .await
            .unwrap()
    }

    pub fn destroy(self) -> File {
        self.file
    }
}
