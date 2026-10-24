"""Generate human-readable views from the canonical v1 GDD model."""
import json
from pathlib import Path
r=Path(__file__).resolve().parent
m=json.loads((r/'gdd.json').read_text());n=m['nodes']
# Focused feature index, generated independently from the module-level contracts.
features=['# Vara v1 — feature index','','Derived from `gdd.json`; see PRODUCT-SPEC.md for accepted release scope and CONTRACTS.md for interfaces. The screens are a first design proposal. The earlier broad ideal exploration is preserved in reports/history/ideal-before-v1.','']
for id,c in n.items():
 if c['kind']!='component':continue
 features+=['## '+c['label'],'']
 for aid in c['interfaces']:features+=['- **'+n[aid]['label']+'** (`'+aid+'`): '+n[aid]['description']]
 features.append('')
features+=['## Remaining engineering acceptance','','No additional product questionnaire is required to begin. These require implementation evidence:','']+[f"- {q['statement']}" for q in m['questions'].values()]+['','Deferred features are listed in PRODUCT-SPEC.md and are not first-release graph obligations.','']
(r/'PRODUCT.md').write_text('\n'.join(features))
