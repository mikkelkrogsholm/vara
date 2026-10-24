# Failure-audio expiry — 2026-10-23

Direct user answer, copied verbatim:

> Ja, 24 timer er fint. Slet tidligere, hvis genprøvningen lykkes, eller brugeren selv sletter optagelsen.

Accepted: failed-transcription audio is retained for at most 24 hours, and deleted
earlier after a successful retry or explicit user deletion. Engineering detail:
assign a fixed expiry on first failure retention; retry attempts do not refresh it.
Expired audio must not be offered or used for retry. Remove expired/orphaned audio
on startup/resume and during normal operation; physical cleanup cannot execute
while the application/device is not running. Report deletion failures honestly.
No audio history, audio download or background service is introduced.
