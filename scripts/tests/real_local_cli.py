#!/usr/bin/env python3
"""Opt-in real local ASR/Who smoke test against a signed normal macOS app.

Creates a new isolated profile, repeated tracked audio and synthetic video; reads models
through a symlink. Does not download models, use private audio, or access keys.
Example: python3 scripts/tests/real_local_cli.py APP MODELS NEW_OUTPUT_DIR [LAUNCHER]
"""
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys

if len(sys.argv) not in (4, 5):
    sys.exit('usage: real_local_cli.py APP MODELS NEW_OUTPUT_DIR [LAUNCHER]')
app, models, out = (Path(p).expanduser().resolve() for p in sys.argv[1:4])
exe = app / 'Contents/MacOS/vara-desktop'
if not exe.is_file() or not (models / 'parakeet-redux').is_dir() or not (models / 'who').is_dir():
    sys.exit('Expected a packaged app and existing Redux and Who models')
entry = [str(Path(sys.argv[4]).expanduser().resolve())] if len(sys.argv) == 5 else [str(exe), 'cli']
if len(sys.argv) == 5 and app != Path('/Applications/Vara.app').resolve():
    sys.exit('The public launcher targets /Applications/Vara.app')
version = subprocess.run([*entry, '--version'], capture_output=True, text=True, check=True).stdout.strip()
if '(test-harness)' in version or not version.startswith('vara '):
    sys.exit('Expected a normal non-harness app')
out.mkdir(mode=0o700)  # Refuse to overwrite or reuse a profile.
(out / 'version.txt').write_text(version + '\n')
profile = out / 'profile'
profile.mkdir(mode=0o700)
(profile / 'models').symlink_to(models, target_is_directory=True)
db = sqlite3.connect(profile / 'vara.sqlite3')
db.execute('CREATE TABLE preferences (id INTEGER PRIMARY KEY, value TEXT NOT NULL)')
db.execute('INSERT INTO preferences VALUES (1, ?)', (json.dumps({'speech_engine':'local.parakeet', 'strict_local':True}),))
db.commit()
db.close()
sample = Path(__file__).resolve().parents[2] / 'Samples/da_test.wav'
if not sample.is_file():
    sys.exit('Tracked Samples/da_test.wav is required')
# 31 repeats span the SDK's 300-second piece boundary, needed by the existing
# known-count cross-piece evidence rule. Nine repeats are used for video.
for name, repeats in (('audio', 30), ('video-audio', 8)):
    subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-stream_loop', str(repeats), '-i', str(sample), '-ar', '16000', '-ac', '1', str(out / (name + '.wav'))], check=True)
subprocess.run(['say', '-v', 'Sara', '-o', str(out / 'synthetic.aiff'),
                'Hej, dette er en syntetisk dansk testoptagelse. Vara kan transskribere lyd og skelne talere.'], check=True)
subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-i', str(out / 'synthetic.aiff'), '-ar', '16000', '-ac', '1', str(out / 'synthetic.wav')], check=True)
subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-f', 'lavfi', '-i', 'color=c=navy:s=320x240:r=1', '-i', str(out / 'video-audio.wav'), '-c:v', 'libx264', '-tune', 'stillimage', '-c:a', 'aac', '-shortest', str(out / 'video.mp4')], check=True)
env = {k:v for k,v in os.environ.items() if not k.startswith('VARA_')}
env['VARA_DATA_DIR'] = str(profile)
for name, source, options in (
    ('audio', out / 'audio.wav', ['--diarize', '--speakers', '1']),
    ('video', out / 'video.mp4', ['--diarize']),
    ('short-known', out / 'synthetic.wav', ['--diarize', '--speakers', '1']),
    ('single-piece-known', out / 'video-audio.wav', ['--diarize', '--speakers', '1']),
):
    args = [*entry, 'transcribe', str(source), '--model', 'local.parakeet', *options, '--request-id', 'real-local-' + name]
    result = subprocess.run(args, env=env, capture_output=True, text=True, timeout=600)
    (out / (name + '.json')).write_text(result.stdout)
    (out / (name + '.stderr')).write_text(result.stderr)
    data = json.loads(result.stdout)
    if name in ('short-known', 'single-piece-known'):
        code = data.get('error', {}).get('code')
        if result.returncode != 4 or code not in ('file.who_unclear', 'who.known_count'):
            sys.exit(f'unexpected short known-count outcome {code}; inspect {out}')
        print(name, 'expected honest failure', code, 'job_id', data.get('job_id'), flush=True)
        continue
    if result.returncode or not data.get('ok') or data.get('status') != 'ready':
        sys.exit(f'{name} failed with exit {result.returncode}; inspect {out}')
    doc = data['document']
    diarization = doc.get('diarization') or {}
    turns = diarization.get('turns') or []
    if any(not (0 <= turn['start'] < turn['end'] <= doc['duration'] + 0.1) for turn in turns):
        sys.exit(f'{name} has out-of-bounds speaker intervals; inspect {out}')
    if not turns:
        sys.exit(f'{name} had no speaker turns; inspect {out}')
    raw = diarization.get('raw_turns')
    print(name, 'job_id', data['job_id'], 'duration', doc['duration'], 'turns', len(turns), 'sdk_turns', len(raw if raw is not None else turns), flush=True)
print('Real local ASR/Who smoke completed; profile and outputs:', out, flush=True)
