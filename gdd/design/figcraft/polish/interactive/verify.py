#!/usr/bin/env python3
"""Headless native integrity and representative rendering checks; not presenter click tests."""
import json,pathlib,subprocess,zipfile,hashlib
from PIL import Image,ImageChops
D=pathlib.Path(__file__).resolve().parent;F=json.loads((D/'frames.json').read_text());CLI='/Users/mikkelfreltoftkrogsholm/.local/bin/figcraft-cli';native=D/'Vara-interactive.figcraft';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
doc=json.loads(zipfile.ZipFile(native).read('document.json'));nodes=doc['nodes'];baseline=json.loads((D.parents[1]/'frames.json').read_text());assert set(baseline)<=set(F)
commands=[{'command':'file.fontReport','params':{}}]+[{'command':'file.export','params':{'ids':[F[n]],'format':'png','scale':2,'path':str(D/'exports'/f'{n}.png')}}for n in ['01-history','04-dictation','12-transcription-editor']]
(D/'verify-commands.json').write_text(json.dumps(commands,indent=2));r=subprocess.run([CLI,'run',str(D/'verify-commands.json'),'--in',str(native),'--automation-read-root',str(D),'--automation-write-root',str(D)],capture_output=True,text=True,check=True);(D/'verify-results.jsonl').write_text(r.stdout);rows=[json.loads(x)for x in r.stdout.splitlines()if x.startswith('{')];assert all(x['ok']for x in rows);assert rows[0]['result']['missing']==0
flows=[f for p in doc['pages'] for f in nodes[str(p)].get('flows',[])];assert flows[0]['start']==F['01-history'];assert all(f['start']in F.values()for f in flows)
edges={i:set()for i in nodes};external=[];interaction_count=0;hotspots=[]
def actions(items,source):
 global interaction_count
 for a in items:
  if a['type']=='openUrl':external.append(a)
  if 'dest'in a:assert str(a['dest'])in nodes,a;edges[str(source)].add(str(a['dest']))
  for b in a.get('blocks',[]):actions(b.get('actions',[]),source)
def owner(n):
 while n.get('parent')is not None and nodes[str(n['parent'])]['type']!='page':n=nodes[str(n['parent'])]
 return n['id']
for n in nodes.values():
 for it in n.get('interactions',[]):interaction_count+=1;actions(it.get('actions',[]),owner(n))
 if n['name'].startswith('Hotspot ·'):
  hotspots.append(n);assert n.get('visible',True) and n.get('opacity',1)==1
assert not external
reachable=set();todo=[str(F['01-history'])]
while todo:
 k=todo.pop()
 if k in reachable:continue
 reachable.add(k);todo.extend(edges.get(k,()))
assert all(str(f)in reachable for f in F.values()),[n for n,f in F.items()if str(f)not in reachable]
comparison={}
for n in ['01-history','04-dictation','12-transcription-editor']:
 im=Image.open(D/'exports'/f'{n}.png').convert('RGBA');static=Image.open(D.parent/'exports'/f'{n}.png').convert('RGBA');assert im.size==static.size==(2560,1640)
 diff=ImageChops.difference(im,static);comparison[n]={'pixel_identical_to_static_polish':not diff.getbbox(),'difference_bounds':diff.getbbox(),'native_export_sha256':sha(D/'exports'/f'{n}.png')}
assert sha(D.parents[1]/'Vara.figcraft')=='a0b8d5d7739dace528e0854152ed5575f88e89182895bed866f25f5997fd1ab8'
assert sha(D.parent/'Vara-polish.figcraft')=='74c2b7a29fb6fd64cf523e5e8a3d3e3bca26c3b6044a61f1987b89aad35e8cff'
source_hashes=json.loads((D.parent/'manifest.json').read_text())['production_source_hashes']
for p,h in source_hashes.items():assert sha(D.parents[4]/p)==h,p
report={'status':'Native prototype; headless integrity verified; GUI presenter testing separate','tool_version':subprocess.check_output([CLI,'--version'],text=True).strip(),'native_sha256':sha(native),'baseline_native_sha256':sha(D.parents[1]/'Vara.figcraft'),'static_polish_sha256':sha(D.parent/'Vara-polish.figcraft'),'original_frames_preserved':len(baseline),'frames':F,'flows':flows,'prototype_interactions':interaction_count,'hotspot_nodes':len(hotspots),'all_frames_reachable_from_history':True,'external_actions':external,'font_report':rows[0]['result'],'editable_text_layers':sum(n['type']=='text'for n in nodes.values()),'components':sum(n['type']=='component'for n in nodes.values()),'instances':sum(n['type']=='instance'for n in nodes.values()),'visual_comparison':comparison,'production_source_hashes':source_hashes,'proposal_css_sha256':sha(D.parent/'proposal.css')}
(D/'manifest.json').write_text(json.dumps(report,indent=2));print(json.dumps({k:report[k]for k in ['original_frames_preserved','prototype_interactions','hotspot_nodes','all_frames_reachable_from_history','visual_comparison']},indent=2))
