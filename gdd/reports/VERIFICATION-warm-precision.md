# Warm precision — design specification verification

Executed 2026-10-23. Current GDD revision: `sha256:6c3f357d5efbd0a144797a6770f28066903731d25ca360297e781c851fa012b7`.

## Actual checks

| Check | Outcome |
| --- | --- |
| Official @google/design.md 0.4.0 lint | 0 errors, 4 intentional orphan-color warnings, 1 token-summary info |
| Independent declared contrast pairs | 36 opaque sRGB pair checks PASS |
| Canonical section order | All eight expected headings present in order |
| Local document/image links | All DESIGN.md local links resolve |
| Referenced public GDD action IDs | All exist in the current graph |
| GDD model schema, semantics and record integrity | PASS |
| Core structural readiness | PASS; no application behavioral proof |
| Generated graph projection | PASS |
| Screen readiness | UNKNOWN: no current trusted readiness_review decision is fabricated |

The four lint warnings identify brand-signal, focus, focus-inverse and
success-inverse. Their uses are specified in prose for brand/selection marks,
focus outlines and the readiness indicator. The core alpha component properties
do not encode outline/icon color. They are deliberately retained and explained
in DESIGN.md rather than hidden by invented component entries.

The contrast check includes muted text on selected rows, all action fills,
navigation text, semantic notices, required control boundaries, light/dark focus
outlines and the local-ready indicator. Decorative dividers are intentionally
subtle and are not used as the only boundary of a required control.

## Scope and limitations

The first mockup was visually inspected and its structure translated into exact
tokens and 20 component families. Exact font/geometry/color values and invisible
states are authored design choices, not measurements of an original design file.
The selected direction is a direct user decision; the agent's detailed component
specification is not a separate human approval of every value or state.

No app UI, HTML prototype, component code or font installation was changed in this
documentation task. No new native/platform, keyboard, screen-reader, rendering or
runtime behavior is claimed tested. The earlier prototype's flow evidence remains
bound to its earlier recorded HTML hash. The existing prototype does not yet
implement this detailed visual system.

Prior DESIGN.md, model, records, graph and snapshot are preserved under
history/design-before-warm-precision. Current reports are named *warm-precision*
to keep historical v1 results separate.
