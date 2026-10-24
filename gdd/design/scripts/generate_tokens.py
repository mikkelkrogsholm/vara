"""Generate prototype CSS from the canonical DESIGN.md. Uses GDD's existing PyYAML."""
from pathlib import Path
import sys
import yaml
root = Path(__file__).resolve().parents[1]
design = root.parent / 'DESIGN.md'
tokens = yaml.safe_load(design.read_text().split('---', 2)[1])
lines = ['/* Generated from gdd/DESIGN.md. Run scripts/generate_tokens.py; do not edit. */', ':root {']
for group, prefix in [('colors', 'color'), ('spacing', 'space'), ('rounded', 'radius')]:
    for key, value in tokens[group].items():
        lines.append(f'  --{prefix}-{key}: {value};')
for name, role in tokens['typography'].items():
    for prop, value in role.items():
        prop = ''.join('-' + c.lower() if c.isupper() else c for c in prop)
        lines.append(f'  --type-{name}-{prop}: {value};')
lines.append('}')
output = '\n'.join(lines) + '\n'
path = root / 'styles/tokens.css'
if '--check' in sys.argv:
    if path.read_text() != output:
        raise SystemExit('tokens.css differs from DESIGN.md; regenerate it.')
    print('Design tokens match DESIGN.md.')
else:
    path.write_text(output)
