# Default mode and system prompts — 2026-10-23

Direct user answer, copied verbatim:

> Direkte transskription som standard, men kan sendes til en AI, med en systemprompt (der kan være generelt eller er knyttet til en app), jus oprydningen/justeringen passer til use-cases

Accepted: direct transcription is the default, without an extra text AI step.
Optional AI processing uses an editable system prompt that can be general or
assigned to a particular application, to fit cleanup/adjustment use cases.

Scope change: minimal app-specific prompt configuration is now included in the
first release, superseding the earlier deferral of all app-specific customization.
This does not include arbitrary app observation or the deferred Ask AI workflow.
Prompt selection uses the app captured at invocation, not later foreground focus.
Whether a configured app prompt automatically enables processing, how per-session
selection works and whether transformed output needs preview are still open.
Local processing remains the default execution policy; cloud is explicit choice.
