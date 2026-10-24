# Cloud access: own API keys — 2026-10-23

Direct user answer, copied verbatim:

> Egne API-nøgler altid. Det skal være nemt at tilføje dem, men vi behøver ikke bygge en Vara-tjeneste med login og betaling nu.

Accepted: cloud uses the user’s own API credentials. Make adding/replacing/removing
keys straightforward in the app. Do not build a managed Vara cloud service,
Vara login, usage billing or credential-brokering backend in this scope.
The key-free approachable default is the local workflow. A deliberate switch to
cloud requires the relevant user-supplied key, with clear provider/setup guidance.
The actual provider/model list remains open. This decision does not require
unsupported plaintext credential storage or make key presence proof of validity.
