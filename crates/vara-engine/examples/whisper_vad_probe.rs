//! Opt-in actual neural VAD diagnostic. No inference downloads or product changes.
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperVadContext,
    WhisperVadContextParams, WhisperVadParams,
};
fn read(name: &str) -> Vec<f32> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../Samples/acceptance/audio")
        .join(name);
    let mut r = hound::WavReader::open(path).unwrap();
    assert_eq!(r.spec().sample_rate, 16000);
    r.samples::<i16>()
        .map(|s| s.unwrap() as f32 / 32768.)
        .collect()
}
fn main() {
    whisper_rs::install_logging_hooks();
    let normalize = std::env::var("VARA_PROBE_VAD_NORMALIZE").as_deref() == Ok("1");
    let padding: i32 = std::env::var("VARA_PROBE_VAD_PAD_MS")
        .unwrap_or_else(|_| "200".into())
        .parse()
        .unwrap();
    let model = std::env::var("VARA_TEST_WHISPER_MODEL").unwrap();
    let vad_model = std::env::var("VARA_TEST_WHISPER_VAD").unwrap();
    let mut cp = WhisperContextParameters::default();
    cp.use_gpu(false);
    let context = WhisperContext::new_with_params(&model, cp).unwrap();
    let mut vp = WhisperVadContextParams::default();
    vp.set_use_gpu(false);
    vp.set_n_threads(8);
    let mut vad = WhisperVadContext::new(&vad_model, vp).unwrap();
    let speech = read("da-000.wav");
    let speech1 = read("da-001.wav");
    let mixed = |scale: f32| {
        std::iter::repeat_n(0., 33 * 16000)
            .chain(speech.iter().map(|s| (s * scale * 32768.).round() / 32768.))
            .chain(std::iter::repeat_n(0., 33 * 16000))
            .collect::<Vec<_>>()
    };
    let multiple = std::iter::repeat_n(0., 33 * 16000)
        .chain(speech.iter().copied())
        .chain(std::iter::repeat_n(0., 33 * 16000))
        .chain(speech1)
        .chain(std::iter::repeat_n(0., 33 * 16000))
        .collect::<Vec<_>>();
    let extra = read("da-002.wav");
    let cases = vec![
        ("da-002", extra.clone()),
        (
            "quiet-da-002",
            extra
                .iter()
                .map(|s| (s * 0.02 * 32768.).round() / 32768.)
                .collect(),
        ),
        ("mixed", mixed(1.)),
        ("short", speech.clone()),
        (
            "quiet-short",
            speech
                .iter()
                .map(|s| (s * 0.02 * 32768.).round() / 32768.)
                .collect(),
        ),
        ("quiet-mixed", mixed(0.02)),
        ("multiple", multiple),
        ("silence", read("silence.wav")),
        ("noise", read("noise.wav")),
    ];
    for (name, samples) in cases {
        if std::env::var("VARA_PROBE_CASE").is_ok_and(|filter| filter != name) {
            continue;
        }
        for language in [Some("da"), None] {
            let begin = std::time::Instant::now();
            let peak = samples.iter().map(|x| x.abs()).fold(0f32, f32::max);
            let gain = if normalize && peak > 0. {
                1. / peak
            } else {
                1.
            };
            let detector_samples: Vec<_> = samples.iter().map(|x| x * gain).collect();
            vad.detect_speech(&detector_samples).unwrap();
            let probs = vad.probabilities();
            let stats = serde_json::json!({"frames":probs.len(),"max":probs.iter().copied().fold(0f32,f32::max),"mean":probs.iter().sum::<f32>()/probs.len() as f32});
            let mut settings = WhisperVadParams::default();
            settings.set_min_speech_duration(100);
            settings.set_min_silence_duration(1000);
            settings.set_speech_pad(padding);
            let spans: Vec<_> = vad.segments_from_probabilities(settings).unwrap().collect();
            let mut out = Vec::new();
            for span in &spans {
                for timed in [false, true] {
                    // Native VAD time is centiseconds, as documented by whisper.h and implementation.
                    let start = (span.start as f64 * 160.).round() as usize;
                    let end = (span.end as f64 * 160.).round() as usize;
                    assert!(
                        start < end && end <= samples.len(),
                        "VAD region bounds {start} {end} {}",
                        samples.len()
                    );
                    let mut state = context.create_state().unwrap();
                    let mut p = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                    p.set_n_threads(8);
                    p.set_language(language);
                    p.set_translate(false);
                    p.set_no_context(true);
                    p.set_no_timestamps(!timed);
                    p.set_print_progress(false);
                    p.set_print_realtime(false);
                    p.set_print_timestamps(false);
                    p.set_print_special(false);
                    state.full(p, &samples[start..end]).unwrap();
                    for i in 0..state.full_n_segments() {
                        let s = state.get_segment(i).unwrap();
                        out.push(serde_json::json!({"timed":timed,"text":s.to_str().unwrap(),"p_no_speech":s.no_speech_probability(),"start":start as f64/16000.+s.start_timestamp() as f64/100.,"end":start as f64/16000.+s.end_timestamp() as f64/100.}));
                    }
                }
            }
            println!(
                "{}",
                serde_json::json!({"normalize_vad":normalize,"vad_gain":gain,"input_peak":peak,"padding_ms":padding,"case":name,"language":language,"duration":samples.len() as f64/16000.,"vad_stats":stats,"vad_spans":spans.iter().map(|s|vec![s.start,s.end]).collect::<Vec<_>>(),"segments":out,"elapsed_s":begin.elapsed().as_secs_f64()})
            );
        }
    }
}
