# Model-dependent responsiveness — 2026-10-23

Direct user answer, copied verbatim:

> afhænger jo af modellen. men det skal gå stærkt. jeg tror der er mange små modeller, whisper, paracheet-redux, osv som vi kan gøre i rust

Accepted direction: prioritize fast perceived response, acknowledging that actual
latency depends on the model. Investigate compact local speech models that can be
integrated into the Rust application rather than selecting a universal time limit
up front. The mentioned names are candidates, not a selected engine or a demand
to reimplement model inference in pure Rust. No measurements have been executed.

Hardware dependence, warm/cold measurements and end-to-end timing are engineering
interpretations. Exact budgets, evaluation samples and a winning model remain open.
