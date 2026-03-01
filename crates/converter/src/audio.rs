use std::io::Cursor;

use symphonia::core::{
    audio::{Channels, SampleBuffer},
    codecs::{CODEC_TYPE_NULL, DecoderOptions},
    errors::Error as SymphoniaError,
    formats::{FormatOptions, Packet},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

/// Decode any supported audio format to interleaved f32 PCM.
/// Returns `(samples, channel_count, sample_rate_hz)`.
fn decode_to_pcm(input: &[u8], extension: &str) -> Result<(Vec<f32>, u16, u32), String> {
    let mss = MediaSourceStream::new(Box::new(Cursor::new(input.to_vec())), Default::default());

    let mut hint = Hint::new();
    hint.with_extension(extension);

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Unsupported audio format: {e}"))?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "No audio track found in file".to_string())?
        .clone();

    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track
        .codec_params
        .channels
        .map(|c: Channels| c.count() as u16)
        .unwrap_or(2);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Failed to create audio decoder: {e}"))?;

    let mut samples: Vec<f32> = Vec::new();

    loop {
        let packet: Packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::ResetRequired) => {
                decoder.reset();
                continue;
            }
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => return Err(format!("Error reading audio packet: {e}")),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            // Skip isolated corrupted frames rather than aborting
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => return Err(format!("Audio decode error: {e}")),
        };

        let spec = *decoded.spec();
        let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        buf.copy_interleaved_ref(decoded);
        samples.extend_from_slice(buf.samples());
    }

    Ok((samples, channels, sample_rate))
}

/// Encode interleaved f32 PCM samples to 16-bit signed integer PCM WAV.
fn encode_wav(samples: &[f32], channels: u16, sample_rate: u32) -> Result<Vec<u8>, String> {
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buf = Cursor::new(Vec::new());
    let mut writer =
        hound::WavWriter::new(&mut buf, spec).map_err(|e| format!("WAV writer error: {e}"))?;

    for &s in samples {
        let val = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer
            .write_sample(val)
            .map_err(|e| format!("WAV write error: {e}"))?;
    }

    writer
        .finalize()
        .map_err(|e| format!("WAV finalize error: {e}"))?;

    Ok(buf.into_inner())
}

/// Convert a supported input audio format to 16-bit PCM WAV.
///
/// Supported inputs: `wav`, `mp3`, `flac`, `ogg`
pub fn to_wav(input: &[u8], from_ext: &str) -> Result<Vec<u8>, String> {
    let (samples, channels, sample_rate) = decode_to_pcm(input, from_ext)?;
    encode_wav(&samples, channels, sample_rate)
}
