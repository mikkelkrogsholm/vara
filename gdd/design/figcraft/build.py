#!/usr/bin/env python3
"""Assemble fixture captures into an editable FigCraft document using the CLI."""
import base64, json, os, pathlib, subprocess, zipfile, re
D=pathlib.Path(__file__).resolve().parent
CLI=os.environ.get('FIGCRAFT_CLI','/Applications/FigCraft Tools/current/figcraft-cli')
mail=base64.b64encode((D.parents[2]/'desktop/ui/assets/mail.png').read_bytes()).decode()
runtime=json.loads((D.parents[2]/'crates/vara-core/messages/runtime.json').read_text())
for n,title,detail,active in [(n,runtime[title]['en'],runtime[detail]['en'],active) for n,title,detail,active in [('18-hud-recording','hud.recording','hud.audio.local',True),('19-hud-transcribing','hud.recognizing','hud.audio.local',False),('20-hud-processing','hud.processing','hud.ai.local.general',False)]]:
 bars=''.join(f'<rect x="{169+i*5}" y="{38-(3+42*(0.4 if active else 0)*v)/2}" width="3" height="{3+42*(0.4 if active else 0)*v}" rx="1.5" fill="#A84417" opacity="{1 if active and i<12 else .35 if active else .25}"/>' for i,v in enumerate([.14,.25,.42,.65,.82,1,.72,.92,.75,.59,.38,.25,.4,.23,.13,.08]))
 stop='<circle cx="454" cy="38" r="16" fill="#F0EBE3"/><rect x="449" y="33" width="10" height="10" rx="1" fill="#A84417"/>' if active else ''
 svg=f'''<svg xmlns="http://www.w3.org/2000/svg" width="520" height="76"><rect x="0.5" y="0.5" width="519" height="75" rx="24" fill="#FFFCF7" stroke="#DCD6CA"/><image x="20" y="26" width="24" height="24" href="data:image/png;base64,{mail}"/><text id="text-w500-hud-origin" x="55.5" y="40.5" font-family="system-ui" font-size="14" font-weight="500" fill="#25251F">Mail</text>{bars}<text id="text-w500-hud-stage" x="267.5" y="29.5" font-family="system-ui" font-size="14" font-weight="500" fill="#25251F">{title}</text><text id="text-w400-hud-detail" x="268" y="49.75" font-family="system-ui" font-size="13" fill="#636459">{detail}</text>{stop}<path d="M489 33.5 L498 42.5 M498 33.5 L489 42.5" stroke="#A84417" stroke-width="1.2"/></svg>'''
 (D/'svg'/f'{n}.svg').write_text(svg)
# Capture fragments are the rendered elements removed from their screen SVGs.
# Identical fragments share a single main component and become instances.
import hashlib, tempfile, shutil
work=pathlib.Path(tempfile.mkdtemp(prefix="vara-figcraft-"))
primary=['01-history','02-original','03-apps','04-dictation','05-connections','06-privacy','10-transcription-empty','11-transcription-import','12-transcription-editor','24-transcription-ai','26-dictation-permissions','27-custom-endpoints','28-cli-connections','29-api-providers','36-privacy-audio','37-recording-controls','38-speech-model','39-local-models','40-transcription-recent','41-app-rule-disabled']
states=['07-processing-dialog','08-delete-dialog','13-who-download','14-who-known-count','15-who-uncertainty','09-setup-prepare','16-setup-permissions','17-setup-trial','21-shortcut-dialog','22-key-dialog','23-app-picker','25-transcription-export','30-endpoint-dialog','31-model-dialog','32-update-available','33-update-ready','34-update-error','35-recovery','18-hud-recording','19-hud-transcribing','20-hud-processing']
assets={};screen_assets={}
for n in primary+states:
 p=D/'svg'/f'{n}.assets.json'
 screen_assets[n]=json.loads(p.read_text()) if p.exists() else []
 for a in screen_assets[n]:
  key=hashlib.sha256(a['svg'].encode()).hexdigest();a['key']=key
  if key not in assets:assets[key]=a

def run(stage,commands,src=None):
 path=work/f'figcraft-{stage}.json';path.write_text(json.dumps(commands,ensure_ascii=False,indent=2))
 args=[CLI,'run',str(path)]
 if src:args+=['--in',str(src)]
 args+=['--out',str(work/'Vara.figcraft'),'--automation-read-root',str(D),'--automation-write-root',str(work)]
 result=subprocess.run(args,check=True,text=True,capture_output=True)
 (D/f'build-{stage}.jsonl').write_text(result.stdout)
 rows=[json.loads(x) for x in result.stdout.splitlines() if x.startswith('{')]
 if any(not x['ok'] for x in rows):raise RuntimeError(result.stdout)
 return rows
cmd=[{'command':'page.rename','params':{'name':'03 · Components'}}]
keys=list(assets)
for i,key in enumerate(keys):
 a=assets[key];file=work/f'component-{i}.svg';file.write_text(a['svg'])
 cmd += [{'command':'file.import','params':{'path':str(file),'x':(i%4)*550,'y':(i//4)*900}}, {'command':'node.set','params':{'name':a['kind']+' / '+str(i+1)}},{'command':'component.create','params':{}}]
rows=run('components',cmd)
components={key:rows[1+i*3]['result']['id'] for i,key in enumerate(keys)}
cmd=[];steps={};page_steps={}
for page,names in [('01 · Primary screens',primary),('02 · Dialogs and states',states)]:
 page_steps[page]=len(cmd);cmd.append({'command':'page.add','params':{'name':page}})
 for i,n in enumerate(names):
  steps[n]=len(cmd);cmd.append({'command':'file.import','params':{'path':str(D/'svg'/f'{n}.svg'),'x':(i%3)*1350,'y':(i//3)*900}})
rows=run('screens',cmd,work/'Vara.figcraft')
frames={n:rows[step]['result']['id'] for n,step in steps.items()}
cmd=[]
for n in primary+states:
 for a in screen_assets[n]:
  cmd.append({'command':'instance.create','params':{'component':components[a['key']],'parent':frames[n],'x':a['x'],'y':a['y']}})
cmd.extend([{'command':'page.move','params':{'id':rows[page_steps['01 · Primary screens']]['result']['id'],'index':0}},{'command':'page.move','params':{'id':rows[page_steps['02 · Dialogs and states']]['result']['id'],'index':1}},{'command':'page.select','params':{'index':0}}])
run('instances',cmd,work/'Vara.figcraft')
# Apple's variable system font defaults to opsz=28 in the font file. WebKit and
# AppKit select optical size for actual text size (minimum supported axis 17).
# Preserve native named weight; specify the real optical-size axis, never scale text.
document=json.loads(zipfile.ZipFile(work/'Vara.figcraft').read('document.json'))
font_commands=[]
for node in document['nodes'].values():
 style=node.get('style',{})
 if node.get('type')=='text' and style.get('font_family')=='system-ui':
  weight=re.match(r'text-w([\d.]+)-',node.get('name',''))
  css_weight=float(weight[1]) if weight else {'Regular':400,'Medium':500,'Semi Bold':600,'Semibold':600,'Bold':700}.get(style.get('font_style'),400)
  # Apple's named system weights: CSS500→Medium510, CSS600→Semibold590.
  anchors=[(100,30.925),(200,110.725),(300,274.315),(400,400),(500,510),(600,590),(700,700),(800,860),(900,1000)]
  actual_weight=css_weight
  for (a,av),(b,bv) in zip(anchors,anchors[1:]):
   if a<=css_weight<=b: actual_weight=av+(bv-av)*(css_weight-a)/(b-a);break
  font_commands.append({'command':'text.setVariations','params':{'id':node['id'],'variations':{'opsz':max(17,min(96,style['font_size'])),'wght':actual_weight}}})
run('optical-size',font_commands,work/'Vara.figcraft')
shutil.copy2(work/'Vara.figcraft',D/'Vara.figcraft')
(D/'frames.json').write_text(json.dumps(frames,indent=2)+'\n')
print(f'{len(frames)} frames, {len(components)} components, {len(cmd)-3} instances')

shutil.rmtree(work)
