#!/usr/bin/env python3
"""Compare new production captures with preserved approved WebKit evidence."""
from pathlib import Path
import hashlib
import json
from PIL import Image, ImageChops

HERE = Path(__file__).resolve().parent
POLISH = HERE.parent
ROOT = HERE.parents[4]
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
frames = json.loads((POLISH / 'frames.json').read_text())
routes = json.loads((POLISH / 'interactive/routes.json').read_text())['frames']
results = []
for name, frame in frames.items():
    reference = POLISH / 'render-evidence' / f'{name}.png'
    actual = HERE / 'render-evidence' / f'{name}.png'
    before, after = (Image.open(p).convert('RGB') for p in [reference, actual])
    if before.size != after.size:
        raise ValueError(f'{name}: capture size differs')
    diff = ImageChops.difference(before, after)
    bounds = diff.getbbox()
    pixels = sum(pixel != (0, 0, 0) for pixel in diff.getdata())
    results.append(dict(name=name, polishFrame=frame, interactiveFrame=routes[name],
                        differentPixels=pixels, totalPixels=before.width * before.height,
                        bounds=bounds, approvedSha256=sha(reference), actualSha256=sha(actual)))
    if bounds:
        diff.point(lambda value: min(value * 4, 255)).save(HERE / 'render-evidence' / f'{name}-approved-diff.png')
(HERE / 'comparison.json').write_text(json.dumps(results, indent=2) + '\n')
files = ['gdd/design/styles/tokens.css', 'gdd/design/styles/app.css',
         'gdd/design/js/components.js', 'desktop/ui/desktop.css', 'desktop/ui/views.js',
         'gdd/design/figcraft/capture.cjs', 'gdd/design/figcraft/capture-transcription.cjs',
         'gdd/design/figcraft/polish/check.cjs', 'gdd/design/figcraft/polish/proposal.css',
         'gdd/design/figcraft/polish/interactive/Vara-interactive.figcraft',
         'gdd/design/figcraft/polish/interactive/routes.json']
(HERE / 'sources.json').write_text(json.dumps({file: sha(ROOT / file) for file in files}, indent=2) + '\n')
print(f'{len(results)} approved browser comparisons; {sum(r["differentPixels"] for r in results)} differing pixels')
if any(r['differentPixels'] for r in results):
    raise SystemExit(1)
