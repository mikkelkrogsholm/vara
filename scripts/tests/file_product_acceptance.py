#!/usr/bin/env python3
"""Offline production queue/CLI/media acceptance. ASR is explicitly scripted.
Usage: file_product_acceptance.py TEST_HARNESS NEW_OUTPUT
FFmpeg makes bounded task-owned media from the public acceptance corpus.
"""
import hashlib, json, os, pathlib, socket, sqlite3, struct, subprocess, sys, time
exe = str(pathlib.Path(sys.argv[1]).resolve())
out = pathlib.Path(sys.argv[2]).resolve(); out.mkdir(mode=0o700)
repo = pathlib.Path(__file__).resolve().parents[2]
profile = out/'profile'; profile.mkdir(mode=0o700)
db = sqlite3.connect(profile/'vara.sqlite3')
db.execute('CREATE TABLE preferences (id INTEGER PRIMARY KEY,value TEXT NOT NULL)')
db.execute('INSERT INTO preferences VALUES(1,?)',(json.dumps({'speech_engine':'gemini.speech'}),)); db.commit(); db.close()
fixture = out/'transcript.json'
plain = {'text':'Scripted offline ASR','timing':'none','timing_source':'fixture','segments':[]}
fixture.write_text(json.dumps(plain))
env = {k:v for k,v in os.environ.items() if not k.startswith('VARA_')}
env.update(VARA_DATA_DIR=str(profile), VARA_TEST_BACKEND_NO_SECRET='1',
           VARA_TEST_BACKEND_IDLE_MS='500', VARA_TEST_ASR_FIXTURE=str(fixture), VARA_TEST_ASR_DELAY_MS='600')
source = repo/'Samples/acceptance/audio/da-000.wav'
def ffmpeg(*args): subprocess.run(['ffmpeg','-hide_banner','-loglevel','error',*map(str,args)],check=True,timeout=45)
ffmpeg('-i',source,out/'compressed.mp3')
ffmpeg('-f','lavfi','-i','color=c=navy:s=64x64:r=1','-i',source,'-c:v','libx264','-c:a','aac','-shortest',out/'video.mp4')
ffmpeg('-f','lavfi','-i','color=c=navy:s=64x64:r=1','-t','1','-c:v','libx264',out/'no-audio.mp4')
(out/'invalid.bin').write_bytes(b'invalid task-owned media')
def cli(*args, expected=0):
    p=subprocess.run([exe,'cli',*map(str,args)],env=env,capture_output=True,timeout=40)
    assert p.returncode==expected,(args,p.returncode,p.stdout,p.stderr)
    return json.loads(p.stdout)
def backend_socket():
    return pathlib.Path('/tmp')/f'vara-{os.getuid()}'/(hashlib.sha256(str(profile).encode()).digest()[:16].hex()+'.sock')
def rpc(action, **kw):
    cli('models','--json')  # Ensure the profile owner is running.
    path=backend_socket()
    payload=json.dumps({'version':2,'operation':{'action':action,**kw}}).encode()
    with socket.socket(socket.AF_UNIX) as conn:
        conn.settimeout(5); conn.connect(str(path)); conn.sendall(struct.pack('>I',len(payload))+payload)
        def exact(n):
            b=b''
            while len(b)<n:
                part=conn.recv(n-len(b)); assert part; b+=part
            return b
        response=json.loads(exact(struct.unpack('>I',exact(4))[0]))
    assert response['error'] is None,response
    return response['result']
rows=[]
try:
    assert '(test-harness)' in subprocess.run([exe,'cli','--version'],capture_output=True,text=True,check=True).stdout
    for name,path in [('wav',source),('mp3',out/'compressed.mp3'),('video',out/'video.mp4'),('long',repo/'Samples/acceptance/audio/long-boundaries.wav')]:
        before=hashlib.sha256(path.read_bytes()).hexdigest()
        v=cli('transcribe',path,'--request-id','media-'+name)
        assert v['status']=='ready' and v['document']['transcript']==plain
        assert hashlib.sha256(path.read_bytes()).hexdigest()==before
        assert v['document']['duration'] > (300 if name=='long' else 10)
        assert pathlib.Path(v['document']['source_path'])==path
        rows.append(dict(case=name,status='pass',job_id=v['job_id'],duration=v['document']['duration'],source_sha256=before))
    for path in (out/'no-audio.mp4',out/'invalid.bin'):
        v=cli('transcribe',path,expected=4); assert v['status']=='failed'
        rows.append(dict(case=path.name,status='pass',error=v['error']))
    # Cancel a real in-flight ASR fixture, wait past late completion, retry the
    # same durable document via the production backend operation, then restart.
    v=cli('transcribe',source,'--no-wait','--request-id','cancel-retry'); id=v['job_id']
    assert cli('job','cancel',id)['job_id']==id
    time.sleep(.8)
    assert cli('job','status',id,expected=4)['status']=='cancelled'
    rpc('retry',id=id)
    ready=cli('job','wait',id)
    assert ready['job_id']==id and ready['status']=='ready'
    assert sum(d['id']==id for d in rpc('list')['documents'])==1
    time.sleep(2)
    assert not backend_socket().exists(), 'backend must exit before restart check'
    assert cli('job','result',id)['document']==ready['document']
    rows.append(dict(case='cancel-retry-restart-no-duplicates',status='pass',job_id=id))
finally:
    time.sleep(2)
    (out/'observations.json').write_text(json.dumps(rows,indent=2)+'\n')
print(json.dumps(rows,indent=2))
