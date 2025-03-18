use super::error::Error;
use core::convert::TryInto;
use heapless::Vec;

pub const MAX_CHUNKS: usize = 5;

/// RIFF chunks are tagged with 4 byte identifiers.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ChunkTag {
    /// Root level "chunk"
    Riff,
    /// Mandatory chunk for WAV files, contains data such as the sample rate, bit depth, and number of channels.
    Fmt,
    /// Mandatory chunk for WAV files, contains the (interleaved) samples.
    Data,
    /// File identifier, should be located right after the RIFF tag and chunk size
    Wave,
    /// Option Chunks that extend RIFF
    List,
    /// Optional List chunk for Metadata: Title, album, etc...
    Info,
    /// Unkown/unhandled chunk tag, useful for parsing [`Chunk`] bytes.
    Unknown([u8; 4]),
}

impl ChunkTag {
    fn from_bytes(bytes: &[u8; 4]) -> Self {
        match bytes {
            [b'R', b'I', b'F', b'F'] => ChunkTag::Riff,
            [b'f', b'm', b't', b' '] => ChunkTag::Fmt,
            [b'd', b'a', b't', b'a'] => ChunkTag::Data,
            [b'W', b'A', b'V', b'E'] => ChunkTag::Wave,
            _ => ChunkTag::Unknown(*bytes),
        }
    }

    fn to_bytes(self) -> [u8; 4] {
        match self {
            ChunkTag::Riff => [b'R', b'I', b'F', b'F'],
            ChunkTag::Fmt => [b'f', b'm', b't', b' '],
            ChunkTag::Data => [b'd', b'a', b't', b'a'],
            ChunkTag::Wave => [b'W', b'A', b'V', b'E'],
            ChunkTag::List => [b'L', b'I', b'S', b'T'],
            ChunkTag::Info => [b'I', b'N', b'F', b'O'],
            ChunkTag::Unknown(bytes) => bytes,
        }
    }
}

/// LIST chunks are tagged with 4 byte identifiers.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ListChunkTag {
    Iarl, // The location where the subject of the file is archived
    Iart, // The artist of the original subject of the file
    Icms, // The name of the person or organization that commissioned the original subject of the file
    Icmt, // General comments about the file or its subject
    Icop, // Copyright information about the file (e.g., "Copyright Some Company 2011")
    Icrd, // The date the subject of the file was created (creation date) (e.g., "2022-12-31")
    Icrp, // Whether and how an image was cropped
    Idim, // The dimensions of the original subject of the file
    Idpi, // Dots per inch settings used to digitize the file
    Ieng, // The name of the engineer who worked on the file
    Ignr, // The genre of the subject
    Ikey, // A list of keywords for the file or its subject
    Ilgt, // Lightness settings used to digitize the file
    Imed, // Medium for the original subject of the file
    Inam, // Title of the subject of the file (name)
    Iplt, // The number of colors in the color palette used to digitize the file
    Iprd, // Name of the title the subject was originally intended for
    Isbj, // Description of the contents of the file (subject)
    Isft, // Name of the software package used to create the file
    Isrc, // The name of the person or organization that supplied the original subject of the file
    Isrf, // The original form of the material that was digitized (source form)
    Itch, // The name of the technician who digitized the subject file
}

impl ListChunkTag {
    fn from_bytes(bytes: &[u8; 4]) -> Self {
        match bytes {
            [b'I', b'A', b'R', b'L'] => ListChunkTag::Iarl,
            [b'I', b'A', b'R', b'T'] => ListChunkTag::Iart,
            [b'I', b'C', b'M', b'S'] => ListChunkTag::Icms,
            [b'I', b'C', b'M', b'T'] => ListChunkTag::Icmt,
            [b'I', b'C', b'O', b'P'] => ListChunkTag::Icop,
            [b'I', b'C', b'R', b'D'] => ListChunkTag::Icrd,
            [b'I', b'C', b'R', b'P'] => ListChunkTag::Icrp,
            [b'I', b'D', b'I', b'M'] => ListChunkTag::Idim,
            [b'I', b'D', b'P', b'I'] => ListChunkTag::Idpi,
            [b'I', b'E', b'N', b'G'] => ListChunkTag::Ieng,
            [b'I', b'G', b'N', b'R'] => ListChunkTag::Ignr,
            [b'I', b'K', b'E', b'Y'] => ListChunkTag::Ikey,
            [b'I', b'L', b'G', b'T'] => ListChunkTag::Ilgt,
            [b'I', b'M', b'E', b'D'] => ListChunkTag::Imed,
            [b'I', b'N', b'A', b'M'] => ListChunkTag::Inam,
            [b'I', b'P', b'L', b'T'] => ListChunkTag::Iplt,
            [b'I', b'P', b'R', b'D'] => ListChunkTag::Iprd,
            [b'I', b'S', b'B', b'J'] => ListChunkTag::Isbj,
            [b'I', b'S', b'F', b'T'] => ListChunkTag::Isft,
            [b'I', b'S', b'R', b'C'] => ListChunkTag::Isrc,
            [b'I', b'S', b'R', b'F'] => ListChunkTag::Isrf,
            [b'I', b'T', b'C', b'H'] => ListChunkTag::Itch,
            _ => panic!("Unknown ListChunkTag: {:?}", bytes),
        }
    }

    fn to_bytes(self) -> [u8; 4] {
        match self {
            ListChunkTag::Iarl => [b'I', b'A', b'R', b'L'],
            ListChunkTag::Iart => [b'I', b'A', b'R', b'T'],
            ListChunkTag::Icms => [b'I', b'C', b'M', b'S'],
            ListChunkTag::Icmt => [b'I', b'C', b'M', b'T'],
            ListChunkTag::Icop => [b'I', b'C', b'O', b'P'],
            ListChunkTag::Icrd => [b'I', b'C', b'R', b'D'],
            ListChunkTag::Icrp => [b'I', b'C', b'R', b'P'],
            ListChunkTag::Idim => [b'I', b'D', b'I', b'M'],
            ListChunkTag::Idpi => [b'I', b'D', b'P', b'I'],
            ListChunkTag::Ieng => [b'I', b'E', b'N', b'G'],
            ListChunkTag::Ignr => [b'I', b'G', b'N', b'R'],
            ListChunkTag::Ikey => [b'I', b'K', b'E', b'Y'],
            ListChunkTag::Ilgt => [b'I', b'L', b'G', b'T'],
            ListChunkTag::Imed => [b'I', b'M', b'E', b'D'],
            ListChunkTag::Inam => [b'I', b'N', b'A', b'M'],
            ListChunkTag::Iplt => [b'I', b'P', b'L', b'T'],
            ListChunkTag::Iprd => [b'I', b'P', b'R', b'D'],
            ListChunkTag::Isbj => [b'I', b'S', b'B', b'J'],
            ListChunkTag::Isft => [b'I', b'S', b'F', b'T'],
            ListChunkTag::Isrc => [b'I', b'S', b'R', b'C'],
            ListChunkTag::Isrf => [b'I', b'S', b'R', b'F'],
            ListChunkTag::Itch => [b'I', b'T', b'C', b'H'],
        }
    }
}

/// Resource Interchange File Format (RIFF) tagged chunk.
#[derive(Debug, Clone, Copy)]
pub struct Chunk {
    /// Chunk tag
    pub id: ChunkTag,
    pub start: usize,
    pub end: usize,
}

impl Chunk {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let id = bytes[0..4]
            .try_into()
            .map_err(|_| Error::CantParseSliceInto)
            .map(ChunkTag::from_bytes)?;

        let size = bytes[4..8]
            .try_into()
            .map_err(|_| Error::CantParseSliceInto)
            .map(|b| u32::from_le_bytes(b))?;

        let start = 8 + 12;
        let end = 20 + size as usize;

        Ok(Chunk { id, start, end })
    }
}

pub fn parse_riff(bytes: &[u8]) -> Result<Vec<Chunk, MAX_CHUNKS>, Error> {
    let mut chunks: Vec<Chunk, MAX_CHUNKS> = Vec::new();

    let riff = Chunk::from_bytes(bytes)?;

    if riff.id != ChunkTag::Riff {
        return Err(Error::NoRiffChunkFound);
    }

    let tag: [u8; 4] = bytes[8..8 + 4].try_into().unwrap();
    if tag != ChunkTag::Wave.to_bytes() {
        return Err(Error::NoWaveTagFound);
    }

    // skip parsed bytes
    let mut index = 12;

    while index < bytes.len() {
        let chunk = &bytes[index..];
        let chunk_info = Chunk::from_bytes(chunk)?;

        // Chunks should always have an even number of bytes,
        // if it is odd there is an empty padding byte at the end
        let chunk_length = chunk_info.end - chunk_info.start;
        let padding_byte = (chunk_length & 1) * 8;

        index += 8 + chunk_length + padding_byte;

        chunks.push(chunk_info).unwrap();
    }

    Ok(chunks)
}

pub fn parse_list(bytes: &[u8]) -> Result<Vec<Chunk, MAX_CHUNKS>, Error> {
    let mut chunks: Vec<Chunk, MAX_CHUNKS> = Vec::new();

    let list = Chunk::from_bytes(bytes)?;

    if list.id != ChunkTag::List {
        return Err(Error::NoListTagFound);
    }

    // TODO: Check for other List Tags
    let tag: [u8; 4] = bytes[8..8 + 4].try_into().unwrap();
    if tag != ChunkTag::Info.to_bytes() {
        return Err(Error::NoInfoTagFound);
    }

    // skip parsed bytes
    let mut index = 12;

    while index < bytes.len() {
        let chunk = &bytes[index..];
        let chunk_info = Chunk::from_bytes(chunk)?;

        // Chunks should always have an even number of bytes,
        // if it is odd there is an empty padding byte at the end
        let chunk_length = chunk_info.end - chunk_info.start;
        let padding_byte = (chunk_length & 1) * 8;

        index += 8 + chunk_length + padding_byte;

        chunks.push(chunk_info).unwrap();
    }

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wav::error::Error;

    #[test]
    fn should_parse_chunks() {
        let bytes: [u8; 60] = [
            0x52, 0x49, 0x46, 0x46, // RIFF
            0x34, 0x00, 0x00, 0x00, // chunk size
            0x57, 0x41, 0x56, 0x45, // WAVE
            0x66, 0x6d, 0x74, 0x20, // fmt_
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x01, 0x00, // audio format
            0x02, 0x00, // num channels
            0x22, 0x56, 0x00, 0x00, // sample rate
            0x88, 0x58, 0x01, 0x00, // byte rate
            0x04, 0x00, // block align
            0x10, 0x00, // bits per sample
            0x64, 0x61, 0x74, 0x61, // data
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x00, 0x00, 0x00, 0x00, // sample 1 L+R
            0x24, 0x17, 0x1e, 0xf3, // sample 2 L+R
            0x3c, 0x13, 0x3c, 0x14, // sample 3 L+R
            0x16, 0xf9, 0x18, 0xf9, // sample 4 L+R
        ];

        let chunks = parse_riff(&bytes).unwrap();

        assert_eq!(chunks.len(), 2);
        assert!(chunks.iter().find(|c| c.id == ChunkTag::Fmt).is_some());
        assert!(chunks.iter().find(|c| c.id == ChunkTag::Data).is_some());
    }

    #[test]
    fn should_fail_on_non_wave_files() {
        let bytes: [u8; 60] = [
            0x52, 0x49, 0x46, 0x46, // RIFF
            0x34, 0x00, 0x00, 0x00, // chunk size
            0x57, 0x41, 0x56, 0x56, // WAVV
            0x66, 0x6d, 0x74, 0x20, // fmt_
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x01, 0x00, // audio format
            0x02, 0x00, // num channels
            0x22, 0x56, 0x00, 0x00, // sample rate
            0x88, 0x58, 0x01, 0x00, // byte rate
            0x04, 0x00, // block align
            0x10, 0x00, // bits per sample
            0x64, 0x61, 0x74, 0x61, // data
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x00, 0x00, 0x00, 0x00, // sample 1 L+R
            0x24, 0x17, 0x1e, 0xf3, // sample 2 L+R
            0x3c, 0x13, 0x3c, 0x14, // sample 3 L+R
            0x16, 0xf9, 0x18, 0xf9, // sample 4 L+R
        ];

        assert_eq!(parse_riff(&bytes).unwrap_err(), Error::NoWaveTagFound);
    }

    #[test]
    fn should_not_fail_with_random_chunks_added() {
        let bytes: [u8; 88] = [
            0x52, 0x49, 0x46, 0x46, // RIFF
            0x34, 0x00, 0x00, 0x00, // chunk size
            0x57, 0x41, 0x56, 0x56, // WAVV
            0x66, 0x6d, 0x74, 0x20, // fmt_
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x01, 0x00, // audio format
            0x02, 0x00, // num channels
            0x22, 0x56, 0x00, 0x00, // sample rate
            0x88, 0x58, 0x01, 0x00, // byte rate
            0x04, 0x00, // block align
            0x10, 0x00, // bits per sample
            0x72, 0x6e, 0x64, 0x6d, // rndm
            0x04, 0x00, 0x00, 0x00, // chunk size
            0xaa, 0xaa, 0xaa, 0xaa, // ...
            0x8b, 0xad, 0xf0, 0x0d, // 8badfood
            0x08, 0x00, 0x00, 0x00, // chunk size
            0xaa, 0xff, 0xaa, 0xff, // ...
            0xff, 0xaa, 0xff, 0xaa, // ...
            0x64, 0x61, 0x74, 0x61, // data
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x00, 0x00, 0x00, 0x00, // sample 1 L+R
            0x24, 0x17, 0x1e, 0xf3, // sample 2 L+R
            0x3c, 0x13, 0x3c, 0x14, // sample 3 L+R
            0x16, 0xf9, 0x18, 0xf9, // sample 4 L+R
        ];

        assert_eq!(parse_riff(&bytes).unwrap_err(), Error::NoWaveTagFound);
    }
}
