use crate::{Cancellation, Result};
use candle_core::{Device, Tensor, quantized::gguf_file};
use candle_transformers::{generation::LogitsProcessor, models::quantized_qwen3::ModelWeights};
use parking_lot::Mutex;
use std::{fs::File, path::Path};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
#[derive(Default)]
pub struct Local {
    speech: Mutex<Option<WhisperContext>>,
    text: Mutex<Option<(ModelWeights, tokenizers::Tokenizer)>>,
}
impl Local {
    pub fn speech(
        &self,
        model: &Path,
        path: &Path,
        language: Option<&str>,
        cancel: &Cancellation,
    ) -> Result<String> {
        let (samples, _) = read_audio(path)?;
        if samples.len() < 1600 {
            return Err("Optagelsen er for kort.".into());
        }
        let mut guard = self.speech.lock();
        cancel.check()?;
        if guard.is_none() {
            let mut params = WhisperContextParameters::default();
            params.use_gpu(false);
            *guard = Some(
                WhisperContext::new_with_params(model.to_str().ok_or("Ugyldig modelsti.")?, params)
                    .map_err(|_| "Whisper kunne ikke indlæses. Hent modellen i Indstillinger.")?,
            );
        }
        let mut state = guard
            .as_ref()
            .unwrap()
            .create_state()
            .map_err(|_| "Whisper kunne ikke starte.")?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(
            std::thread::available_parallelism()
                .map(|n| n.get().min(8))
                .unwrap_or(4) as i32,
        );
        params.set_language(language);
        params.set_translate(false);
        params.set_no_context(true);
        params.set_no_timestamps(true);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_print_special(false);
        // whisper_full is synchronous; cancel's Arc outlives every callback. No ownership crosses FFI.
        unsafe extern "C" fn abort(data: *mut std::ffi::c_void) -> bool {
            unsafe {
                (*(data as *const std::sync::atomic::AtomicBool))
                    .load(std::sync::atomic::Ordering::SeqCst)
            }
        }
        unsafe {
            params.set_abort_callback(Some(abort));
            params.set_abort_callback_user_data(
                std::sync::Arc::as_ptr(&cancel.0) as *mut std::ffi::c_void
            );
        }
        state
            .full(params, &samples)
            .map_err(|_| "Transskriptionen mislykkedes. Optagelsen kan genprøves.")?;
        cancel.check()?;
        let mut text = String::new();
        for i in 0..state.full_n_segments() {
            if let Some(s) = state.get_segment(i) {
                if s.no_speech_probability() < 0.85 {
                    text.push_str(s.to_str().map_err(|_| "Teksten kunne ikke afkodes.")?);
                }
            }
        }
        Ok(text)
    }
    pub fn text(
        &self,
        dir: &Path,
        plan: &vara_core::TextPlan,
        text: &str,
        cancel: &Cancellation,
    ) -> Result<String> {
        let mut guard = self.text.lock();
        cancel.check()?;
        if guard.is_none() {
            let mut file = File::open(dir.join("Qwen3-0.6B-Q8_0.gguf"))
                .map_err(|_| "Hent først den lokale tekstmodel.")?;
            let ct =
                gguf_file::Content::read(&mut file).map_err(|_| "Tekstmodellen er beskadiget.")?;
            let weights = ModelWeights::from_gguf(ct, &mut file, &Device::Cpu)
                .map_err(|_| "Tekstmodellen kunne ikke indlæses.")?;
            let tokenizer = tokenizers::Tokenizer::from_file(dir.join("tokenizer.json"))
                .map_err(|_| "Tokenizer kunne ikke indlæses.")?;
            *guard = Some((weights, tokenizer));
        }
        let (model, tokenizer) = guard.as_mut().unwrap();
        model.clear_kv_cache();
        // Escape chat control tokens in user-controlled content, preserving the actual transcript separately.
        let escape = |s: &str| s.replace("<|", "< | ").replace("|>", " | >");
        let prompt = format!(
            "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\nTransskription til bearbejdning:\n{}\n/no_think<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n",
            escape(&plan.system_prompt),
            escape(text)
        );
        let mut tokens = tokenizer
            .encode(prompt, false)
            .map_err(|_| "Teksten kunne ikke tokeniseres.")?
            .get_ids()
            .to_vec();
        if tokens.len() > 3072 {
            return Err("Teksten er for lang til den lokale model. Originalen er bevaret.".into());
        }
        let mut sampler = LogitsProcessor::new(42, None, None);
        let mut output = Vec::new();
        let mut offset = 0;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        for _ in 0..1024 {
            cancel.check()?;
            if std::time::Instant::now() > deadline {
                return Err("Lokal AI nåede tidsgrænsen. Originalen er bevaret.".into());
            }
            let input = Tensor::new(tokens.as_slice(), &Device::Cpu)
                .and_then(|t| t.unsqueeze(0))
                .map_err(|_| "Tekstmodellen kunne ikke køre.")?;
            let logits = model
                .forward(&input, offset)
                .and_then(|t| t.squeeze(0))
                .map_err(|_| "Tekstmodellen kunne ikke køre.")?;
            offset += tokens.len();
            let token = sampler
                .sample(&logits)
                .map_err(|_| "AI-generering mislykkedes.")?;
            if Some(token) == tokenizer.token_to_id("<|im_end|>")
                || Some(token) == tokenizer.token_to_id("<|endoftext|>")
            {
                return tokenizer
                    .decode(&output, true)
                    .map_err(|_| "AI-teksten kunne ikke afkodes.".into());
            }
            output.push(token);
            tokens = vec![token];
        }
        Err("AI nåede tekstgrænsen. Originalen er bevaret.".into())
    }
}
/// Decode mono PCM and resample to Whisper's 16kHz. Cloud gets this same bounded WAV.
pub(crate) fn read_audio(path: &Path) -> Result<(Vec<f32>, f64)> {
    let mut reader = hound::WavReader::open(path).map_err(|_| "Optagelsen kunne ikke læses.")?;
    let spec = reader.spec();
    if spec.sample_format != hound::SampleFormat::Int
        || spec.bits_per_sample != 16
        || spec.channels != 1
        || spec.sample_rate == 0
    {
        return Err("Optagelsen har et ugyldigt lydformat.".into());
    }
    let source: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.map(|v| v as f32 / 32768.0))
        .collect::<std::result::Result<_, _>>()
        .map_err(|_| "Optagelsen er beskadiget.")?;
    let duration = source.len() as f64 / spec.sample_rate as f64;
    if duration > 600.0 {
        return Err("Optagelsen overstiger 10 minutter.".into());
    }
    if spec.sample_rate == 16000 {
        return Ok((source, duration));
    }
    let n = (duration * 16000.0) as usize;
    let ratio = spec.sample_rate as f64 / 16000.;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let p = i as f64 * ratio;
        let lo = p as usize;
        let hi = (lo + 1).min(source.len().saturating_sub(1));
        let f = (p - lo as f64) as f32;
        out.push(source[lo] * (1. - f) + source[hi] * f);
    }
    Ok((out, duration))
}
