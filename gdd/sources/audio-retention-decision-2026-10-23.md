# Audio retention — 2026-10-23

Direct user answer, copied verbatim:

> Slet lyden efter vellykket transskription, men behold den midlertidigt ved fejl, så man kan prøve igen. Vi behøver ikke lyd i historikken.

Accepted: delete audio after successful speech recognition. Temporarily retain
failed-transcription audio for retry. Do not save/play audio in text history.
Engineering clarification: a later cleanup or insertion failure does not justify
retaining/recreating successfully transcribed audio; retry those stages from text.
Delete recovery audio when a retry succeeds. The maximum failure-retention period
and exact expiry/crash-cleanup mechanics remain to be specified. Existing text
history retention is a separate policy.
