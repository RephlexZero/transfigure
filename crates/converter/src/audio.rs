//! Audio: decode with symphonia, encode WAV (hound), FLAC (flacenc) or MP3
//! (rusty_mp3). Everything is pure Rust.

use std::io::Cursor;

use symphonia::core::{
    audio::SampleBuffer,
    codecs::{CODEC_TYPE_NULL, DecoderOptions},
    errors::Error as SymphoniaError,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

pub struct Pcm {
    /// Interleaved samples in [-1, 1].
    pub samples: Vec<f32>,
    pub channels: u16,
    pub rate: u32,
    /// Bit depth of the source when it was lossless, so 24-bit masters stay 24-bit.
    pub bits: Option<u32>,
}

pub fn decode(input: &[u8], extension: &str) -> Result<Pcm, String> {
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
        .map_err(|e| format!("Unsupported or damaged audio file: {e}"))?;
    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "No audio track found in file".to_string())?
        .clone();
    let lossless = matches!(extension, "flac" | "wav" | "aiff" | "caf")
        || track.codec_params.codec == symphonia::core::codecs::CODEC_TYPE_ALAC;
    let bits = track
        .codec_params
        .bits_per_sample
        .filter(|_| lossless)
        .map(|b| if b > 16 { 24 } else { 16 });

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Unsupported audio codec: {e}"))?;

    let mut samples = Vec::new();
    // Taken from the decoded audio, not the container header: MP3 and ADTS
    // headers often omit the channel count, and guessing wrong doubles the
    // playback speed.
    let mut spec: Option<(u16, u32)> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::ResetRequired) => {
                decoder.reset();
                continue;
            }
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => return Err(format!("Error reading audio: {e}")),
        };
        if packet.track_id() != track.id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            // Skip an isolated corrupt frame rather than failing the file.
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => return Err(format!("Audio decode error: {e}")),
        };
        let s = *decoded.spec();
        let this = (s.channels.count() as u16, s.rate);
        match spec {
            None => spec = Some(this),
            // A mid-stream format change can't be represented; stop there.
            Some(prev) if prev != this => break,
            _ => {}
        }
        if decoded.frames() == 0 {
            continue;
        }
        let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, s);
        buf.copy_interleaved_ref(decoded);
        samples.extend_from_slice(buf.samples());
    }

    let (channels, rate) = spec.ok_or_else(|| "The file contains no audio".to_string())?;
    Ok(Pcm {
        samples,
        channels,
        rate,
        bits,
    })
}

pub fn convert(input: &[u8], from: &str, to: &str, quality: Option<u8>) -> Result<Vec<u8>, String> {
    let pcm = decode(input, from)?;
    match to {
        "wav" => encode_wav(&pcm),
        "flac" => encode_flac(&pcm),
        "mp3" => encode_mp3(&pcm, quality),
        _ => Err(format!("Unsupported audio output: {to}")),
    }
}

fn quantize(s: f32, bits: u32) -> i32 {
    let max = ((1i64 << (bits - 1)) - 1) as f32;
    (s.clamp(-1.0, 1.0) * max).round() as i32
}

fn encode_wav(pcm: &Pcm) -> Result<Vec<u8>, String> {
    let bits = pcm.bits.unwrap_or(16);
    let spec = hound::WavSpec {
        channels: pcm.channels,
        sample_rate: pcm.rate,
        bits_per_sample: bits as u16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut buf = Cursor::new(Vec::new());
    let mut w = hound::WavWriter::new(&mut buf, spec).map_err(|e| format!("WAV error: {e}"))?;
    for &s in &pcm.samples {
        let r = if bits == 16 {
            w.write_sample(quantize(s, 16) as i16)
        } else {
            w.write_sample(quantize(s, bits))
        };
        r.map_err(|e| format!("WAV error: {e}"))?;
    }
    w.finalize().map_err(|e| format!("WAV error: {e}"))?;
    Ok(buf.into_inner())
}

fn encode_flac(pcm: &Pcm) -> Result<Vec<u8>, String> {
    use flacenc::component::BitRepr;
    use flacenc::error::Verify;
    let bits = pcm.bits.unwrap_or(16);
    let ints: Vec<i32> = pcm.samples.iter().map(|&s| quantize(s, bits)).collect();
    let config = flacenc::config::Encoder::default()
        .into_verified()
        .map_err(|e| format!("FLAC config error: {e:?}"))?;
    let source = flacenc::source::MemSource::from_samples(
        &ints,
        pcm.channels as usize,
        bits as usize,
        pcm.rate as usize,
    );
    let stream = flacenc::encode_with_fixed_block_size(&config, source, config.block_size)
        .map_err(|e| format!("FLAC encode error: {e:?}"))?;
    let mut sink = flacenc::bitsink::ByteSink::new();
    stream
        .write(&mut sink)
        .map_err(|e| format!("FLAC write error: {e:?}"))?;
    let mut out = sink.as_slice().to_vec();
    // STREAMINFO's minimum block size must exclude the final (short) block.
    // flacenc counts it, which makes a fixed-block-size stream look variable
    // and strict decoders reject the file; copy the maximum over it.
    if out.len() > 12 && &out[..4] == b"fLaC" {
        let (max_hi, max_lo) = (out[10], out[11]);
        out[8] = max_hi;
        out[9] = max_lo;
    }
    Ok(out)
}

/// Quality slider (10–100) to a constant MP3 bitrate.
fn mp3_bitrate(quality: Option<u8>) -> u32 {
    match quality.unwrap_or(85) {
        0..=30 => 96,
        31..=50 => 128,
        51..=70 => 160,
        71..=88 => 192,
        89..=95 => 256,
        _ => 320,
    }
}

const MP3_RATES: [u32; 9] = [48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000];

fn encode_mp3(pcm: &Pcm, quality: Option<u8>) -> Result<Vec<u8>, String> {
    // MP3 carries at most two channels: fold surround down to stereo.
    let (mut samples, channels) = if pcm.channels > 2 {
        (downmix_stereo(&pcm.samples, pcm.channels as usize), 2u16)
    } else {
        (pcm.samples.clone(), pcm.channels)
    };
    let mut rate = pcm.rate;
    if !MP3_RATES.contains(&rate) {
        // e.g. 96 kHz or 88.2 kHz masters.
        let target = if rate.is_multiple_of(11025) {
            44100
        } else {
            48000
        };
        samples = resample(&samples, channels as usize, rate, target);
        rate = target;
    }
    let mut enc = rusty_mp3::Mp3Encoder::new(rusty_mp3::Mp3EncoderConfig {
        bitrate_kbps: mp3_bitrate(quality),
        vbr_quality: None,
    });
    enc.push_pcm_f32(&samples, channels as _, rate)
        .map_err(|e| format!("MP3 encode error: {e:?}"))?;
    enc.finish();
    let mut out = Vec::new();
    while let Ok(packet) = enc.next_packet() {
        out.extend_from_slice(&packet);
    }
    if out.is_empty() {
        return Err("MP3 encoder produced no output".into());
    }
    Ok(out)
}

/// ITU-R BS.775 style fold-down, assuming the usual L R C LFE Ls Rs order.
fn downmix_stereo(samples: &[f32], ch: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(samples.len() / ch * 2);
    let g = std::f32::consts::FRAC_1_SQRT_2;
    for frame in samples.chunks_exact(ch) {
        let (l, r) = (frame[0], frame[1]);
        let c = frame.get(2).copied().unwrap_or(0.0);
        let ls = frame.get(4).copied().unwrap_or(0.0);
        let rs = frame.get(5).copied().unwrap_or(0.0);
        let norm = 1.0 / (1.0 + g + g);
        out.push((l + g * c + g * ls) * norm);
        out.push((r + g * c + g * rs) * norm);
    }
    out
}

/// Windowed-sinc resampler. Good enough for sample-rate changes ahead of a
/// lossy encoder; low-passes at the lower Nyquist to avoid aliasing.
fn resample(samples: &[f32], ch: usize, from: u32, to: u32) -> Vec<f32> {
    debug_assert!(ch <= 8);
    let frames = samples.len() / ch;
    let ratio = to as f64 / from as f64;
    let out_frames = (frames as f64 * ratio).floor() as usize;
    let cutoff = ratio.min(1.0) * 0.95;
    let half = 16i64;
    let mut out = vec![0.0f32; out_frames * ch];
    for o in 0..out_frames {
        let pos = o as f64 / ratio;
        let center = pos.floor() as i64;
        let mut acc = [0.0f64; 8];
        let mut wsum = 0.0f64;
        for k in (center - half + 1)..=(center + half) {
            if k < 0 || k as usize >= frames {
                continue;
            }
            let t = pos - k as f64;
            let x = std::f64::consts::PI * t * cutoff;
            let sinc = if x.abs() < 1e-9 { 1.0 } else { x.sin() / x };
            let win = 0.5 + 0.5 * (std::f64::consts::PI * t / half as f64).cos();
            let w = sinc * win;
            wsum += w;
            for (c, a) in acc.iter_mut().take(ch).enumerate() {
                *a += samples[k as usize * ch + c] as f64 * w;
            }
        }
        for (c, a) in acc.iter().take(ch).enumerate() {
            out[o * ch + c] = if wsum.abs() > 1e-9 {
                (a / wsum) as f32
            } else {
                0.0
            };
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_halves_frame_count() {
        let s: Vec<f32> = (0..9600).map(|i| (i as f32 * 0.01).sin()).collect();
        let out = resample(&s, 1, 96000, 48000);
        assert_eq!(out.len(), 4800);
        // A low tone survives resampling nearly unchanged.
        assert!((out[1000] - s[2000]).abs() < 0.01);
    }

    #[test]
    fn downmix_keeps_two_channels() {
        let s = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert_eq!(downmix_stereo(&s, 6).len(), 2);
    }

    #[test]
    fn quality_maps_to_bitrate() {
        assert_eq!(mp3_bitrate(None), 192);
        assert_eq!(mp3_bitrate(Some(100)), 320);
        assert_eq!(mp3_bitrate(Some(10)), 96);
    }
}
