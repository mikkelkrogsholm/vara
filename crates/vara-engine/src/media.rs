//! Stream media to a private mono 16 kHz WAV. Source files are never modified.
use crate::{Cancellation, Result};
use std::{fs::File, path::Path};
use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, errors::Error, io::MediaSourceStream, probe::Hint,
};
fn symphonia_cause(error: &Error) -> vara_core::UserMessage {
    match error {
        Error::IoError(e) => vara_core::UserMessage::io_cause(e),
        Error::DecodeError(_) => vara_core::UserMessage::new("media.cause_format"),
        Error::Unsupported(_) => vara_core::UserMessage::new("media.cause_unsupported"),
        Error::LimitError(_) => vara_core::UserMessage::new("media.cause_limit"),
        Error::SeekError(_) => vara_core::UserMessage::new("media.cause_seek"),
        Error::ResetRequired => vara_core::UserMessage::new("media.cause_reset"),
    }
}
fn hound_cause(error: &hound::Error) -> vara_core::UserMessage {
    match error {
        hound::Error::IoError(e) => vara_core::UserMessage::io_cause(e),
        hound::Error::Unsupported => vara_core::UserMessage::new("media.cause_unsupported"),
        _ => vara_core::UserMessage::new("media.cause_format"),
    }
}
const MAX_SAMPLES: u64 = 16_000 * 60 * 60 * 8;
pub fn normalize(source: &Path, destination: &Path, cancel: &Cancellation) -> Result<f64> {
    cancel.check()?;
    if source == destination || same_file::is_same_file(source, destination).unwrap_or(false) {
        return Err(vara_core::UserMessage::new("media.overwrite").wire());
    }
    let mut compatible_fallback = false;
    let result = decode(source, destination, cancel, &mut compatible_fallback).or_else(|native| {
        if !compatible_fallback {
            return Err(native);
        }
        cancel.check()?;
        // Optional local compatibility adapter. Common formats decode in Rust without setup.
        if let Some(ffmpeg) = ffmpeg_path() {
            decode_ffmpeg(&ffmpeg, source, destination, cancel)
        } else {
            Err(native)
        }
    });
    if result.is_err() {
        let _ = std::fs::remove_file(destination);
    }
    result
}
fn decode(
    source: &Path,
    destination: &Path,
    cancel: &Cancellation,
    compatible_fallback: &mut bool,
) -> Result<f64> {
    let file = File::open(source).map_err(|e| {
        vara_core::UserMessage::new("media.source_open")
            .caused_by(vara_core::UserMessage::io_cause(&e))
            .wire()
    })?;
    if !file
        .metadata()
        .map_err(|e| {
            vara_core::UserMessage::new("media.source_read")
                .caused_by(vara_core::UserMessage::io_cause(&e))
                .wire()
        })?
        .is_file()
    {
        return Err(vara_core::UserMessage::new("media.choose_file").wire());
    }
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = source.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }
    let mut format = symphonia::default::get_probe()
        .format(&hint, stream, &Default::default(), &Default::default())
        .map_err(|e| {
            *compatible_fallback = true;
            vara_core::UserMessage::new("media.format")
                .caused_by(symphonia_cause(&e))
                .wire()
        })?
        .format;
    let (id, mut decoder) = format
        .tracks()
        .iter()
        .find_map(|t| {
            symphonia::default::get_codecs()
                .make(&t.codec_params, &DecoderOptions::default())
                .ok()
                .map(|d| (t.id, d))
        })
        .ok_or_else(|| {
            *compatible_fallback = true;
            vara_core::UserMessage::new("media.no_supported_audio").wire()
        })?;
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut wav = hound::WavWriter::create(destination, spec).map_err(|e| {
        vara_core::UserMessage::new("media.work_save")
            .caused_by(hound_cause(&e))
            .wire()
    })?;
    let mut source_frames = 0u64;
    let mut output_samples = 0u64;
    let mut rate = 0u32;
    let mut previous = 0f32;
    loop {
        cancel.check()?;
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                return Err(vara_core::UserMessage::new("media.corrupt_format")
                    .caused_by(symphonia_cause(&e))
                    .wire());
            }
        };
        if packet.track_id() != id {
            continue;
        }
        let audio = decoder.decode(&packet).map_err(|e| {
            *compatible_fallback = true;
            vara_core::UserMessage::new("media.decode")
                .caused_by(symphonia_cause(&e))
                .wire()
        })?;
        let cfg = *audio.spec();
        let channels = cfg.channels.count();
        if cfg.rate == 0 || channels == 0 || (rate != 0 && rate != cfg.rate) {
            return Err(vara_core::UserMessage::new("media.track_format").wire());
        }
        rate = cfg.rate;
        let mut samples = SampleBuffer::<f32>::new(audio.capacity() as u64, cfg);
        samples.copy_interleaved_ref(audio);
        for frame in samples.samples().chunks(channels) {
            let sample = frame.iter().sum::<f32>() / channels as f32;
            if !sample.is_finite() {
                return Err(vara_core::UserMessage::new("media.invalid_samples").wire());
            }
            // A continuous clock across packets prevents packet-boundary drift.
            while (output_samples as f64 * rate as f64 / 16000.) <= source_frames as f64 {
                let at = output_samples as f64 * rate as f64 / 16000.;
                let alpha = (at - (source_frames as f64 - 1.)).clamp(0., 1.) as f32;
                let v = if source_frames == 0 {
                    sample
                } else {
                    previous + (sample - previous) * alpha
                };
                wav.write_sample((v.clamp(-1., 1.) * 32767.).round() as i16)
                    .map_err(|e| {
                        vara_core::UserMessage::new("media.work_save")
                            .caused_by(hound_cause(&e))
                            .wire()
                    })?;
                output_samples += 1;
                if output_samples > MAX_SAMPLES {
                    return Err(vara_core::UserMessage::new("media.too_long").wire());
                }
            }
            source_frames += 1;
            previous = sample;
        }
    }
    cancel.check()?;
    if output_samples < 1600 {
        return Err(vara_core::UserMessage::new("media.too_short").wire());
    }
    wav.finalize().map_err(|e| {
        vara_core::UserMessage::new("media.work_finish")
            .caused_by(hound_cause(&e))
            .wire()
    })?;
    Ok(output_samples as f64 / 16000.)
}

/// Bounded chunks for runtimes/providers with per-request limits. Chunk files are temporary.
pub fn chunks(
    path: &Path,
    cancel: &Cancellation,
    mut consume: impl FnMut(&Path, f64, f64) -> Result<()>,
) -> Result<()> {
    let mut reader = hound::WavReader::open(path).map_err(|e| {
        vara_core::UserMessage::new("media.work_missing")
            .caused_by(hound_cause(&e))
            .wire()
    })?;
    let mut samples = reader.samples::<i16>();
    let mut offset = 0.;
    loop {
        cancel.check()?;
        let buffer = samples
            .by_ref()
            .take(300 * 16000)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                vara_core::UserMessage::new("media.work_corrupt")
                    .caused_by(hound_cause(&e))
                    .wire()
            })?;
        if buffer.is_empty() {
            break;
        }
        let chunk = tempfile::NamedTempFile::new().map_err(|e| {
            vara_core::UserMessage::new("media.temp_create")
                .caused_by(vara_core::UserMessage::io_cause(&e))
                .wire()
        })?;
        let mut writer = hound::WavWriter::create(
            chunk.path(),
            hound::WavSpec {
                channels: 1,
                sample_rate: 16000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .map_err(|e| {
            vara_core::UserMessage::new("media.temp_save")
                .caused_by(hound_cause(&e))
                .wire()
        })?;
        let duration = buffer.len() as f64 / 16000.;
        for v in buffer {
            writer.write_sample(v).map_err(|e| {
                vara_core::UserMessage::new("media.temp_save")
                    .caused_by(hound_cause(&e))
                    .wire()
            })?;
        }
        writer.finalize().map_err(|e| {
            vara_core::UserMessage::new("media.temp_finish")
                .caused_by(hound_cause(&e))
                .wire()
        })?;
        consume(chunk.path(), offset, duration)?;
        offset += duration;
    }
    Ok(())
}

fn ffmpeg_path() -> Option<std::path::PathBuf> {
    let name = if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    let mut dirs =
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect::<Vec<_>>();
    #[cfg(target_os = "macos")]
    {
        dirs.push("/opt/homebrew/bin".into());
        dirs.push("/usr/local/bin".into());
    }
    dirs.into_iter().map(|p| p.join(name)).find(|p| p.is_file())
}
fn decode_ffmpeg(
    binary: &Path,
    source: &Path,
    destination: &Path,
    cancel: &Cancellation,
) -> Result<f64> {
    use std::process::{Command, Stdio};
    let child = cancel.spawn_child(|| {
        Command::new(binary)
            .args([
                "-nostdin",
                "-v",
                "error",
                "-xerror",
                "-y",
                "-protocol_whitelist",
                "file,pipe",
                "-i",
            ])
            .arg(source)
            .args([
                "-map",
                "0:a:0",
                "-vn",
                "-ac",
                "1",
                "-ar",
                "16000",
                "-t",
                "28800.1",
                "-c:a",
                "pcm_s16le",
                "-f",
                "wav",
            ])
            .arg(destination)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    })?;
    let start = std::time::Instant::now();
    loop {
        if cancel.check().is_err() || start.elapsed().as_secs() > 900 {
            let mut child = child.lock();
            let _ = child.kill();
            let _ = child.wait();
            cancel.check()?;
            return Err(vara_core::UserMessage::new("media.timeout").wire());
        }
        let status = child.lock().try_wait();
        match status {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err(vara_core::UserMessage::new("media.unreadable_audio")
                        .caused_by(crate::process_cause(&status))
                        .wire());
                }
                break;
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(40)),
            Err(error) => {
                let mut child = child.lock();
                let _ = child.kill();
                let _ = child.wait();
                return Err(vara_core::UserMessage::new("media.interrupted")
                    .caused_by(vara_core::UserMessage::io_cause(&error))
                    .wire());
            }
        }
    }
    let reader = hound::WavReader::open(destination).map_err(|e| {
        vara_core::UserMessage::new("media.decoded_invalid")
            .caused_by(hound_cause(&e))
            .wire()
    })?;
    let duration = reader.duration() as f64 / 16000.;
    if reader.spec().sample_rate != 16000 || reader.spec().channels != 1 || duration < 0.1 {
        return Err(vara_core::UserMessage::new("media.no_usable_audio").wire());
    }
    if duration > 28800. {
        return Err(vara_core::UserMessage::new("media.too_long").wire());
    }
    cancel.check()?;
    Ok(duration)
}

/// Fixed-size actual amplitude envelope for the seek control; no fabricated waveform.
pub fn waveform(path: &Path, cancel: &Cancellation) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path).map_err(|e| {
        vara_core::UserMessage::new("media.copy_read")
            .caused_by(hound_cause(&e))
            .wire()
    })?;
    let total = reader.duration() as usize;
    let width = 128;
    let mut peaks = vec![0f32; width];
    for (i, sample) in reader.samples::<i16>().enumerate() {
        if i % 16000 == 0 {
            cancel.check()?;
        }
        let sample = sample.map_err(|e| {
            vara_core::UserMessage::new("media.copy_corrupt")
                .caused_by(hound_cause(&e))
                .wire()
        })?;
        let bin = (i * width / total.max(1)).min(width - 1);
        peaks[bin] = peaks[bin].max((sample as f32 / 32768.).abs());
    }
    Ok(peaks)
}

#[cfg(all(test, unix))]
mod subprocess_tests {
    use super::*;
    #[test]
    fn cancellation_kills_registered_media_children_before_returning() {
        let cancel = Cancellation::default();
        let child = cancel
            .spawn_child(|| std::process::Command::new("/bin/sleep").arg("30").spawn())
            .unwrap();
        assert!(cancel.cancel());
        assert!(child.lock().try_wait().unwrap().is_some());
        assert!(
            cancel
                .spawn_child(|| std::process::Command::new("/bin/sleep").arg("30").spawn())
                .is_err()
        );
    }
}

/// An already-published private playback copy can be reused after a failed recognition.
pub fn normalized_duration(path: &Path) -> Result<f64> {
    let reader = hound::WavReader::open(path).map_err(|e| {
        vara_core::UserMessage::new("media.copy_read")
            .caused_by(hound_cause(&e))
            .wire()
    })?;
    let spec = reader.spec();
    if spec.sample_rate != 16000
        || spec.channels != 1
        || spec.bits_per_sample != 16
        || spec.sample_format != hound::SampleFormat::Int
    {
        return Err(vara_core::UserMessage::new("media.copy_format").wire());
    }
    let duration = reader.duration() as f64 / 16000.;
    if !(0.1..=28800.).contains(&duration) {
        return Err(vara_core::UserMessage::new("media.copy_duration").wire());
    }
    Ok(duration)
}
