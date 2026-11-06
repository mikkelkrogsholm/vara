#!/usr/bin/env python3
"""Fresh native exports plus draft-to-proposal fidelity evidence and baseline comparison."""
import hashlib, html, json, os, pathlib, subprocess, zipfile
from PIL import Image, ImageChops, ImageEnhance, ImageDraw
D=pathlib.Path(__file__).resolve().parent
CLI=os.environ.get('FIGCRAFT_CLI','/Applications/FigCraft Tools/current/figcraft-cli')
frames=json.loads((D/'frames.json').read_text())
commands=[{'command':'file.fontReport','params':{}}]+[{'command':'file.export','params':{'ids':[v],'format':'png','scale':2,'path':str(D/'exports'/f'{k}.png')}} for k,v in frames.items()]
(D/'export-commands.json').write_text(json.dumps(commands,indent=2)+'\n')
r=subprocess.run([CLI,'run',str(D/'export-commands.json'),'--in',str(D/'Vara-polish.figcraft'),'--automation-write-root',str(D)],capture_output=True,text=True,check=True)
(D/'export-results.jsonl').write_text(r.stdout)
rows=[json.loads(x) for x in r.stdout.splitlines() if x.startswith('{')]
assert len(rows)==13 and all(x['ok'] for x in rows)
assert rows[0]['result']['missing']==0
info=json.loads(subprocess.check_output([CLI,'info',str(D/'Vara-polish.figcraft'),'--json'],text=True));(D/'info.json').write_text(json.dumps(info,indent=2)+'\n')
doc=json.loads(zipfile.ZipFile(D/'Vara-polish.figcraft').read('document.json'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
# Retain only the two controls needed to distinguish proposals from renderer variation.
control_records={}
for name in ['01-history','12-transcription-editor']:
 control=D/'render-evidence'/('control-'+name+'.png')
 if control.exists():
  before=Image.open(D.parent/'references'/(name+'.png')).convert('RGB');fresh=Image.open(control).convert('RGB');diff=ImageChops.difference(before,fresh)
  control_records[name]={'pixel_identical':diff.getbbox() is None,'difference_bounds':diff.getbbox()}
  if name=='01-history':control_records[name]['group_header_identical']=ImageChops.difference(before.crop((460,170,1280,260)),fresh.crop((460,170,1280,260))).getbbox() is None
(D/'control-comparison.json').write_text(json.dumps(control_records,indent=2)+'\n')
records=[]
for n,id in frames.items():
 ref=Image.open(D/'render-evidence'/f'{n}.png').convert('RGB'); native=Image.open(D/'exports'/f'{n}.png').convert('RGB')
 assert ref.size==native.size==(2560,1640)
 ImageEnhance.Contrast(ImageChops.difference(ref,native)).enhance(3).save(D/'render-evidence'/f'{n}-native-diff.png')
 records.append({'name':n,'frame_id':id,'frame_source_svg_sha256':sha(D/'svg'/f'{n}.svg'),'frame_component_fragments_sha256':sha(D/'svg'/f'{n}.assets.json'),'logical_size':[1280,820],'scale':2,'before_reference':'../references/'+n+'.png','before_reference_sha256':sha(D.parent/'references'/f'{n}.png'),'proposal_render_sha256':sha(D/'render-evidence'/f'{n}.png'),'native_export_sha256':sha(D/'exports'/f'{n}.png')})
manifest={'status':'DRAFT — not approved or integrated','baseline_native_sha256':sha(D.parent/'Vara.figcraft'),'draft_native_sha256':sha(D/'Vara-polish.figcraft'),'tool_version':subprocess.check_output([CLI,'--version'],text=True).strip(),'font_report':rows[0]['result'],'editable_text_layers':sum(n.get('type')=='text' for n in doc['nodes'].values()),'components':sum(n.get('type')=='component' for n in doc['nodes'].values()),'instances':sum(n.get('type')=='instance' for n in doc['nodes'].values()),'frames':records,'production_source_hashes':json.loads((D.parent/'manifest.json').read_text())['source_hashes'],'proposal_css_sha256':sha(D/'proposal.css')}
for p,v in manifest['production_source_hashes'].items():assert sha(D.parents[3]/p)==v,p
assert manifest['baseline_native_sha256']=='a0b8d5d7739dace528e0854152ed5575f88e89182895bed866f25f5997fd1ab8'
(D/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
nav=' · '.join(f'<a href="#{n}">{n}</a>' for n in frames)
sections=''.join(f'<section id="{n}"><h2>{n}</h2><div class="pair"><figure><figcaption>Before · current WebKit fixture</figcaption><a href="../references/{n}.png"><img src="../references/{n}.png"></a></figure><figure><figcaption>After · editable native draft export</figcaption><a href="exports/{n}.png"><img src="exports/{n}.png"></a></figure></div><p><a href="render-evidence/{n}.png">Proposal browser render</a> · <a href="render-evidence/{n}-native-diff.png">Native fidelity difference (3× contrast)</a></p></section>' for n in frames)
(D/'review.html').write_text('<!doctype html><html lang="en"><meta charset="utf-8"><title>Vara spacing polish — DRAFT</title><style>body{font:15px/1.5 system-ui;background:#f7f4ee;color:#25251f;margin:32px}a{color:#a84417}nav{max-width:1100px}.pair{display:grid;grid-template-columns:1fr 1fr;gap:16px}figure{margin:0}img{width:100%;display:block}section{margin:40px 0;border-top:1px solid #dcd6ca;padding-top:16px}figcaption{padding:8px 0}h1{font:36px Georgia} @media(max-width:900px){.pair{grid-template-columns:1fr}}</style><h1>Vara spacing polish · DRAFT</h1><p>Design review only. No app implementation or approval. Same fictional fixture data and1280×820 logical viewport at2×. <a href="REVIEW.md">Contract, checks and limits</a> · <a href="Vara-polish.figcraft">Editable native source</a> · <a href="manifest.json">Hashes and frame IDs</a></p><nav>'+nav+'</nav><p><a href="overview.png"><img style="max-width:1280px" src="overview.png" alt="All twelve draft states, before left and after right"></a></p><p>Stress evidence: '+ ' · '.join(f'<a href="render-evidence/{n}{suffix}.png">{n} {label}</a>' for n in ['04-dictation','03-apps','13-who-download'] for suffix,label in [('-1000','1000px'),('-stress','Danish/text spacing'),('-large-text','150% font size'),('-large-text-end','150% end actions')]) +'</p>'+sections+'</html>')
sheet=Image.new('RGB',(1280,6*240),'#f7f4ee');draw=ImageDraw.Draw(sheet)
for i,n in enumerate(frames):
 x=(i%2)*640;y=(i//2)*240;draw.text((x+8,y+5),n,fill='#25251f')
 for j,p in enumerate([D.parent/'references'/f'{n}.png',D/'exports'/f'{n}.png']):
  im=Image.open(p).convert('RGB');im.thumbnail((310,210));sheet.paste(im,(x+8+j*316,y+24))
sheet.save(D/'overview.png')
print(f'PASS: {len(frames)} fresh native frames; {manifest["components"]} components; {manifest["instances"]} instances; {manifest["editable_text_layers"]} text layers; zero missing fonts; baseline/source unchanged')
