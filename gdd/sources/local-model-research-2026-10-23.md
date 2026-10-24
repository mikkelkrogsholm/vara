# Local speech model candidates — checked 2026-10-23

This is primary-source desk research, not a performance benchmark or an engine
selection. Websites and library versions may change. User spelling
“paracheet-redux” is preserved in the input; the matching researched model name is
Parakeet Redux.

| Candidate | What the source establishes | What remains unverified for Vara |
| --- | --- | --- |
| Whisper family | whisper.cpp offers C/C++ inference, quantization, CPU/GPU paths and platform support. Rust can call a binding rather than reproduce inference. | Variant, Danish accuracy, end-to-end latency, binding and packaged acceleration on each target |
| Parakeet TDT 0.6B v3 | NVIDIA lists Danish and English among 25 supported languages. parakeet-rs documents multilingual TDT integration through ONNX Runtime. | Our audio pipeline, selected runtime/export, Danish mixed-language quality and OS packaging |
| Parakeet Redux | Moondream describes a ternary compressed version of Parakeet v3. Its model card reports 178 MB packed weights and explicitly attributes its speed results to Photon. MLX Audio separately documents an MLX route. | Efficient Rust-accessible inference and model-format/kernel compatibility on all three OSes; no assumption that ordinary Parakeet ONNX integration runs Redux efficiently |

Primary sources:

- https://github.com/ggml-org/whisper.cpp
- https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3
- https://github.com/altunenes/parakeet-rs
- https://huggingface.co/moondream/parakeet-redux
- https://github.com/Blaizzy/mlx-audio/blob/main/docs/models/stt/parakeet.md

Compare the same permitted short Danish/English utterances, names, numbers,
negation, mixed-language vocabulary and quiet/noisy conditions. Record capture
startup, stop-to-raw-text, optional cleanup and final delivery separately, plus
cold/warm loading, RAM, download size and actual runtime/hardware. A speech-model
benchmark alone does not establish the final experience. Third-party reported
speed is not a Vara result. CPU-only capability and accelerated paths must be
measured on the supported environment envelope, following the agreed Mac-first
and final Windows/Linux test sequence.
