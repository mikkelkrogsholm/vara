#!/usr/bin/env python3
"""Export every frame from the native file and generate inspectable comparison evidence."""
import hashlib, html, json, os, pathlib, subprocess, zipfile
from PIL import Image, ImageChops, ImageEnhance, ImageDraw
D=pathlib.Path(__file__).resolve().parent
ROOT=D.parents[2]
CLI=os.environ.get('FIGCRAFT_CLI','/Applications/FigCraft Tools/current/figcraft-cli')
frames=json.loads((D/'frames.json').read_text())
commands=[{'command':'file.fontReport','params':{}}]+[{'command':'file.export','params':{'ids':[identifier],'format':'png','scale':2,'path':str(D/'exports'/f'{name}.png')}} for name,identifier in frames.items()]
commands_file=D/'export-commands.json';commands_file.write_text(json.dumps(commands,indent=2)+'\n')
result=subprocess.run([CLI,'run',str(commands_file),'--in',str(D/'Vara.figcraft'),'--automation-write-root',str(D)],check=True,capture_output=True,text=True)
(D/'export-results.jsonl').write_text(result.stdout)
rows=[json.loads(line) for line in result.stdout.splitlines() if line.startswith('{')]
assert len(rows)==len(frames)+1 and all(row['ok'] for row in rows)
assert rows[0]['result']['missing']==0
info=json.loads(subprocess.check_output([CLI,'info',str(D/'Vara.figcraft'),'--json'],text=True));(D/'info.json').write_text(json.dumps(info,indent=2)+'\n')
doc=json.loads(zipfile.ZipFile(D/'Vara.figcraft').read('document.json'))
old=json.loads(zipfile.ZipFile(D/'history/Vara-26f61b5.figcraft').read('document.json'))
old_names={n['name'] for n in old['nodes'].values() if n.get('type')=='frame' and n.get('parent') in old['pages'][:2]}
assert old_names<=set(frames),old_names-set(frames)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(D/'history/Vara-26f61b5.figcraft')==json.loads((D/'history/source.json').read_text())['sha256']
records=[]
for name,identifier in frames.items():
 reference=Image.open(D/'references'/f'{name}.png').convert('RGBA');exported=Image.open(D/'exports'/f'{name}.png').convert('RGBA')
 assert reference.size==exported.size,(name,reference.size,exported.size)
 # White canvas handles transparent rounded AppKit surfaces equally in both views.
 def flatten(im):
  background=Image.new('RGBA',im.size,'white');background.alpha_composite(im);return background.convert('RGB')
 reference,exported=map(flatten,(reference,exported))
 size=(reference.width//2,reference.height//2)
 reference=reference.resize(size,Image.Resampling.LANCZOS);exported=exported.resize(size,Image.Resampling.LANCZOS)
 pair=Image.new('RGB',(size[0]*2,size[1]),'white');pair.paste(reference);pair.paste(exported,(size[0],0));pair.save(D/'comparisons'/f'{name}.png',optimize=True)
 Image.blend(reference,exported,.5).save(D/'comparisons'/f'{name}-overlay.png',optimize=True)
 ImageEnhance.Contrast(ImageChops.difference(reference,exported)).enhance(3).save(D/'comparisons'/f'{name}-diff.png',optimize=True)
 records.append({'name':name,'frame_id':identifier,'reference_engine':'AppKit source-rendered synthetic HUD' if 'hud-' in name else 'WebKit 26.0 fictional host fixture','logical_size':size,'reference_sha256':sha(D/'references'/f'{name}.png'),'export_sha256':sha(D/'exports'/f'{name}.png'),'native_live_app_verified':False})
manifest={'source_revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'installed_app_version':'1.0.2','native_file_sha256':sha(D/'Vara.figcraft'),'tool_version':subprocess.check_output([CLI,'--version'],text=True).strip(),'font_report':rows[0]['result'],'editable_text_layers':sum(n.get('type')=='text' for n in doc['nodes'].values()),'components':sum(n.get('type')=='component' for n in doc['nodes'].values()),'instances':sum(n.get('type')=='instance' for n in doc['nodes'].values()),'preserved_old_frames':len(old_names),'frames':records,'source_hashes':{str(p.relative_to(ROOT)):sha(p) for p in [ROOT/'gdd/design/styles/tokens.css',ROOT/'gdd/design/styles/app.css',ROOT/'desktop/ui/app.js',ROOT/'desktop/ui/views.js',ROOT/'desktop/ui/i18n.js',ROOT/'desktop/ui/transcription.js',ROOT/'desktop/ui/desktop.css',ROOT/'crates/vara-platform/src/macos_hud.m',ROOT/'crates/vara-core/messages/runtime.json']}}
(D/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
# Local HTML makes every reference/export/overlay/difference accessible without loading a remote service.
links=''.join(f'<tr><td>{html.escape(r["name"])}</td><td>{r["frame_id"]}</td><td>{r["reference_engine"]}</td>'+''.join(f'<td><a href="{folder}/{r["name"]}{suffix}.png">{label}</a></td>' for folder,suffix,label in [('references','','Reference'),('exports','','Export'),('comparisons','','Pair'),('comparisons','-overlay','50% overlay'),('comparisons','-diff','3× contrast difference')])+'</tr>' for r in records)
(D/'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Vara current app design evidence</title><style>body{font:15px system-ui;max-width:1400px;margin:40px auto;background:#f8f5ef;color:#25251f}table{border-collapse:collapse}td,th{padding:8px;border-bottom:1px solid #dcd6ca;text-align:left}a{color:#a84417}</style><h1>Vara 1.0.2 design evidence</h1><p>Left: reference. Right: export from Vara.figcraft. Browser frames use fictional WebKit fixtures. HUD references render the actual AppKit source. These are review artifacts, not a claim of universal pixel equality or native live-app verification.</p><p>All images use a logical 1280×820 viewport (HUD520×76); originals are2×. <a href="README.md">Limits and reproduction</a> · <a href="manifest.json">Source hashes and frame IDs</a> · <a href="visual-review.md">Bounded visual review</a></p><table><tr><th>State</th><th>Frame ID</th><th>Reference</th><th colspan="5">Evidence</th></tr>'+links+'</table>')
# Compact paired sheets for visual inspection; retain the original-size exports above.
for group in range(0,len(records),10):
 sheet=Image.new('RGB',(1040,5*208),'#eee');draw=ImageDraw.Draw(sheet)
 for index,r in enumerate(records[group:group+10]):
  x=(index%2)*520;y=(index//2)*208
  draw.text((x+6,y+4),r['name'],fill='black')
  pair=Image.open(D/'comparisons'/f'{r["name"]}.png');pair.thumbnail((510,180));sheet.paste(pair,(x+5,y+24))
 sheet.save(D/'comparisons'/f'sheet-{group//10+1}.png')
print(f'PASS: {len(frames)} fresh native exports, {len(old_names)} old frames preserved, zero missing fonts, pair/overlay/diff per frame')
