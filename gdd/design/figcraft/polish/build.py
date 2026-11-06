#!/usr/bin/env python3
"""Reuse the corrected native baseline importer, without baseline writes or HUD rebuilds."""
import pathlib
D=pathlib.Path(__file__).resolve().parent
base=D.parent
source=(base/'build.py').read_text()
start=source.index('import hashlib, tempfile, shutil')
source='import base64, json, os, pathlib, subprocess, zipfile, re\nD=pathlib.Path('+repr(str(D))+')\nCLI=os.environ.get("FIGCRAFT_CLI","/Applications/FigCraft Tools/current/figcraft-cli")\n'+source[start:]
selected=['01-history','03-apps','04-dictation','37-recording-controls','39-local-models','29-api-providers','36-privacy-audio','11-transcription-import','12-transcription-editor','24-transcription-ai','22-key-dialog','13-who-download']
a=source.index('primary='); b=source.index('assets={};',a)
source=source[:a]+'primary='+repr(selected[:10])+'\nstates='+repr(selected[10:])+'\n'+source[b:]
source=source.replace("03 · Components","03 · Draft shared components").replace("01 · Primary screens","01 · DRAFT polish screens").replace("02 · Dialogs and states","02 · DRAFT dialogs")
source=source.replace("shutil.copy2(work/'Vara.figcraft',D/'Vara.figcraft')","shutil.copy2(work/'Vara.figcraft',D/'Vara-polish.figcraft')")
# Give metadata useful role/state names only after mapping weight-ID font axes.
naming=r"""
import html
roles={'Qwen · on device':'Text model','Parakeet Redux · on device':'Speech model','Automatic':'Transcription language','English':'Interface language','Hold to talk':'Activation','System default':'Microphone','Detect automatically':'Dictation language · disabled','Until I delete it':'History retention'}
component_names={}
for key,identifier in components.items():
 asset=assets[key];texts=[html.unescape(x) for x in re.findall(r'<text[^>]*>(.*?)</text>',asset['svg'])]
 owner=next(n for n in primary+states if any(a['key']==key for a in screen_assets[n]))
 kind=asset['kind'];value=' '.join(texts)
 if kind=='Sidebar':
  section='History' if owner.startswith('01-') else 'AI per app' if owner.startswith('03-') else 'Transcription' if 'transcription' in owner else 'Settings'
  name=section+' sidebar · '+owner
 elif kind=='Select field':name=roles.get(value,'Select')+' select · '+value+' · '+owner
 elif kind=='Save bar':name='Settings actions · Save disabled / Discard · '+owner
 else:name=value+(' · disabled' if value=='Save app setting' else ' · enabled')+' · '+owner
 component_names[identifier]=name
native_doc=json.loads(zipfile.ZipFile(work/'Vara.figcraft').read('document.json'))
name_commands=[]
for node in native_doc['nodes'].values():
 name=None
 if node['type']=='text':name=node.get('characters','').strip()[:100] or 'Empty text'
 elif node['type']=='component':name=component_names.get(node['id'])
 elif node['type']=='instance':name=component_names.get(node.get('component'))
 if name:name_commands.append({'command':'node.set','params':{'id':node['id'],'name':name}})
run('names',name_commands,work/'Vara.figcraft')
"""
source=source.replace("shutil.copy2(work/'Vara.figcraft',D/'Vara-polish.figcraft')",naming+"\nshutil.copy2(work/'Vara.figcraft',D/'Vara-polish.figcraft')")
exec(compile(source,str(base/'build.py'),'exec'))
