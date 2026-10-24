"""Generate human-readable views from the canonical v1 GDD model."""
import json
from pathlib import Path
r=Path(__file__).resolve().parent
m=json.loads((r/'gdd.json').read_text());n=m['nodes']
def typ(t):
 k=t['type']
 if k=='reference':v=t['target'].removeprefix('entity.')+' ID'
 elif k=='enum':v=' | '.join(t['values'])
 elif k=='object':v='{ '+', '.join(k+': '+typ(v) for k,v in t.get('properties',{}).items())+' }'
 elif k=='array':v='list<'+typ(t['items'])+'>'
 else:v=k
 return v+('?' if t.get('optional') else '')+(' or null' if t.get('nullable') else '')
lines=['# Vara v1 — module contracts','', 'Derived from `gdd.json`; the graph owns action/type/scenario details. Regenerate this view with `python3 render_contracts.py`. These are implementation contracts and proposed design boundaries, not executed application evidence.','', '## Ownership','', '| Module | Owns | Excludes |','| --- | --- | --- |']
for id,x in n.items():
 if x['kind']=='component':lines.append(f"| `{id}` | {x['boundary']['responsibility']} | {' '.join(x['boundary']['excludes'])} |")
lines+=['','## Main flow','','```mermaid','flowchart LR','  UI[Experience] --> Session','  Session --> Preferences','  Session --> Desktop[Desktop integration]','  Session --> Engines','  Session --> History[History and lifecycle]','  Engines --> Speech[Speech adapters]','  Engines --> Text[Text adapters]','```','','One application, shared Rust domain core. Public calls are explicit `calls` edges. Adapters do not own session policy, delivery or history. Module identifiers describe responsibility, not a required crate count.','','## Shared boundary semantics','','- Entity references are opaque logical identities, not implicit embedded payloads. Audio, staged text and secrets stay behind private handles. UI receives only the data projection it needs.','- Timestamps are UTC instants; the implementation defines serialized ID/time formats consistently. Audio expiry derives from original capture creation, never retry time.','- Generation identity travels through async calls. The session rejects stale responses independently of provider cancellation support.','- New profiles are empty, direct and local. App identity, prompt, engine and policy are snapshotted at invocation.','- Result commit precedes irreversible OS dispatch. A durable result is not proof that another app received text. Uncertain dispatch is never automatically replayed.','- Errors are typed domain outcomes; redact secrets and content from diagnostics. A successful operation may still carry explicit nonfatal cleanup state.','- `effects_complete` and `dependencies_complete` describe declared contract coverage, not proof that a future implementation conforms.','','## Data contracts','','`?` means the field may be absent; `or null` permits a present null value. Reference IDs do not automatically embed the referred entity.','']
for id,x in n.items():
 if x['kind']=='entity':
  lines+=['### '+x['label'],f'`{id}`','',x['description'],'','| Field | Type |','| --- | --- |']
  for field,t in x['fields'].items():lines.append(f'| `{field}` | {typ(t).replace("|", " / ")} |')
  lines.append('')
for cid,c in n.items():
 if c['kind']!='component':continue
 lines+=['## '+c['label'],'']
 for id in c['interfaces']:
  a=n[id];v=a['contract'];lines+=['### '+a['label'],f'`{id}`','','**Input:** '+('; '.join(f'`{k}`: {typ(t)}' for k,t in v['input'].items()) or 'none')+'.','', '**Output:** '+typ(v['output'])+'.','',a['behavior'],'','**Guarantee:** '+ ' '.join(v['postconditions']),'','**Effects:** '+(' '.join(v['external_effects']) or 'No independent external effects.'),'']
  if v['errors']:
   lines+=['| Error | Required behavior |','| --- | --- |']
   lines += [f'| `{code}` | {desc} |' for code,desc in v['errors'].items()];lines.append('')
  lines+=['Scenario IDs: '+', '.join('`'+e['from']+'`' for e in m['edges'] if e['relation']=='exercises' and e['to']==id)+'.','']
lines+=['## Verification boundary','','GDD validates model shape, types, references, explicit dependencies, ownership and structural scenario coverage. The scenario prose must become implementation tests; its presence is not a passed test. UI designs remain proposed and no human design approval is claimed. Model/runtime choice, measured quality/latency and all target-platform runtime acceptance remain visible acceptance questions.','']
(r/'CONTRACTS.md').write_text('\n'.join(lines))
