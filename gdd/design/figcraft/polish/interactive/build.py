#!/usr/bin/env python3
"""Assemble all polished states using unchanged corrected baseline importer."""
import pathlib,shutil
D=pathlib.Path(__file__).resolve().parent
base=D.parents[1]
for n in ['18-hud-recording','19-hud-transcribing','20-hud-processing']:
 shutil.copy2(base/'svg'/f'{n}.svg',D/'svg'/f'{n}.svg')
source=(base/'build.py').read_text();source=source[source.index('import hashlib, tempfile, shutil'):]
source='import base64,json,os,pathlib,subprocess,zipfile,re\nD=pathlib.Path('+repr(str(D))+')\nCLI=os.environ.get("FIGCRAFT_CLI","/Users/mikkelfreltoftkrogsholm/.local/bin/figcraft-cli")\n'+source
source=source.replace("'41-app-rule-disabled']","'41-app-rule-disabled','42-ai-defaults-enabled','43-history-selection','44-app-notes','45-transcription-import-actions','46-transcription-ai-result']")
source=source.replace("D/'Vara.figcraft'","D/'Vara-interactive.figcraft'")
source=source.replace("01 · Primary screens","01 · Interactive screens").replace("02 · Dialogs and states","02 · Interactive dialogs and states")
exec(compile(source,str(base/'build.py'),'exec'))
