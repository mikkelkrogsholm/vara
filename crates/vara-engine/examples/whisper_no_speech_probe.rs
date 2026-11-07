//! Opt-in diagnostic: no model download and no product parameter changes.
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
fn main() {
    let model = std::env::var("VARA_TEST_WHISPER_MODEL").expect("VARA_TEST_WHISPER_MODEL");
    let mut context_params = WhisperContextParameters::default();
    context_params.use_gpu(false);
    let context = WhisperContext::new_with_params(&model, context_params).unwrap();
    for name in [
        "silence.wav",
        "noise.wav",
        "da-000.wav",
        "da-001.wav",
        "da-002.wav",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../Samples/acceptance/audio")
            .join(name);
        let mut reader = hound::WavReader::open(path).unwrap();
        assert_eq!(reader.spec().sample_rate, 16000);
        let original: Vec<f32> = reader
            .samples::<i16>()
            .map(|x| x.unwrap() as f32 / 32768.)
            .collect();
        for scale in if name.starts_with("da-") {
            vec![1., 0.02]
        } else {
            vec![1.]
        } {
            let samples: Vec<f32> = original.iter().map(|x| x * scale).collect();
            let start = std::time::Instant::now();
            let mut probe = context.create_state().unwrap();
            probe.pcm_to_mel(&samples, 8).unwrap();
            probe.encode(0, 8).unwrap();
            probe.decode(&[context.token_sot()], 0, 8).unwrap();
            let logits = probe.get_logits().unwrap();
            let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let denominator: f32 = logits.iter().map(|x| (x - max).exp()).sum();
            let sot_probability = (logits[context.token_nosp() as usize] - max).exp() / denominator;
            let sot_seconds = start.elapsed().as_secs_f64();
            for timed in [false, true] {
                let mut state = context.create_state().unwrap();
                let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                params.set_n_threads(8);
                params.set_language(Some("da"));
                params.set_translate(false);
                params.set_no_context(true);
                params.set_no_timestamps(!timed);
                params.set_print_progress(false);
                params.set_print_realtime(false);
                params.set_print_timestamps(false);
                params.set_print_special(false);
                state.full(params, &samples).unwrap();
                let segments: Vec<_> = (0..state.full_n_segments()).map(|i| {
                    let s = state.get_segment(i).unwrap();
                    let logprobs: Vec<_> = (0..s.n_tokens()).map(|j| s.get_token(j).unwrap().token_data().plog).collect();
                    serde_json::json!({"text":s.to_str().unwrap(), "no_speech_probability":s.no_speech_probability(), "token_logprobs":logprobs, "token_avg_logprob":logprobs.iter().sum::<f32>() / logprobs.len() as f32})
                }).collect();
                println!(
                    "{}",
                    serde_json::json!({"fixture":name,"scale":scale,"timed":timed,"sot_probability":sot_probability,"sot_seconds":sot_seconds,"segments":segments})
                );
            }
        }
    }
}
