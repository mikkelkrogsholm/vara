//! Stream media to a private mono 16 kHz WAV. Source files are never modified.
use crate::{Cancellation, Result};
use std::{fs::File, path::Path};
use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, errors::Error, io::MediaSourceStream, probe::Hint,
};
const MAX_SAMPLES: u64 = 16_000 * 60 * 60 * 8;
pub fn normalize(source: &Path, destination: &Path, cancel: &Cancellation) -> Result<f64> {
    cancel.check()?;
    if source == destination || same_file::is_same_file(source, destination).unwrap_or(false) {
        return Err("Originalen må ikke overskrives.".into());
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
    let file = File::open(source).map_err(|_| "Kildefilen kunne ikke åbnes. Vælg den igen.")?;
    if !file
        .metadata()
        .map_err(|_| "Kildefilen kunne ikke læses.")?
        .is_file()
    {
        return Err("Vælg en lyd- eller videofil.".into());
    }
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = source.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }
    let mut format = symphonia::default::get_probe()
        .format(&hint, stream, &Default::default(), &Default::default())
        .map_err(|_| {
            *compatible_fallback = true;
            "Formatet kunne ikke læses. Prøv WAV, MP3, FLAC, M4A eller video med AAC-lyd."
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
            "Filen indeholder ingen understøttet lyd. Prøv at eksportere lyden som WAV eller MP3."
        })?;
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut wav = hound::WavWriter::create(destination, spec)
        .map_err(|_| "Arbejdslyden kunne ikke gemmes. Kontroller ledig plads.")?;
    let mut source_frames = 0u64;
    let mut output_samples = 0u64;
    let mut rate = 0u32;
    let mut previous = 0f32;
    loop {
        cancel.check()?;
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(_) => {
                return Err(
                    "Mediefilen er beskadiget eller skifter lydformat. Originalen er bevaret."
                        .into(),
                );
            }
        };
        if packet.track_id() != id {
            continue;
        }
        let audio = decoder.decode(&packet).map_err(|_| {
            *compatible_fallback = true;
            "Lydsporet kunne ikke afkodes. Originalen er bevaret."
        })?;
        let cfg = *audio.spec();
        let channels = cfg.channels.count();
        if cfg.rate == 0 || channels == 0 || (rate != 0 && rate != cfg.rate) {
            return Err("Lydsporets format understøttes ikke.".into());
        }
        rate = cfg.rate;
        let mut samples = SampleBuffer::<f32>::new(audio.capacity() as u64, cfg);
        samples.copy_interleaved_ref(audio);
        for frame in samples.samples().chunks(channels) {
            let sample = frame.iter().sum::<f32>() / channels as f32;
            if !sample.is_finite() {
                return Err("Lydsporet indeholder ugyldige samples.".into());
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
                    .map_err(|_| "Arbejdslyden kunne ikke gemmes. Kontroller ledig plads.")?;
                output_samples += 1;
                if output_samples > MAX_SAMPLES {
                    return Err(
                        "Filen overstiger grænsen på 8 timers lyd. Del den i mindre filer.".into(),
                    );
                }
            }
            source_frames += 1;
            previous = sample;
        }
    }
    cancel.check()?;
    if output_samples < 1600 {
        return Err("Filen indeholder ingen eller for kort lyd.".into());
    }
    wav.finalize()
        .map_err(|_| "Arbejdslyden kunne ikke færdiggøres.")?;
    Ok(output_samples as f64 / 16000.)
}

/// Bounded chunks for runtimes/providers with per-request limits. Chunk files are temporary.
pub fn chunks(
    path: &Path,
    cancel: &Cancellation,
    mut consume: impl FnMut(&Path, f64, f64) -> Result<()>,
) -> Result<()> {
    let mut reader =
        hound::WavReader::open(path).map_err(|_| "Arbejdslyden mangler. Genprøv filen.")?;
    let mut samples = reader.samples::<i16>();
    let mut offset = 0.;
    loop {
        cancel.check()?;
        let buffer = samples
            .by_ref()
            .take(300 * 16000)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| "Arbejdslyden er beskadiget.")?;
        if buffer.is_empty() {
            break;
        }
        let chunk =
            tempfile::NamedTempFile::new().map_err(|_| "Arbejdsfilen kunne ikke oprettes.")?;
        let mut writer = hound::WavWriter::create(
            chunk.path(),
            hound::WavSpec {
                channels: 1,
                sample_rate: 16000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .map_err(|_| "Arbejdsfilen kunne ikke gemmes.")?;
        let duration = buffer.len() as f64 / 16000.;
        for v in buffer {
            writer
                .write_sample(v)
                .map_err(|_| "Arbejdsfilen kunne ikke gemmes.")?;
        }
        writer
            .finalize()
            .map_err(|_| "Arbejdsfilen kunne ikke færdiggøres.")?;
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
            return Err("Medieafkodning nåede tidsgrænsen. Prøv en mindre fil.".into());
        }
        let status = child.lock().try_wait();
        match status {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err("Filen indeholder ingen læsbar lyd, eller er beskadiget. Prøv at eksportere lyden som WAV eller MP3.".into());
                }
                break;
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(40)),
            Err(_) => {
                let mut child = child.lock();
                let _ = child.kill();
                let _ = child.wait();
                return Err("Medieafkodningen blev afbrudt.".into());
            }
        }
    }
    let reader = hound::WavReader::open(destination).map_err(|_| "Den afkodede lyd er ugyldig.")?;
    let duration = reader.duration() as f64 / 16000.;
    if reader.spec().sample_rate != 16000 || reader.spec().channels != 1 || duration < 0.1 {
        return Err("Filen indeholder ingen brugbar lyd.".into());
    }
    if duration > 28800. {
        return Err("Filen overstiger grænsen på 8 timers lyd. Del den i mindre filer.".into());
    }
    cancel.check()?;
    Ok(duration)
}

/// Fixed-size actual amplitude envelope for the seek control; no fabricated waveform.
pub fn waveform(path: &Path, cancel: &Cancellation) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path).map_err(|_| "Lydkopien kunne ikke læses.")?;
    let total = reader.duration() as usize;
    let width = 128;
    let mut peaks = vec![0f32; width];
    for (i, sample) in reader.samples::<i16>().enumerate() {
        if i % 16000 == 0 {
            cancel.check()?;
        }
        let sample = sample.map_err(|_| "Lydkopien er beskadiget.")?;
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
    let reader = hound::WavReader::open(path).map_err(|_| "Lydkopien kunne ikke læses.")?;
    let spec = reader.spec();
    if spec.sample_rate != 16000
        || spec.channels != 1
        || spec.bits_per_sample != 16
        || spec.sample_format != hound::SampleFormat::Int
    {
        return Err("Lydkopien har et ugyldigt format.".into());
    }
    let duration = reader.duration() as f64 / 16000.;
    if !(0.1..=28800.).contains(&duration) {
        return Err("Lydkopiens varighed understøttes ikke.".into());
    }
    Ok(duration)
}
