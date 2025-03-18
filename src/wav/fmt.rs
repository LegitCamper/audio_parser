use crate::{wav::error::Error, AudioCodec};
use core::convert::TryInto;

/// Struct representing the `fmt_` section of a WAV file
///
/// for more information see [`here`]
///
/// [`here`]: http://soundfile.sapp.org/doc/WaveFormat/
pub struct Fmt {
    pub audio_format: AudioCodec,
    /// sample rate, typical values are `44_100`, `48_000` or `96_000`
    pub sample_rate: u32,
    /// number of audio channels in the sample data, channels are interleaved
    pub num_channels: u16,
    /// bit depth for each sample, typical values are `16` or `24`
    pub bit_depth: u16,
}

impl Fmt {
    pub(crate) fn from_chunk(bytes: &[u8]) -> Result<Self, Error> {
        let format = bytes[0..2]
            .try_into()
            .map_err(|_| Error::CantParseSliceInto)
            .map(|b| u16::from_le_bytes(b))?;

        // if format != 1 {
        //     return Err(Error::UnsupportedFormat(format));
        // }
        //
        let format = match format {
            1 => Ok(AudioCodec::UncompressedPcm),
            _ => Err(Error::UnsupportedFormat(format)),
        }?;

        let num_channels = bytes[2..4]
            .try_into()
            .map_err(|_| Error::CantParseSliceInto)
            .map(|b| u16::from_le_bytes(b))?;

        let sample_rate = bytes[4..8]
            .try_into()
            .map_err(|_| Error::CantParseSliceInto)
            .map(|b| u32::from_le_bytes(b))?;

        let bit_depth = bytes[14..16]
            .try_into()
            .map_err(|_| Error::CantParseSliceInto)
            .map(|b| u16::from_le_bytes(b))?;

        Ok(Fmt {
            audio_format: format,
            num_channels,
            sample_rate,
            bit_depth,
        })
    }
}
