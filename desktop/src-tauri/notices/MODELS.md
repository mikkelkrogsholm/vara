# Local model attribution

Parakeet Redux weights are © NVIDIA and M87 Labs (Moondream), licensed under
Creative Commons Attribution 4.0 International (CC-BY-4.0).

Model: https://huggingface.co/moondream/parakeet-redux
License: https://creativecommons.org/licenses/by/4.0/
Base architecture: https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3
Revision: 2bf128600aac4b16946f7ed8372e56117fe5e23b

Vara downloads these pinned weights unchanged. The voxrt runtime is Apache-2.0;
its license and notice accompany this file. No evaluation dataset normalizer from
the runtime repository is used or shipped in Vara.

Whisper Base: OpenAI Whisper, MIT. https://github.com/openai/whisper

Whisper voice activity detection helper: Silero VAD 5.1.2, MIT.
Copyright (c) 2020-present Silero Team.
Vara's local Whisper model install also downloads the unchanged GGML helper
`ggml-silero-v5.1.2.bin` (885,098 bytes, approximately 885 KB). This is a required
companion to the Whisper weights and is downloaded separately, not bundled in
the installer.

Upstream: https://github.com/snakers4/silero-vad
License: https://github.com/snakers4/silero-vad/blob/master/LICENSE
Pinned GGML source: https://huggingface.co/ggml-org/whisper-vad/resolve/9ffd54a1e1ee413ddf265af9913beaf518d1639b/ggml-silero-v5.1.2.bin
Revision: 9ffd54a1e1ee413ddf265af9913beaf518d1639b
SHA-256: 29940d98d42b91fbd05ce489f3ecf7c72f0a42f027e4875919a28fb4c04ea2cf

Qwen3-0.6B: Qwen, Apache-2.0. https://huggingface.co/Qwen/Qwen3-0.6B
