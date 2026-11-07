# Pinned Whisper native repair

`whisper-rs-sys/` is the published **0.15.0** crate, with two narrowly scoped
patches described below. Cargo uses it through
the workspace `[patch.crates-io]`; whisper-rs remains 0.16.0. No registry source or
machine-global configuration is modified.

The official crate archive is
[whisper-rs-sys-0.15.0.crate](https://static.crates.io/crates/whisper-rs-sys/whisper-rs-sys-0.15.0.crate),
SHA256 `6986c0fe081241d391f09b9a071fbcbb59720c3563628c3c829057cf69f2a56f`.
`whisper-rs-sys-provenance.json` records the original archive/native-source hashes,
patch hash, upstream fix, and SHA256 of every resulting file. The Cargo registry's
local `.cargo-ok` marker is excluded; it is not part of the published archive.

The patch adapts only the SOT-logits fix from upstream commit
[`65dacc8082ac4c7c3c48e998bbcb9dbbc6751148`](https://github.com/ggml-org/whisper.cpp/commit/65dacc8082ac4c7c3c48e998bbcb9dbbc6751148)
to the crate's native source line numbers. Prompt decoding now requests the SOT
logits row and computes `no_speech_prob` from that row. The other CLI/VAD changes
in upstream PR 4068 are deliberately excluded. Model artifacts, inference backend,
native default thresholds, app filtering and cancellation remain unchanged.

`whisper-build-sync.patch` repairs incremental build integrity: Cargo watches the
native source directory, and the build script refreshes changed source files in
its existing OUT_DIR copy before CMake runs. Unchanged files keep their timestamps
and CMake compiler intermediates remain available. The upstream script watched
only wrapper.h and copied sources only when its destination did not exist, which
could silently reuse native objects from before a local patch.

To reproduce in a fresh temporary directory: verify the archive hash, extract it,
then apply, in order, `whisper-sot-logits.patch` and `whisper-build-sync.patch` with `patch -p1` from the extracted crate directory.
Compare resulting files against the provenance manifest. The normal
offline `whisper_vendor` test verifies those hashes without any network or model.

Rust bindings retain the package's declared **Unlicense** license. Native
whisper.cpp and ggml retain their bundled license notices, including
[`whisper.cpp/LICENSE`](whisper-rs-sys/whisper.cpp/LICENSE). All vendored source
copyright/license headers are preserved. Remove this override only when a pinned
published native dependency contains the same fix and real no-speech regression
acceptance passes.
