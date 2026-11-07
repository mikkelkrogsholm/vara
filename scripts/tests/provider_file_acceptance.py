#!/usr/bin/env python3
"""Opt-in, exactly two Gemini rich-file production requests on public corpus audio.
Usage: provider_file_acceptance.py SIGNED_HARNESS NEW_OUTPUT --allow-two-cloud-requests
No keys are read by this script; the production backend uses its OS credential API.
No retries, AI processing, microphone, Who model, or personal profile writes.
"""
import hashlib, json, os, pathlib, sqlite3, subprocess, sys, time, wave

# Fixed public assets only. A changed manifest cannot authorize replacement audio.
PINS = {
    'da-000': '823386a11c070104e26aaa6761cc7db68159a6d2e4e4fd83f16ebfbccb2542f7',
    'four-speakers': '0ca72524098483a45f31bfab45ec4aeb17f8bf2b6a30525941858e2e6e62823c',
}

def approved_sources(repo):
    corpus = repo / 'Samples/acceptance'
    manifest = json.loads((corpus / 'manifest.json').read_text())
    cases = {c['id']: c for c in manifest['cases']}
    approved = []
    for case, pinned_sha in PINS.items():
        row = cases[case]
        if row['rights']['cloud_upload_permitted'] is not True:
            raise ValueError('cloud permission missing')
        if row['path'] != 'audio/'+case+'.wav':
            raise ValueError('unexpected public asset path')
        source = corpus / row['path']
        if not row['sha256'] == pinned_sha == hashlib.sha256(source.read_bytes()).hexdigest():
            raise ValueError('public source hash mismatch')
        with wave.open(str(source), 'rb') as audio:
            duration = audio.getnframes() / audio.getframerate()
        if not (0 < duration <= 30 and 0 < row['duration_seconds'] <= 30):
            raise ValueError('single-request duration exceeded')
        if abs(duration-row['duration_seconds']) >= .001:
            raise ValueError('duration manifest mismatch')
        approved.append((case, source, pinned_sha))
    return approved

def main():
    if len(sys.argv) != 4 or sys.argv[3] != '--allow-two-cloud-requests':
        sys.exit(__doc__)
    exe = pathlib.Path(sys.argv[1]).resolve()
    out = pathlib.Path(sys.argv[2]).resolve()
    repo = pathlib.Path(__file__).resolve().parents[2]
    approved = approved_sources(repo)  # BOTH sources checked before any process/key/request.
    out.mkdir(mode=0o700)  # New task-owned directory only.
    profile = out / 'profile'; profile.mkdir(mode=0o700)
    db = sqlite3.connect(profile / 'vara.sqlite3')
    db.execute('CREATE TABLE preferences (id INTEGER PRIMARY KEY, value TEXT NOT NULL)')
    db.execute('INSERT INTO preferences VALUES (1, ?)', (json.dumps({'speech_engine':'gemini.speech'}),))
    db.commit(); db.close()
    env = {k:v for k,v in os.environ.items() if not k.startswith('VARA_')}
    env.update(VARA_DATA_DIR=str(profile), VARA_TEST_BACKEND_IDLE_MS='1000')
    rows = []
    def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
    def run(name, args, timeout=30):
        p = subprocess.run([str(exe), *args], env=env, capture_output=True, timeout=timeout)
        (out / (name+'.stdout')).write_bytes(p.stdout)
        (out / (name+'.stderr')).write_bytes(p.stderr)
        return p
    probe = run('version', ['cli','--version'])
    if probe.returncode != 0 or b'(test-harness)' not in probe.stdout:
        raise ValueError('refusing a non-harness executable')
    try:
        for case, source, pinned_sha in approved:
            before = digest(source)
            if before != pinned_sha:
                raise ValueError('approved public source changed after preflight')
            start = time.monotonic()
            p = run(case, ['cli','transcribe',str(source),'--model','gemini.speech',
                           '--request-id','epic42-48-'+case,'--timeout','550'], timeout=600)
            row = dict(case=case, requests_issued=1, exit_code=p.returncode,
                       elapsed_seconds=time.monotonic()-start, source_sha256=before,
                       source_unchanged=digest(source)==before)
            rows.append(row)
            data = json.loads(p.stdout)
            row.update(job_id=data.get('job_id'), document_id=data.get('document_id'),
                       status=data.get('status'), error=data.get('error'))
            if p.returncode != 0 or not data.get('ok'):
                raise AssertionError('Provider/parser failed; no retry or additional paid request')
            doc = data['document']; transcript = doc['transcript']; duration = doc['duration']
            assert row['job_id'] == row['document_id'] == doc['id']
            assert row['source_unchanged'] and doc['engine']['id'] == 'gemini.speech'
            words = [w for s in transcript['segments'] for w in s['words']]
            for interval in [*transcript['segments'], *words]:
                if interval.get('start') is not None:
                    assert 0 <= interval['start'] <= interval['end'] <= duration + .1, interval
            row.update(duration=duration, timing=transcript['timing'], timing_source=transcript['timing_source'],
                       words=len(words), segments=len(transcript['segments']),
                       speakers=sorted({x['speaker'] for x in [*transcript['segments'], *words] if x.get('speaker')}))
            for fmt in ('txt','json','srt','vtt'):
                exported = run(case+'-'+fmt, ['cli','job','result',doc['id'],'--format',fmt,'--original'])
                row.setdefault('exports',{})[fmt] = dict(exit_code=exported.returncode, bytes=len(exported.stdout))
                assert exported.returncode == (0 if fmt in ('txt','json') or transcript['timing'] != 'none' else 6)
            # Windowless real Tauri AppHandle + production Files adapter, no WebView or insertion.
            native = run(case+'-adapter', ['--vara-test-file-snapshot',str(profile)], timeout=45)
            assert native.returncode == 0
            listed = json.loads(native.stdout)
            visible = next(d for d in listed['documents'] if d['id'] == doc['id'])
            assert visible['stage'] == doc['stage'] == 'ready'
            assert visible['transcript'] == transcript
            row['native_adapter_same_document_and_transcript'] = True
    finally:
        # Backend is task-owned and uses its bounded idle exit; do not kill unrelated apps.
        time.sleep(3)
        (out/'observations.json').write_text(json.dumps(rows, indent=2)+'\n')
    print(json.dumps(rows, indent=2))

if __name__ == "__main__":
    main()
