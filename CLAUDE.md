# Vara development

Work on the Rust/Tauri workspace described in README.md. The retired Swift app
is available in Git history and is not an active implementation.

Follow gdd/PRODUCT-SPEC.md, gdd/DESIGN.md and gdd/RUST-BINDINGS.md. Keep shared
engine/domain APIs independent of Tauri and native desktop APIs. Edit canonical
UI tokens/components/assets under gdd/design rather than desktop/ui/design.

Use direct searches for local edits. Graphify is optional when an existing graph
helps answer a cross-module question; graph regeneration is not a prerequisite
for ordinary changes. Keep generated graphs and build output out of Git.

Use optimized builds for local inference and verification, and remove unneeded
build intermediates after delivery. Never expose credentials or commit .env files.
