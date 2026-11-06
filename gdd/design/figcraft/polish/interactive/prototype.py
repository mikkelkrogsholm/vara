#!/usr/bin/env python3
"""Attach native prototype routes; no website or runtime app is generated."""
import json,pathlib,subprocess,zipfile,hashlib
D=pathlib.Path(__file__).resolve().parent;CLI='/Users/mikkelfreltoftkrogsholm/.local/bin/figcraft-cli';native=D/'Vara-interactive.figcraft'
F=json.loads((D/'frames.json').read_text());C=json.loads((D/'controls.json').read_text());routes=[]
def run(stage,cmd):
 p=D/f'{stage}-commands.json';p.write_text(json.dumps(cmd,indent=2));r=subprocess.run([CLI,'run',str(p),'--in',str(native),'--out',str(native),'--automation-read-root',str(D),'--automation-write-root',str(D)],capture_output=True,text=True,check=True);(D/f'{stage}-results.jsonl').write_text(r.stdout);rows=[json.loads(x) for x in r.stdout.splitlines()if x.startswith('{')];assert len(rows)==len(cmd) and all(x['ok'] for x in rows),r.stdout;return rows
cmd=[];steps={};varsteps={}
def add(command,**params):cmd.append({'command':command,'params':params});return len(cmd)-1
def text(parent,label,x,y,width=480,height=34,size=15):return add('node.create',type='text',parent=parent,name=label,characters=label,x=x,y=y,width=width,height=height,props={'fontFamily':'system-ui','fontSize':size,'fill':'#25251f','textAutoResize':'none'})
add('variable.createCollection',name='Prototype state only',modes=['Preview'])
# Selector choice panels are compact native overlays, never full-screen duplicates.
selectors={}
for name,data in C.items():
 for e in data['controls']:
  if e['tag']=='SELECT'and not e['disabled']:selectors.setdefault(e['id'],e)
for i,(key,e)in enumerate(selectors.items()):
 opts=e['options'];varsteps[key]=add('variable.create',collection='Prototype state only',name=key,type='string',value=opts[0]['label'],description='Prototype selection; source CSS remains design authority')
 steps['select:'+key]=add('node.create',type='frame',name='Choices · '+key,x=i*570,y=8000,width=540,height=min(740,127+len(opts)*34),props={'fill':'#fffcf7','cornerRadius':12,'clip':True})
 # Children are added in the next stage after real frame IDs exist.
for name,data in C.items():
 for e in data['controls']:
  if e['tag']=='INPUT' and e['type'] in ['text','password','url','search'] and not e['label']:
   key='field:'+name+':'+e['id'];varsteps[key]=add('variable.create',collection='Prototype state only',name=key,type='string',value='')
for i,(key,title)in enumerate({'copied':'Copied','saved':'Saved','exported':'File prepared','model':'Model is ready','play':'Playing preview','checked':'Connection checked','changed':'Selection updated','deleted':'Deleted'}.items()):
 steps['feedback:'+key]=add('node.create',type='frame',name='Feedback · '+title,x=i*570,y=9000,width=360,height=92,props={'fill':'#fffcf7','cornerRadius':12,'stroke':'#dcd6ca'})
for n,title in [('01-history','History · start here'),('04-dictation','Settings'),('03-apps','AI per app'),('10-transcription-empty','Transcription'),('09-setup-prepare','Setup'),('35-recovery','Recovery')]:
 add('proto.flow.add',start=F[n],name=title);add('proto.flow.rename',start=F[n],description='Native click-through with fictional examples. Use PageDown / PageUp or the right scrollbar gutter for section continuations. Fields select examples; no free-form input, microphone, network, clipboard or file writes. Escape closes dialogs. Playback and downloads are visual demonstrations.')
add('proto.settings',device='None',responsive=False);add('proto.presentOptions',scale='fit',hotspotHints=True,deviceFrame=False)
rows=run('prototype-setup',cmd);O={key:rows[step]['result']['id']for key,step in steps.items()};V={key:rows[step]['result']['id']for key,step in varsteps.items()};cmd=[]
def interact(node,actions,trigger=None):
 if not isinstance(actions,list):actions=[actions]
 add('proto.interaction.add',id=node,trigger=trigger or {'type':'onClick'},actions=actions,transition={'type':'instant'})
def nav(name):return {'type':'navigate','dest':F[name]}
def openoverlay(key):return {'type':'openOverlay','dest':O[key],'position':'center','close_on_outside':True,'background':'#25251f28'}
def hotspot(parent,rect,label,actions):
 x,y,w,h=rect
 step=add('node.create',type='rectangle',parent=parent,name='Hotspot · '+label,x=x,y=y,width=w,height=h,props={'fill':'#00000000','visible':True,'opacity':1})
 # Selected node is the new hotspot; native schema uses it when id is omitted.
 add('proto.interaction.add',trigger={'type':'onClick'},actions=actions if isinstance(actions,list)else[actions],transition={'type':'instant'})
 return step
for key,e in selectors.items():
 p=O['select:'+key];text(p,'Choose an option',18,10,size=17);text(p,'Current selection',18,40,size=13)
 add('node.create',type='text',parent=p,name='Current choice · '+key,characters='',x=175,y=40,width=345,height=28,props={'fontFamily':'system-ui','fontSize':13,'fill':'#636459'});add('variable.bind',field='characters',variable=V[key])
 for j,opt in enumerate(e['options']):
  text(p,opt['label'],20,86+j*34,width=500,height=30)
  if not opt['disabled']:hotspot(p,[10,82+j*34,520,34],opt['label'],[{'type':'setVariable','variable':V[key],'value':opt['label']},{'type':'closeOverlay'}])
 interact(p,{'type':'closeOverlay'},{'type':'keyPress','key':'Escape'})
for key,title in {'copied':'Copied','saved':'Saved','exported':'File prepared','model':'Model is ready','play':'Playing preview','checked':'Connection checked','changed':'Selection updated','deleted':'Deleted'}.items():
 p=O['feedback:'+key];text(p,title,22,24,width=320,height=40,size=20);interact(p,{'type':'closeOverlay'},{'type':'afterDelay','ms':1000});interact(p,{'type':'closeOverlay'},{'type':'keyPress','key':'Escape'});hotspot(p,[0,0,360,92],'Dismiss feedback',{'type':'closeOverlay'})
# Bind selector labels in existing reusable components to native prototype variables.
doc=json.loads(zipfile.ZipFile(native).read('document.json'));nodes=doc['nodes'];bound=set()
def descendants(identifier):
 n=nodes[str(identifier)]
 for child in n.get('children',[]):
  yield nodes[str(child)];yield from descendants(child)
for name,data in C.items():
 for e in data['controls']:
  if e['tag']!='SELECT' or e['disabled'] or e['id'] not in V:continue
  for n in nodes.values():
   if n.get('type')!='instance' or n.get('parent')!=F[name]:continue
   if abs(n.get('transform',[1,0,0,1,0,0])[4]-e['rect'][0])>2 or abs(n.get('transform',[1,0,0,1,0,0])[5]-e['rect'][1])>2:continue
   for t in descendants(n['id']):
    if t.get('type')=='text' and t['id']not in bound:
     add('variable.set',variable=V[e['id']],value=t.get('characters',''));add('variable.bind',ids=[t['id']],field='characters',variable=V[e['id']]);bound.add(t['id'])
# Native state navigation. Dialog snapshots already contain exactly one backdrop.
main={'history':'01-history','transcription':'10-transcription-empty','apps':'03-apps','settings':'04-dictation'}
tabs={'dictation':'04-dictation','connections':'05-connections','privacy':'06-privacy'}
returns={'07-processing-dialog':'01-history','08-delete-dialog':'01-history','13-who-download':'12-transcription-editor','14-who-known-count':'12-transcription-editor','21-shortcut-dialog':'04-dictation','22-key-dialog':'05-connections','23-app-picker':'03-apps','25-transcription-export':'12-transcription-editor','30-endpoint-dialog':'27-custom-endpoints','31-model-dialog':'27-custom-endpoints'}
nexts={'04-dictation':'37-recording-controls','37-recording-controls':'38-speech-model','38-speech-model':'26-dictation-permissions','26-dictation-permissions':'42-ai-defaults-enabled','42-ai-defaults-enabled':'04-dictation','05-connections':'39-local-models','39-local-models':'29-api-providers','29-api-providers':'27-custom-endpoints','27-custom-endpoints':'28-cli-connections','28-cli-connections':'05-connections','06-privacy':'36-privacy-audio','36-privacy-audio':'06-privacy','10-transcription-empty':'40-transcription-recent','40-transcription-recent':'10-transcription-empty','11-transcription-import':'45-transcription-import-actions','45-transcription-import-actions':'11-transcription-import','24-transcription-ai':'46-transcription-ai-result','46-transcription-ai-result':'24-transcription-ai'}
for name,data in C.items():
 p=F[name];add('page.select',id=nodes[str(p)]['parent'])
 if data['modal']:interact(p,{'type':'back'},{'type':'keyPress','key':'Escape'})
 else:interact(p,{'type':'back'},{'type':'keyPress','key':'Escape'})
 if name in nexts:
  dest=nexts[name];interact(p,nav(dest),{'type':'keyPress','key':'PageDown'});interact(p,nav(dest),{'type':'keyPress','key':'ArrowDown'})
  hotspot(p,[1268,410,12,410],'Scroll to next source section',nav(dest));routes.append({'from':name,'role':'section continuation','rect':[1268,410,12,410],'to':dest})
  prev=next((n for n,d in nexts.items()if d==name),name);interact(p,nav(prev),{'type':'keyPress','key':'PageUp'});hotspot(p,[1268,0,12,410],'Scroll to previous source section',nav(prev))
 for e in data['controls']:
  if e['disabled']or e['tag']in ['LI','A']:continue
  if e.get('action') not in ['save-settings','reset-form'] and e['rect'][0]>=260:
   save=next((q for q in data['controls'] if q.get('action')=='save-settings'),None)
   if save and e['rect'][1]+e['rect'][3]>save['rect'][1]-17:
    e=dict(e);e['rect']=e['rect'][:3]+[max(0,save['rect'][1]-17-e['rect'][1])]
    if e['rect'][3]<=0:continue
  a=e.get('action');key=e['id'];di=e.get('dataId');target=None;actions=None;role=e['label']or key or a or 'Control'
  if a=='navigate':target=main.get(e.get('page'))
  elif a=='settings-tab':target=tabs.get(di)
  elif a in ['close-modal']:actions={'type':'back'}
  elif a=='choose-processing':target='07-processing-dialog'
  elif a=='start':target='18-hud-recording'
  elif a=='confirm-delete':target='08-delete-dialog'
  elif a=='select-result':target='01-history'if di=='1'else'43-history-selection'
  elif a=='select-pref':target='03-apps'if di=='com.apple.mail'else'44-app-notes'
  elif a=='add-app':target='23-app-picker'
  elif a=='record-shortcut':target='21-shortcut-dialog'
  elif a=='open-key':target='22-key-dialog'
  elif a=='add-endpoint':target='30-endpoint-dialog'
  elif a=='edit-engine':target='31-model-dialog'
  elif a=='update-check':target='32-update-available'
  elif a=='update-download':target='33-update-ready'
  elif a=='update-install':target='04-dictation'
  elif a=='tx-pick':target='11-transcription-import'
  elif a=='tx-import':target='11-transcription-import'
  elif a=='tx-start':target='12-transcription-editor'
  elif a=='tx-select':target='12-transcription-editor'if di=='file-1'else'40-transcription-recent'
  elif a=='tx-tab':target='24-transcription-ai'if di=='ai'else'12-transcription-editor'
  elif a=='tx-who-open':target='13-who-download'
  elif a=='tx-who-download':target='14-who-known-count'
  elif a=='tx-who-run':target='15-who-uncertainty'
  elif a=='tx-export':target='25-transcription-export'
  elif a=='tx-ai':target='46-transcription-ai-result'
  elif a=='tx-original':target='12-transcription-editor'if name=='15-who-uncertainty'else'15-who-uncertainty'
  elif a in ['tx-remove','tx-clear','tx-cancel','tx-delete']:target='10-transcription-empty'
  elif a in ['copy','copy-original','tx-copy','tx-copy-ai']:actions=openoverlay('feedback:copied')
  elif a=='tx-export-save':actions=[nav('12-transcription-editor'),openoverlay('feedback:exported')]
  elif a in ['save-key','save-engine']:actions=[nav(returns[name]),openoverlay('feedback:saved')]
  elif a=='save-new-app':actions=[nav('44-app-notes'),openoverlay('feedback:saved')]
  elif a in ['arm-processing']:actions=[nav('01-history'),openoverlay('feedback:saved')]
  elif a=='delete-result':actions=[nav('43-history-selection'),openoverlay('feedback:deleted')]
  elif a in ['install']:actions=openoverlay('feedback:model')
  elif a in ['check-cli']:actions=openoverlay('feedback:checked')
  elif a=='setup-next':target='16-setup-permissions'if name=='09-setup-prepare'else'17-setup-trial'
  elif a=='setup-back':target='09-setup-prepare'if name=='16-setup-permissions'else'16-setup-permissions'
  elif a=='open-dictation':target='04-dictation'
  elif a in ['retry','dismiss_message','delete-recovery']:target='01-history'
  elif a=='reset-form':target='03-apps'if 'app' in name else'04-dictation'
  elif key=='app-ai':target='03-apps'if name=='41-app-rule-disabled'else'41-app-rule-disabled'
  elif key=='general-ai':target='42-ai-defaults-enabled'
  elif key=='direct-text':target='26-dictation-permissions'
  elif e['tag']=='SUMMARY':
   if name in ['01-history','02-original','43-history-selection','35-recovery']:target='01-history'if name=='02-original'else'02-original'
   elif name in ['27-custom-endpoints','28-cli-connections']:target='28-cli-connections'if name=='27-custom-endpoints'else'27-custom-endpoints'
   elif name=='15-who-uncertainty':target='12-transcription-editor'
  elif e['tag']=='SELECT':actions=openoverlay('select:'+key)
  elif a in ['tx-play','tx-seek','tx-selection','tx-rewind','tx-forward']:actions=openoverlay('feedback:play')
  # Text editing is a fixed example interaction; no false free-form input claim.
  elif 'field:'+name+':'+key in V:
   variable=V['field:'+name+':'+key];sample={'api-key':'•••• •••• preview','engine-label':'Example provider','engine-endpoint':'https://example.invalid/v1','engine-model':'example-model','tx-query':'Anna'}.get(key,'Example')
   add('node.create',type='text',parent=p,name='Example value · '+key,characters='',x=e['rect'][0]+12,y=e['rect'][1]+13,width=e['rect'][2]-24,height=26,props={'fontFamily':'system-ui','fontSize':15,'fill':'#25251f'});add('variable.bind',field='characters',variable=variable)
   actions={'type':'setVariable','variable':variable,'value':sample}
  elif e['tag']in ['INPUT','TEXTAREA']or a in ['tx-edit','tx-rename','tx-speaker']:actions=openoverlay('feedback:changed')
  if target:actions=nav(target)
  if actions:
   step=hotspot(p,e['rect'],role,actions);routes.append({'from':name,'role':role,'control':{k:v for k,v in e.items()if k not in ['options','rect']},'rect':e['rect'],'to':target,'actions':actions,'command_index':step})
for name,nextname in [('18-hud-recording','19-hud-transcribing'),('19-hud-transcribing','20-hud-processing'),('20-hud-processing','01-history')]:
 p=F[name];add('page.select',id=nodes[str(p)]['parent']);interact(p,nav(nextname),{'type':'afterDelay','ms':1500});interact(p,nav('01-history'),{'type':'keyPress','key':'Escape'});hotspot(p,[435,20,70,40],'Stop or dismiss dictation',nav('01-history'))
# Make the recovery/setup/error states inspectable without changing product buttons.
add('page.select',id=nodes[str(F['01-history'])]['parent'])
for key,dest in [('F1','09-setup-prepare'),('F2','35-recovery'),('F3','34-update-error')]:interact(F['01-history'],nav(dest),{'type':'keyPress','key':key})
rows=run('prototype-routes',cmd)
for route in routes:
 if 'command_index'in route:route['hotspot_id']=rows[route.pop('command_index')]['result']['id']
(D/'routes.json').write_text(json.dumps({'default':F['01-history'],'frames':F,'overlays':O,'variables':V,'routes':routes,'section_continuations':nexts},indent=2))
print(f'Native routes attached: {len(routes)} hotspots; {len(O)} compact overlays; {len(V)} prototype variables')
