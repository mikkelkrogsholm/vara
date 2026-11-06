#!/usr/bin/env python3
"""Capture the real isolated AppKit HUD with fictional labels, without Cargo."""
import os, pathlib, re, subprocess, tempfile
D=pathlib.Path(__file__).resolve().parent
ROOT=D.parents[2]
with tempfile.TemporaryDirectory(prefix='vara-figcraft-hud-') as temporary:
 work=pathlib.Path(temporary)
 tokens=(ROOT/'gdd/design/styles/tokens.css').read_text()
 header='// Selected canonical colors, same conversion as vara-platform/build.rs.\n'
 for name in ['surface','surface-muted','primary','on-surface','muted','divider']:
  rgb=int(re.search(r'--color-'+name+r':\s*#([0-9a-fA-F]+)',tokens)[1],16)
  header+=f'#define VARA_COLOR_{name.upper().replace("-","_")} [NSColor colorWithRed:({rgb>>16}/255.0) green:({(rgb>>8)&255}/255.0) blue:({rgb&255}/255.0) alpha:1]\n'
 (work/'vara_design.h').write_text(header)
 env={**os.environ,'DEVELOPER_DIR':'/Library/Developer/CommandLineTools'}
 subprocess.run(['clang','-fobjc-arc','-fblocks','-I',str(work),'-framework','AppKit','-framework','QuartzCore',str(D/'hud-reference.m'),'-o',str(work/'hud-reference')],check=True,env=env)
 result=subprocess.run([str(work/'hud-reference'),str(ROOT/'crates/vara-core/messages/runtime.json'),str(D/'references')],check=True,capture_output=True,text=True)
 (D/'native-results.txt').write_text(result.stdout)
 print(result.stdout,end='')
