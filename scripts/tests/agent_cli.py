#!/usr/bin/env python3
"""Process-level #27 regression. Synthetic WAV; no model, network or credentials.
The ready document is an explicitly seeded transcript fixture, not inference.
Run against the release test-harness desktop executable passed as argv[1].
"""
import hashlib, json, os, pathlib, signal, socket, sqlite3, struct, subprocess, sys, tempfile, time, uuid, wave

exe = str(pathlib.Path(sys.argv[1]).resolve())
# The fixture variables are intentionally ignored by production builds. Probe
# before creating a profile or launching any backend, and never fall back.
probe_env = {k:v for k,v in os.environ.items() if not k.startswith('VARA_TEST_')}
try:
    probe = subprocess.run([exe, 'cli', '--version'], env=probe_env,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=5)
except (OSError, subprocess.TimeoutExpired):
    sys.exit('Refusing: could not verify the test-harness executable')
if probe.returncode != 0 or not probe.stdout.rstrip().endswith(b' (test-harness)'):
    sys.exit('Refusing: executable is not a test-harness build')
tmp = tempfile.TemporaryDirectory(prefix='vara-cli-')
base = pathlib.Path(tmp.name)
audio = base / 'lyd æ space.wav'
with wave.open(str(audio), 'wb') as w:
    w.setparams((1, 2, 16000, 16000, 'NONE', 'not compressed'))
    w.writeframes(b'\x00\x00' * 16000)

def profile(name, default='openai.speech', strict=False):
    root = base / name
    root.mkdir(mode=0o700)
    db = sqlite3.connect(root / 'vara.sqlite3')
    db.execute('CREATE TABLE preferences (id INTEGER PRIMARY KEY, value TEXT NOT NULL)')
    db.execute('INSERT INTO preferences VALUES (1, ?)', (json.dumps({'speech_engine':default,'strict_local':strict}),))
    db.commit(); db.close()
    return root

def settings(root, **kw):
    db = sqlite3.connect(root / 'vara.sqlite3')
    v = json.loads(db.execute('SELECT value FROM preferences').fetchone()[0]); v.update(kw)
    db.execute('UPDATE preferences SET value=?', (json.dumps(v),)); db.commit(); db.close()

def env(root, **kw):
    return {**os.environ, 'VARA_DATA_DIR':str(root), 'VARA_TEST_BACKEND_NO_SECRET':'1',
            'VARA_TEST_BACKEND_DELAY_SECRET_MS':'2200', 'VARA_TEST_BACKEND_IDLE_MS':'400', **kw}

def run(root, *args, expected=0, extra=None, cwd=None):
    p = subprocess.run([exe,'cli',*map(str,args)], env=env(root, **(extra or {})), cwd=cwd,
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=15)
    assert p.returncode == expected, (args, p.returncode, p.stdout, p.stderr)
    assert b'\x1b' not in p.stdout and b'\x1b' not in p.stderr
    return p

def js(p):
    value=json.loads(p.stdout); assert value['schema_version']==1; return value

root=profile('live')
m=js(run(root,'models','--json')); assert m['default_speech_engine']=='openai.speech'
assert all(x['ready'] is not True for x in m['models'] if x['locality']=='cloud')
p=run(root,'transcribe',audio.name,'--no-wait','--request-id','same',cwd=base)
a=js(p); job=a['job_id']; assert a['document_id']==job and ('job_id: '+job).encode() in p.stderr
pending_path=base/'not-yet.txt'; pending=js(run(root,'transcribe',audio,'--no-wait','--request-id','same','--format','txt')); assert pending['job_id']==job
assert js(run(root,'transcribe',audio,'--no-wait','--output',pending_path,expected=2))['error']['code']=='cli.invalid_arguments' and not pending_path.exists()
assert js(run(root,'transcribe',audio,'--no-wait','--output',pending_path,'--overwrite',expected=2))['error']['code']=='cli.invalid_arguments' and not pending_path.exists()
settings(root, speech_engine='groq.speech')
b=js(run(root,'transcribe',audio,'--no-wait','--request-id','same')); assert b['job_id']==job
c=js(run(root,'transcribe',audio,'--no-wait','--request-id','same','--model','openai.speech',expected=2)); assert c['error']['code']=='file.request_conflict'
assert js(run(root,'job','status',job))['job_id']==job
pending=js(run(root,'job','result',job,'--format','txt')); assert pending['job_id']==job and pending['status'] in ('queued','decoding','transcribing')
p=run(root,'job','wait',job,'--timeout','0.1',expected=7); assert js(p)['error']['code']=='cli.timeout'
# Explicit cancellation remains separate from timeout, with durable job identity.
assert js(run(root,'job','cancel',job))['job_id']==job
v=js(run(root,'job','result',job,expected=4)); assert v['job_id']==job
assert js(run(root,'job','status',job,expected=4))['status']=='cancelled'
p=run(root,'transcribe',audio,'--model','openai.speech',expected=3); assert js(p)['error']['code']=='provider.missing_key'
assert b'job_id: ' in p.stderr
p=run(root,'transcribe',audio,'--model','local.parakeet',expected=3); assert js(p)['error']['code']=='file.model_download'
p=run(root,'transcribe',audio,'--model','local.parakeet','--language','da',expected=2); assert js(p)['error']['code']=='file.language_unsupported'
p=run(root,'transcribe',audio,'--bogus','secret-value',expected=2); assert b'secret-value' not in p.stdout+p.stderr
assert js(run(root,'transcribe',base/'missing.wav','--model','openai.speech',expected=2))['error']['code']=='file.source_missing'
p=subprocess.run([os.fsencode(exe),b'cli',b'transcribe',b'bad-\xff'],env=env(root),stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=5); assert p.returncode==2 and json.loads(p.stdout)['error']['code']=='cli.invalid_arguments'
strict=profile('strict',strict=True)
assert js(run(strict,'transcribe',audio,expected=3))['error']['code']=='file.privacy_only'
# A delayed submit response can be lost after acceptance. The generated request ID
# is available before IPC; retrying it finds the same durable job.
late=profile('late'); le={'VARA_TEST_BACKEND_DELAY_SUBMIT_MS':'1800'}
p=run(late,'transcribe',audio,'--no-wait','--timeout','0.15',expected=7,extra=le)
f=js(p); request=f['request_id']; assert request and ('request_id: '+request).encode() in p.stderr
r=js(run(late,'transcribe',audio,'--request-id',request,'--no-wait',extra=le)); lid=r['job_id']
r2=js(run(late,'transcribe',audio,'--request-id',request,'--no-wait',extra=le)); assert r2['job_id']==lid
other=js(run(late,'transcribe',audio,'--no-wait',extra=le)); assert other['job_id']!=lid
p=subprocess.Popen([exe,'cli','transcribe',str(audio)],env=env(late, **le),start_new_session=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
line=p.stderr.readline(); assert line.startswith(b'request_id: '); interrupted_request=line.decode().split()[1]
time.sleep(.25); os.killpg(p.pid,signal.SIGINT); out,_=p.communicate(timeout=2)
assert p.returncode==7 and json.loads(out)['request_id']==interrupted_request
r=js(run(late,'transcribe',audio,'--request-id',interrupted_request,'--no-wait',extra=le)); interrupted_id=r['job_id']
assert js(run(late,'transcribe',audio,'--request-id',interrupted_request,'--no-wait',extra=le))['job_id']==interrupted_id
# Actual SIGINT delivered to the client process group, not the separate backend.
sig=profile('signal')
p=subprocess.Popen([exe,'cli','transcribe',str(audio)],env=env(sig),start_new_session=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
assert p.stderr.readline().startswith(b'request_id: ')
line=p.stderr.readline(); assert line.startswith(b'job_id: '); sid=line.decode().split()[1]
os.killpg(p.pid,signal.SIGINT); out,err=p.communicate(timeout=4); assert p.returncode==7 and json.loads(out)['job_id']==sid
p=subprocess.run([exe,'cli','job','status',sid],env=env(sig),stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=5); assert p.returncode in (0,3) and json.loads(p.stdout)['status'] in ('decoding','transcribing','failed')
# Closed downstream pipe terminates the waiter, while backend retains the job.
pipe=profile('pipe')
p=subprocess.Popen([exe,'cli','transcribe',str(audio)],env=env(pipe),stdout=subprocess.PIPE,stderr=subprocess.PIPE)
assert p.stderr.readline().startswith(b'request_id: ')
line=p.stderr.readline(); assert line.startswith(b'job_id: '); pid=line.decode().split()[1]; p.stdout.close(); assert p.wait(timeout=4)==7
p=subprocess.run([exe,'cli','job','status',pid],env=env(pipe),stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=5); assert p.returncode in (0,3) and json.loads(p.stdout)['job_id']==pid
# Delayed actual backend status RPC proves timeout remains responsive during IPC.
slow=profile('slow'); e={'VARA_TEST_BACKEND_DELAY_STATUS_MS':'2000'}
p=run(slow,'transcribe',audio,'--no-wait',extra=e); slowid=js(p)['job_id']
t=time.monotonic(); p=run(slow,'job','wait',slowid,'--timeout','0.15',expected=7,extra=e)
assert time.monotonic()-t < 1.5 and js(p)['job_id']==slowid
# Signal and pipe closure also interrupt an in-flight delayed status RPC.
p=subprocess.Popen([exe,'cli','job','wait',slowid],env=env(slow, **e),start_new_session=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
time.sleep(.2); t=time.monotonic(); os.killpg(p.pid,signal.SIGINT); out,_=p.communicate(timeout=2)
assert p.returncode==7 and time.monotonic()-t<1.5 and json.loads(out)['job_id']==slowid
p=subprocess.Popen([exe,'cli','job','wait',slowid],env=env(slow, **e),stdout=subprocess.PIPE,stderr=subprocess.PIPE)
time.sleep(.2); t=time.monotonic(); p.stdout.close(); assert p.wait(timeout=2)==7 and time.monotonic()-t<1.5

# A persisted ready document exercises shared core.export through the real CLI/backend.
# This text/timing is a deliberately seeded fixture, never a claim of inference.
ready=profile('ready'); rid=str(uuid.uuid4())
engine={'id':'groq.speech','label':'Fixture','kind':'speech','provider':'groq','model':'fixture','endpoint':'https://secret.example/?token=private-value','locality':'cloud'}
doc={'id':rid,'title':'Fixture','source_name':audio.name,'source_path':str(audio),'created_at':1,'engine':engine,'language':None,'stage':'ready','error':'raw-provider-secret','duration':1.0,'completed_seconds':1.0,'waveform':[],
     'transcript':{'text':'Fixture words','timing':'word','timing_source':'fixture','segments':[{'id':'s','text':'Fixture words','start':0.0,'end':0.9,'speaker':None,'words':[{'id':'w','text':'Fixture words','start':0.0,'end':0.9,'speaker':None}]}]},
     'edited_text':None,'revision':0,'ai_text':None,'ai_engine':engine,'speakers':{},'auto_speakers':{'who-0':1},'diarization':None,'diarization_error':'raw-who-secret'}
db=sqlite3.connect(ready/'transcriptions.sqlite3'); db.execute('CREATE TABLE documents(id TEXT PRIMARY KEY,created_at INTEGER NOT NULL,value TEXT NOT NULL)'); db.execute('INSERT INTO documents VALUES(?,?,?)',(rid,1,json.dumps(doc)))
other_audio=base/'other source.wav'; other_audio.write_bytes(b'OTHER AUDIO FIXTURE')
plainid=str(uuid.uuid4()); plain={**doc,'id':plainid,'source_path':str(other_audio),'source_name':other_audio.name,'transcript':{'text':'Plain fixture','timing':'none','timing_source':'none','segments':[{'id':'plain','text':'Plain fixture','start':None,'end':None,'speaker':None,'words':[]}]}}
db.execute('INSERT INTO documents VALUES(?,?,?)',(plainid,1,json.dumps(plain)))
db.execute('CREATE TABLE file_requests(request_id TEXT PRIMARY KEY,parameters TEXT NOT NULL,document_id TEXT NOT NULL UNIQUE)')
params=json.dumps({'source':str(audio.resolve()),'engine':None,'language':None},sort_keys=True,separators=(',',':'),ensure_ascii=False)
db.execute('INSERT INTO file_requests VALUES(?,?,?)',('ready-fixture',params,rid)); db.commit(); db.close()
(ready/'transcriptions').mkdir(); (ready/'transcriptions'/f'{rid}.wav').write_bytes(audio.read_bytes())
assert js(run(ready,'transcribe',audio,'--request-id','ready-fixture','--no-wait'))['job_id']==rid
waited=js(run(ready,'transcribe',audio,'--request-id','ready-fixture')); assert waited['job_id']==rid and waited['document']['transcript']['timing']=='word'
assert js(run(ready,'job','result',plainid,'--format','srt',expected=6))['error']['code']=='transcription.subtitle_timing'
r=js(run(ready,'job','result',rid)); assert r['timing_capability']=='word_requested' and 'auto_speakers' not in r['document'] and r['document']['transcript']['timing_source']=='fixture' and r['document']['transcript']['segments'][0]['words'][0]['text']=='Fixture words'
assert r['document']['engine']['endpoint'] is None and r['document']['ai_engine']['endpoint'] is None and 'secret' not in json.dumps(r)
stdout_document=r['document']
for fmt in ('txt','md','srt','vtt'):
    p=run(ready,'job','result',rid,'--format',fmt); assert b'Fixture words' in p.stdout and not p.stdout.startswith(b'{')
for fmt in ('json','txt','md','srt','vtt'):
    target=base/('output.'+fmt); r=js(run(ready,'job','result',rid,'--format',fmt,'--output',target)); assert r['output']['format']==fmt and target.exists()
    if fmt=='json':
        exported=json.loads(target.read_text()); assert exported['engine']['endpoint'] is None and exported['ai_engine']['endpoint'] is None and 'secret' not in target.read_text() and exported['transcript']['text']=='Fixture words' and exported==stdout_document
        stored=json.loads(sqlite3.connect(ready/'transcriptions.sqlite3').execute('SELECT value FROM documents WHERE id=?',(rid,)).fetchone()[0]); assert stored['engine']['endpoint']==engine['endpoint'] and stored['diarization_error']=='raw-who-secret'
    original=target.read_bytes()
    failure=js(run(ready,'job','result',rid,'--format',fmt,'--output',target,expected=6))
    assert failure['job_id']==rid and failure['status']=='ready' and target.read_bytes()==original
    assert js(run(ready,'job','status',rid))['status']=='ready'
    run(ready,'job','result',rid,'--format',fmt,'--output',target,'--overwrite')
readonly=base/'readonly-output'; readonly.mkdir(mode=0o500)
try:
    failure=js(run(ready,'job','result',rid,'--output',readonly/'out.json',expected=6))
    assert failure['error']['code']=='file.export_save' and not (readonly/'out.json').exists()
finally:
    readonly.chmod(0o700)
run(ready,'job','result',rid,'--output',audio,'--overwrite',expected=6)
before_other=other_audio.read_bytes(); failure=js(run(ready,'job','result',rid,'--output',other_audio,'--overwrite',expected=6)); assert failure['job_id']==rid and other_audio.read_bytes()==before_other
case_alias=base/'OTHER SOURCE.WAV'
if case_alias.exists():
    run(ready,'job','result',rid,'--output',case_alias,'--overwrite',expected=6); assert other_audio.read_bytes()==before_other
client_dir=base/'client cwd'; client_dir.mkdir(); relative=js(run(ready,'job','result',rid,'--format','txt','--output','interview.txt',cwd=client_dir)); assert relative['output']['path']=='interview.txt' and (client_dir/'interview.txt').read_text()=='Fixture words'
link=base/'source-link'; link.symlink_to(audio); run(ready,'job','result',rid,'--output',link,'--overwrite',expected=6)
hard=base/'source-hard'; os.link(audio,hard); run(ready,'job','result',rid,'--output',hard,'--overwrite',expected=6)
run(ready,'job','result',rid,'--output',ready/'private','--overwrite',expected=6)
work=ready/'transcriptions'/f'{rid}.wav'; before_audio=audio.read_bytes(); before_work=work.read_bytes()
worklink=base/'work-hard'; os.link(work,worklink); run(ready,'job','result',rid,'--output',worklink,'--overwrite',expected=6)
dblink=base/'profile-hard'; os.link(ready/'vara.sqlite3',dblink); run(ready,'job','result',rid,'--output',dblink,'--overwrite',expected=6)
assert audio.read_bytes()==before_audio and work.read_bytes()==before_work
assert js(run(ready,'job','status',rid))['status']=='ready'
assert not list(base.glob('.vara-export-*.tmp'))
# Synthetic ASR and Who fixtures exercise the real queue and CLI/backend. These
# are deliberately injected only into marked test-harness builds, not inference.
asr_fixture=base/'asr-fixture.json'; synthetic_asr=json.loads(json.dumps(doc['transcript']))
synthetic_asr['segments'][0]['words']=[{'id':'w1','text':'Fixture','start':0.0,'end':0.4,'speaker':None},{'id':'w2','text':'words','start':0.5,'end':0.9,'speaker':None}]
asr_fixture.write_text(json.dumps(synthetic_asr))
who_fixture=base/'who-fixture.json'; who_fixture.write_text(json.dumps({'model':'fixture','version':'fixture','turns':[{'speaker':'who-0','start':0.0,'end':0.45},{'speaker':'who-uncertain','start':0.45,'end':0.9}]}))
fixture_env={'VARA_TEST_ASR_FIXTURE':str(asr_fixture),'VARA_TEST_WHO_FIXTURE':str(who_fixture),'VARA_TEST_WHO_DELAY_MS':'2000'}
def wait_running(profile_root, job_id, extra):
    end=time.monotonic()+10
    while time.monotonic()<end:
        v=js(run(profile_root,'job','status',job_id,extra=extra))
        if v.get('who',{}).get('stage')=='running': return v
        time.sleep(.05)
    raise AssertionError('Who did not enter running state')
def rpc(r, operation):
    path=pathlib.Path('/tmp')/f'vara-{os.getuid()}'/(hashlib.sha256(str(r.resolve()).encode()).digest()[:16].hex()+'.sock')
    payload=json.dumps({'version':2,'operation':operation}).encode()
    with socket.socket(socket.AF_UNIX) as conn:
        conn.settimeout(5); conn.connect(str(path)); conn.sendall(struct.pack('>I',len(payload))+payload)
        def exact(n):
            b=b''
            while len(b)<n:
                part=conn.recv(n-len(b)); assert part; b+=part
            return b
        return json.loads(exact(struct.unpack('>I',exact(4))[0]))
def list_rpc(r): return rpc(r, {'action':'list'})
who=profile('who-success')
p=run(who,'transcribe',audio,'--diarize','--speakers','2','--no-wait','--request-id','who-id',extra=fixture_env)
wid=js(p)['job_id']; v=wait_running(who,wid,fixture_env); assert v['status']=='processing'
ui_busy=list_rpc(who); ui_doc=next(x for x in ui_busy['result']['documents'] if x['id']==wid); assert ui_doc['who_busy'] and ui_doc['stage']=='ready'
assert js(run(who,'job','result',wid,extra=fixture_env))['status']=='processing'
assert js(run(who,'job','wait',wid,'--timeout','.1',expected=7,extra=fixture_env))['job_id']==wid
settings(who,speech_engine='groq.speech')
assert js(run(who,'transcribe',audio,'--diarize','--speakers','2','--no-wait','--request-id','who-id',extra=fixture_env))['job_id']==wid
assert js(run(who,'transcribe',audio,'--diarize','--speakers','3','--no-wait','--request-id','who-id',expected=2,extra=fixture_env))['error']['code']=='file.request_conflict'
assert js(run(who,'transcribe',audio,'--speakers','2',expected=2,extra=fixture_env))['error']['code']=='cli.invalid_arguments'
v=js(run(who,'job','wait',wid,extra=fixture_env)); d=v['document']; assert d['who_plan']['stage']=='ready'
ui_done=list_rpc(who); ui_doc=next(x for x in ui_done['result']['documents'] if x['id']==wid); assert not ui_doc['who_busy'] and ui_done['revision']>ui_busy['revision'] and ui_doc['speaker_transcript'] is not None
assert d['transcript']['segments'][0]['words'][0]['speaker'] is None
assert d['speaker_transcript']['segments'][0]['words'][0]['speaker']=='who-0'
assert d['speaker_transcript']['segments'][1]['words'][0]['speaker'] is None # uncertainty abstains
assert d['diarization']['turns'][1]['speaker']=='who-uncertain' and 'who-uncertain' not in d['speakers']
# Edited text remains saved separately. Subtitle timing is available only through --original.
# Wait for owner to exit before directly seeding a saved edit in the fixture DB.
def socket_for(r): return pathlib.Path('/tmp')/f'vara-{os.getuid()}'/(hashlib.sha256(str(r.resolve()).encode()).digest()[:16].hex()+'.sock')
def stopped(r):
    end=time.monotonic()+10
    while socket_for(r).exists():
        assert time.monotonic()<end
        time.sleep(.05)
stopped(who)
db=sqlite3.connect(who/'transcriptions.sqlite3'); d=json.loads(db.execute('SELECT value FROM documents WHERE id=?',(wid,)).fetchone()[0]); d['edited_text']='Saved edit'; d['revision']=1; db.execute('UPDATE documents SET value=? WHERE id=?',(json.dumps(d),wid)); db.commit(); db.close()
assert run(who,'job','result',wid,'--format','txt').stdout==b'Saved edit'
assert run(who,'job','result',wid,'--format','txt','--original').stdout==b'Fixture words'
assert js(run(who,'job','result',wid,'--format','srt',expected=6))['error']['code']=='transcription.subtitle_timing'
assert b'Fixture words' in run(who,'job','result',wid,'--format','srt','--original').stdout
assert js(run(who,'job','result',wid,'--original'))['document']['selected_text']=='Fixture words'
asr_failed=profile('asr-before-who-failed')
asr_fail_id=js(run(asr_failed,'transcribe',audio,'--diarize','--no-wait'))['job_id']
assert js(run(asr_failed,'job','wait',asr_fail_id,expected=3))['error']['code']=='provider.missing_key'
db=sqlite3.connect(asr_failed/'transcriptions.sqlite3'); saved=json.loads(db.execute('SELECT value FROM documents WHERE id=?',(asr_fail_id,)).fetchone()[0]); db.close()
assert saved['stage']=='failed' and saved['who_plan']['stage']=='skipped'
# Explicit UI retry restarts a previously skipped Who plan, using the same queue.
retry_root=profile('retry-asr'); retry_asr=base/'retry-asr.json'
retry_env={'VARA_TEST_ASR_FIXTURE':str(retry_asr),'VARA_TEST_ASR_DELAY_MS':'400','VARA_TEST_WHO_FIXTURE':str(who_fixture),'VARA_TEST_WHO_DELAY_MS':'300'}
retry_id=js(run(retry_root,'transcribe',audio,'--diarize','--speakers','2','--no-wait',extra=retry_env))['job_id']
assert js(run(retry_root,'job','wait',retry_id,expected=2,extra=retry_env))['error']['code']=='file.source_missing'
retry_asr.write_bytes(asr_fixture.read_bytes())
assert rpc(retry_root,{'action':'retry','id':retry_id})['error'] is None
pending_doc=next(x for x in list_rpc(retry_root)['result']['documents'] if x['id']==retry_id)
assert pending_doc['who_plan']['stage']=='pending' and pending_doc['who_plan']['known_speaker_count']==2
assert js(run(retry_root,'job','wait',retry_id,extra=retry_env))['document']['who_plan']['stage']=='ready'
# A planned Who failure can be explicitly retried through the real UI RPC.
retry_who_root=profile('retry-who'); retry_who=base/'retry-who.json'
retry_who_env={'VARA_TEST_ASR_FIXTURE':str(asr_fixture),'VARA_TEST_WHO_FIXTURE':str(retry_who),'VARA_TEST_WHO_DELAY_MS':'300'}
retry_who_id=js(run(retry_who_root,'transcribe',audio,'--diarize','--no-wait',extra=retry_who_env))['job_id']
assert js(run(retry_who_root,'job','wait',retry_who_id,expected=4,extra=retry_who_env))['status']=='who_failed'
retry_who.write_bytes(who_fixture.read_bytes())
assert rpc(retry_who_root,{'action':'who','id':retry_who_id,'known_speaker_count':3})['error'] is None
retried=js(run(retry_who_root,'job','result',retry_who_id,extra=retry_who_env)); assert retried['status']=='ready' and retried['document']['who_plan']['known_speaker_count']==3
# Force the actual SQLite publication and terminal-state writes to fail.
unsaved_root=profile('unsaved-who'); unsaved_env={**fixture_env,'VARA_TEST_WHO_DELAY_MS':'2500'}
unsaved_id=js(run(unsaved_root,'transcribe',audio,'--diarize','--no-wait',extra=unsaved_env))['job_id']; wait_running(unsaved_root,unsaved_id,unsaved_env)
db=sqlite3.connect(unsaved_root/'transcriptions.sqlite3'); db.execute("CREATE TRIGGER deny_who_terminal BEFORE UPDATE ON documents WHEN json_extract(NEW.value,'$.who_plan.stage') IN ('ready','failed') BEGIN SELECT RAISE(FAIL,'fixture failure'); END"); db.commit(); db.close()
unsaved=js(run(unsaved_root,'job','wait',unsaved_id,expected=4,extra=unsaved_env)); assert unsaved['status']=='who_failed' and unsaved['error']['code']=='file.result_unsaved'
ui=list_rpc(unsaved_root); current=next(x for x in ui['result']['documents'] if x['id']==unsaved_id)
assert current['stage']=='ready' and current['who_plan']['stage']=='failed' and current['transcript']['text']=='Fixture words' and not current['who_busy']
assert ui['notice'] is not None and (unsaved_root/'transcriptions'/f'{unsaved_id}.wav').exists()
db=sqlite3.connect(unsaved_root/'transcriptions.sqlite3'); db.execute('DROP TRIGGER deny_who_terminal'); db.commit(); db.close()
failed=profile('who-failed'); failure_env={'VARA_TEST_ASR_FIXTURE':str(asr_fixture),'VARA_TEST_WHO_DELAY_MS':'300'}
fid=js(run(failed,'transcribe',audio,'--diarize','--no-wait',extra=failure_env))['job_id']
f=js(run(failed,'job','wait',fid,expected=4,extra=failure_env)); assert f['status']=='who_failed' and f['error']['code'] in ('file.resource_missing','who.runtime_missing')
db=sqlite3.connect(failed/'transcriptions.sqlite3'); saved=json.loads(db.execute('SELECT value FROM documents WHERE id=?',(fid,)).fetchone()[0]); db.close()
assert saved['stage']=='ready' and saved['transcript']['text']=='Fixture words' and (failed/'transcriptions'/f'{fid}.wav').exists()
cancel=profile('who-cancel'); cid=js(run(cancel,'transcribe',audio,'--diarize','--no-wait',extra=fixture_env))['job_id']; wait_running(cancel,cid,fixture_env)
cancel_receipt=js(run(cancel,'job','cancel',cid,extra=fixture_env)); assert cancel_receipt['job_id']==cid and cancel_receipt['status']=='who_cancelled' and cancel_receipt['asr_stage']=='ready'
assert js(run(cancel,'job','wait',cid,expected=4,extra=fixture_env))['status']=='who_cancelled'
time.sleep(.3); db=sqlite3.connect(cancel/'transcriptions.sqlite3'); saved=json.loads(db.execute('SELECT value FROM documents WHERE id=?',(cid,)).fetchone()[0]); db.close()
assert saved['stage']=='ready' and saved['transcript']['text']=='Fixture words' and saved['diarization'] is None and (cancel/'transcriptions'/f'{cid}.wav').exists()
assert rpc(cancel,{'action':'who','id':cid,'known_speaker_count':None})['error'] is None
assert js(run(cancel,'job','result',cid,extra=fixture_env))['status']=='ready'
pending_root=profile('who-pending-cancel'); pe={**fixture_env,'VARA_TEST_ASR_DELAY_MS':'2000'}
pid2=js(run(pending_root,'transcribe',audio,'--diarize','--no-wait',extra=pe))['job_id']; run(pending_root,'job','cancel',pid2,extra=pe)
assert js(run(pending_root,'job','wait',pid2,expected=4,extra=pe))['status']=='cancelled'
# Start and own the backend so its forced interruption cannot affect any user process.
restart=profile('who-restart'); re={**fixture_env,'VARA_TEST_WHO_DELAY_MS':'10000'}
backend=subprocess.Popen([exe,'--vara-internal-file-worker',str(restart)],env=env(restart,**re),stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
end=time.monotonic()+5
while not socket_for(restart).exists():
    assert backend.poll() is None and time.monotonic()<end
    time.sleep(.05)
jid=js(run(restart,'transcribe',audio,'--diarize','--no-wait',extra=re))['job_id']; wait_running(restart,jid,re)
backend.kill(); backend.wait(timeout=5)
f=js(run(restart,'job','status',jid,expected=4,extra=fixture_env)); assert f['status']=='who_failed' and f['error']['code']=='file.interrupted'
db=sqlite3.connect(restart/'transcriptions.sqlite3'); saved=json.loads(db.execute('SELECT value FROM documents WHERE id=?',(jid,)).fetchone()[0]); db.close(); assert saved['stage']=='ready' and saved['transcript']['text']=='Fixture words'

# Wait for every actual fixture backend to finish work and exit on idle.
roots=[root,strict,late,sig,pipe,slow,ready,who,asr_failed,retry_root,retry_who_root,unsaved_root,failed,cancel,pending_root,restart]
deadline=time.monotonic()+15
while True:
    sockets=[pathlib.Path('/tmp')/f'vara-{os.getuid()}'/(hashlib.sha256(str(r.resolve()).encode()).digest()[:16].hex()+'.sock') for r in roots]
    if not any(s.exists() for s in sockets): break
    assert time.monotonic()<deadline, sockets
    time.sleep(.1)
print('agent_cli process regression passed: actual CLI/backend, no-secret synthetic audio; seeded ready transcript fixture')
