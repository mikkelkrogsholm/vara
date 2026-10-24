//! Local model comparison only. Does not change the app's installed models/defaults.
use std::{path::Path, time::Instant};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let model = args.get(1).ok_or("usage: whisper_bench MODEL WAV [RUNS]")?;
    let audio = args.get(2).ok_or("provide a WAV")?;
    let repeats: usize = args.get(3).map(|s| s.parse()).transpose()?.unwrap_or(6);
    if !(2..=20).contains(&repeats) {
        return Err("use 2–20 runs".into());
    }
    let mut reader = hound::WavReader::open(audio)?;
    let spec = reader.spec();
    if spec.sample_format != hound::SampleFormat::Int
        || spec.bits_per_sample != 16
        || spec.channels != 1
    {
        return Err("expected mono PCM16 WAV".into());
    }
    let source: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.map(|v| v as f32 / 32768.))
        .collect::<Result<_, _>>()?;
    let duration = source.len() as f64 / spec.sample_rate as f64;
    let samples = if spec.sample_rate == 16000 {
        source
    } else {
        (0..(duration * 16000.) as usize)
            .map(|i| {
                let position = i as f64 * spec.sample_rate as f64 / 16000.;
                let first = (position.floor() as usize).min(source.len() - 1);
                let next = (first + 1).min(source.len() - 1);
                source[first]
                    + (source[next] - source[first]) * (position - position.floor()) as f32
            })
            .collect()
    };
    let gpu = std::env::var("VARA_PROBE_GPU").as_deref() == Ok("1");
    // Keep native backend diagnostics for GPU probes so fallback can be detected.
    if !gpu {
        whisper_rs::install_logging_hooks();
    }
    let mut params = WhisperContextParameters::default();
    params.use_gpu(gpu);
    let begin = Instant::now();
    let context =
        WhisperContext::new_with_params(Path::new(model).to_str().ok_or("invalid path")?, params)?;
    let load_ms = begin.elapsed().as_secs_f64() * 1000.;
    for run in 0..repeats {
        let begin = Instant::now();
        let mut state = context.create_state()?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(8);
        params.set_language(Some("da"));
        params.set_translate(false);
        params.set_no_context(true);
        params.set_no_timestamps(true);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_print_special(false);
        state.full(params, &samples)?;
        let mut text = String::new();
        for i in 0..state.full_n_segments() {
            if let Some(segment) = state.get_segment(i) {
                if segment.no_speech_probability() < 0.85 {
                    text.push_str(segment.to_str()?);
                }
            }
        }
        println!(
            "{}",
            serde_json::json!({"model":model,"device_requested":if gpu {"metal"} else {"cpu"},"threads":8,"audio_s":duration,"load_ms":load_ms,"run":run,"elapsed_ms":begin.elapsed().as_secs_f64()*1000.,"text":text.trim()})
        );
    }
    Ok(())
}
