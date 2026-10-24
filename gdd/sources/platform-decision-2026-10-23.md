# Platform strategy — 2026-10-23

Direct user answer, copied verbatim:

> Alle tre fra starten: macOS, Windows og Linux. Det skal være crossplatform fra begyndelsen.
> men vi er på mac og bygger det til den først. vi kan ikke teste på de andre før til sidst. men rust og god dømmekraft burde kunne få det til at virke alle steder

Accepted direction: all three desktop platforms belong to the first release.
Develop and test on macOS first, with Windows/Linux runtime tests deferred until
the final verification phase. Crossplatform responsibilities and explicit OS
contracts are required from the start. Rust is the working implementation direction;
no UI toolkit or dependency catalog is selected by this answer.

Engineering interpretation: source portability and sound boundaries support this
strategy but do not prove runtime behavior. Windows/Linux remain targeted and
unverified until their final tests. Required versions, hardware and Linux desktop
environments are still an open, separate support-envelope decision. No early
Windows/Linux testing gate is imposed contrary to the agreed sequence.
