#![cfg_attr(not(test), no_std)]
// #![warn(missing_docs)]

use embedded_sdmmc::asynchronous::{BlockDevice, File, TimeSource};
use heapless::{String, Vec};

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
pub enum AudioCodec {
    /// Uncompressed PCM that does not need any decoding
    UncompressedPcm,
}

/// Metadata of the music
#[derive(Debug)]
pub struct Metadata<const MAX_STRING_LEN: usize> {
    artist: Option<String<MAX_STRING_LEN>>,
    title: Option<String<MAX_STRING_LEN>>,
    album: Option<String<MAX_STRING_LEN>>,
    keywords: Option<String<MAX_STRING_LEN>>,
    genre: Option<String<MAX_STRING_LEN>>,
    date: Option<String<MAX_STRING_LEN>>,
}

/// Struct representing an audio file
pub struct AudioFile<
    'a,
    D,
    T,
    const MAX_DIRS: usize,
    const MAX_FILES: usize,
    const MAX_VOLUMES: usize,
    const CHUNK_LEN: usize = 512,
> where
    D: BlockDevice,
    T: TimeSource,
{
    file: File<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    /// How much read of the Audio section
    pub read: usize,
    /// The start of the audio section
    pub start: usize,
    /// The end of the audio section
    pub end: usize,
    /// The audio codec of the read audio bytes
    pub audio_codec: AudioCodec,
    /// sample rate, typical values are `44_100`, `48_000` or `96_000`
    pub sample_rate: u32,
    /// number of audio channels in the sample data, channels are interleaved
    pub num_channels: u16,
    /// bit depth for each sample, typical values are `16` or `24`
    pub bit_depth: u16,
}

impl<
        'a,
        D,
        T,
        const MAX_DIRS: usize,
        const MAX_FILES: usize,
        const MAX_VOLUMES: usize,
        const CHUNK_LEN: usize,
    > AudioFile<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES, CHUNK_LEN>
where
    D: BlockDevice,
    T: TimeSource,
{
    /// Create a new audio file that should point to a .wav file
    pub async fn new_wav(
        file: File<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    ) -> Result<Self, wav::error::Error> {
        let mut bytes: [u8; wav::HEADER_SIZE] = [0; wav::HEADER_SIZE];
        let read = file.read(&mut bytes).await.unwrap();
        let mut parsed_chunks = wav::chunk::parse_riff(&bytes[..read])?;

        let fmt = parsed_chunks
            .iter()
            .find(|c| c.id == wav::chunk::ChunkTag::Fmt)
            .ok_or(wav::error::Error::NoFmtChunkFound)
            .and_then(|c| {
                let (start, end) = (c.start, c.end);
                wav::fmt::Fmt::from_chunk(&bytes[start..end])
            })?;

        let data = match parsed_chunks
            .iter()
            .find(|c| c.id == wav::chunk::ChunkTag::Data)
        {
            Some(data) => data,
            None => {
                // Another chunk is where data was expected to be
                file.seek_from_start(36).unwrap(); // The end of fmt
                let read = file.read(&mut bytes).await.unwrap();
                parsed_chunks = wav::chunk::parse_list(&bytes[..read])?;

                let info = parsed_chunks
                    .iter()
                    .find(|c| c.id == wav::chunk::ChunkTag::Info)
                    .ok_or(wav::error::Error::NoInfoTagFound)?;

                parsed_chunks
                    .iter()
                    .find(|c| c.id == wav::chunk::ChunkTag::Data)
                    .ok_or(wav::error::Error::NoDataChunkFound)?
            }
        }
        .clone();

        let _chunks: Vec<wav::chunk::Chunk, 5> = parsed_chunks
            .into_iter()
            .filter(|c| c.id != wav::chunk::ChunkTag::Data && c.id != wav::chunk::ChunkTag::Fmt)
            .collect();

        // Go to the start of Data
        file.seek_from_start(data.start as u32).unwrap();

        Ok(AudioFile {
            file,
            read: data.start,
            start: data.start,
            end: data.end,
            audio_codec: fmt.audio_format,
            sample_rate: fmt.sample_rate,
            num_channels: fmt.num_channels,
            bit_depth: fmt.bit_depth,
        })
    }

    /// Reads bytes from opened file into the provided buffer and returns the number of bytes written
    pub async fn read_exact(&mut self, buf: &mut [u8]) -> usize {
        let read = self.file.read(buf).await.unwrap();
        self.read += read;
        read
    }

    /// Destroy the AudioFile returning the underlying File
    pub fn destroy(self) -> File<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES> {
        self.file
    }
}
