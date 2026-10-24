# Delivery and per-app history — 2026-10-23

Direct user answer, copied verbatim:

> Ja, det lyder godt. Automatisk indsættelse når fokus er uændret, ellers klar til manuel indsættelse. Og bevar det, der allerede ligger i udklipsholderen.
>
> og gem resultaterne i vara - grupperet under den app den blev kaldt i. lige som i vara nu

Accepted: automatically insert only when the original text target still has focus;
otherwise retain a result ready for deliberate manual insertion. Preserve existing
clipboard contents during automatic delivery. Save completed results in Vara,
grouped under the app where dictation was invoked, even if focus later changes.
The old app is a reference for this grouping behavior, not a mandatory UI layout.

Engineering interpretation: uncertain target identity follows the manual route.
Clipboard preservation includes applicable formats, and restoring temporary paste
data must not overwrite something the user copied concurrently. A deliberate Copy
command may intentionally replace clipboard contents. Cancelled/abandoned sessions
must not appear as completed history. Exact retention duration, raw-versus-final
text persistence and recovery-audio policy remain for the data lifecycle decision.
