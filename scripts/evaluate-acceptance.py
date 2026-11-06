#!/usr/bin/env python3
"""Offline deterministic scoring plus explicit bounded real-engine replay. No LLM judge."""
import argparse
import datetime
import hashlib
import importlib.util
import json
import platform
import re
import statistics
import subprocess
import tempfile
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / 'Samples/acceptance'
MATRIX = {
    'parakeet': ('local.parakeet', None),
    'whisper': ('local.whisper', None),
    'parakeet-qwen': ('local.parakeet', 'local.text'),
    'openai': ('openai.speech', None),
    'openai-openai': ('openai.speech', 'openai.text'),
    'parakeet-openai': ('local.parakeet', 'openai.text'),
    'elevenlabs': ('elevenlabs.speech', None),
    'parakeet-gemini': ('local.parakeet', 'gemini.text'),
    'gemini': ('gemini.speech', None),
    'gemini-gemini': ('gemini.speech', 'gemini.text'),
    'elevenlabs-openai': ('elevenlabs.speech', 'openai.text'),
}
LOCAL = {'local.parakeet', 'local.whisper', 'local.text'}


def normalize(text):
    # No number expansion, digit removal, stopword/negation removal or stemming.
    return re.findall(r'[^\W_]+', unicodedata.normalize('NFC', text).casefold(), re.UNICODE)


def canonical(text, case):
    for group in case.get('allowed_orthographic_variants', []):
        for variant in sorted(group['variants'], key=len, reverse=True):
            text = text.replace(variant, group['canonical'])
    return normalize(text)


def difference(reference, actual):
    """Levenshtein word edits, with deterministic substitution/deletion/insertion ties."""
    dp = [[0] * (len(actual) + 1) for _ in range(len(reference) + 1)]
    for i in range(len(reference) + 1): dp[i][0] = i
    for j in range(len(actual) + 1): dp[0][j] = j
    for i, word in enumerate(reference, 1):
        for j, other in enumerate(actual, 1):
            dp[i][j] = min(dp[i-1][j-1] + (word != other), dp[i-1][j] + 1, dp[i][j-1] + 1)
    edits = []
    i, j = len(reference), len(actual)
    while i or j:
        if i and j and reference[i-1] == actual[j-1] and dp[i][j] == dp[i-1][j-1]:
            i -= 1; j -= 1
        elif i and j and dp[i][j] == dp[i-1][j-1] + 1:
            edits.append({'kind':'substitution', 'reference_index':i-1, 'actual_index':j-1, 'expected':reference[i-1], 'actual':actual[j-1]}); i -= 1; j -= 1
        elif i and dp[i][j] == dp[i-1][j] + 1:
            edits.append({'kind':'omission', 'reference_index':i-1, 'expected':reference[i-1]}); i -= 1
        else:
            edits.append({'kind':'insertion', 'actual_index':j-1, 'actual':actual[j-1]}); j -= 1
    edits.reverse()
    return {'wer':dp[-1][-1] / len(reference) if reference else (0.0 if not actual else None),
            'reference_words':len(reference), 'edits':edits, 'edit_count':dp[-1][-1]}


def contains(tokens, phrase):
    return any(tokens[i:i+len(phrase)] == phrase for i in range(len(tokens)-len(phrase)+1))


def score(case, actual, expected=None, exact_words=False):
    reference = expected if expected is not None else case['reference']['text']
    result = difference(canonical(reference or '', case), canonical(actual, case)) if reference is not None else {'wer':None,'edits':[], 'edit_count':None}
    tokens = normalize(actual)
    facts = [{'kind':fact['kind'], 'value':fact['value'], 'pass':any(contains(tokens, normalize(s)) for s in fact['accepted_surfaces'])} for fact in case.get('critical_facts', [])]
    failures = ['critical_fact'] if any(not f['pass'] for f in facts) else []
    if reference == '' and tokens: failures.append('hallucination_on_non_speech')
    if exact_words and result['edit_count']: failures.append('fixed_text_expectation')
    result.update(critical_facts=facts, failure_categories=failures,
                  status='fail' if failures else ('pass' if facts or reference == '' or expected is not None else 'not_tested'))
    return result


def structure(case, transcript, validation_error):
    """Compare ordered source-clip placements, never claim acoustic word alignment."""
    if not transcript:
        return {'status':'not_tested', 'reason':'no rich transcript'}
    words = [w for s in transcript['segments'] for w in s['words']]
    intervals = case.get('anchors', case.get('turns', []))
    checks = []
    for anchor in intervals:
        placed = [w for w in words if anchor['start_seconds']-2 <= w['start'] <= anchor['end_seconds']+2]
        tokens = normalize(' '.join(w['text'] for w in placed))
        wanted = normalize(anchor['text'])
        # Sequence count exposes missing/duplicated full boundary text; approximate matches remain failures.
        count = sum(tokens[i:i+len(wanted)] == wanted for i in range(max(0,len(tokens)-len(wanted)+1)))
        checks.append({'start_seconds':anchor['start_seconds'], 'end_seconds':anchor['end_seconds'],
                       'source_case':anchor.get('source_case',anchor.get('source_label')), 'full_reference_occurrences':count,
                       'pass':count == 1, 'placed_tokens':tokens})
    raw_speakers = sorted({w['speaker'] for w in words if w.get('speaker')} | {s['speaker'] for s in transcript['segments'] if s.get('speaker')})
    # Word timestamps absent => anchor comparison not performed, not a pass.
    return {'status':('fail' if validation_error else 'not_tested' if not words else 'pass' if not validation_error and all(c['pass'] for c in checks) else 'fail'),
            'validation_error':validation_error, 'anchors':checks, 'raw_speaker_ids':raw_speakers,
            'speaker_identity':'not_tested: IDs can be chunk-scoped; source identity is not inferred',
            'exact_acoustic_alignment':'not_tested', 'player_seeking':'not_tested with this transcript; separate UI fixture check in 47-quality-run.md'}


def inventory(manifest, cleanup):
    rows = []
    def add(path, case, speech, text, profile=None, repeat=0):
        rows.append({'id':f'{path}/{case}/{profile or "direct"}/{repeat}', 'case':case, 'speech':speech,'text':text,'profile':profile})
    for path,(speech,text) in MATRIX.items():
        add(path, 'da-000', speech,text,'punctuation_only' if text else None)
    for path in ('parakeet','whisper','openai','elevenlabs','gemini'):
        speech,text = MATRIX[path]
        for repeat in (1,2): add(path,'da-000',speech,text,repeat=repeat)
    for path in ('parakeet','whisper'):
        for case in ('da-001','en-000','mixed-spliced','da-paused','da-fast-structural','silence','noise','long-boundaries','four-speakers'):
            add(path,case,*MATRIX[path])
    for path in ('parakeet','whisper'):
        for case in ('silence','noise'):
            add(path+'-explicit-da',case,*MATRIX[path])
            rows[-1]['language']='da'
    for engine in ('local.text','openai.text','gemini.text'):
        for profile in cleanup['profiles']:
            for case in ('text-names-amount-date','text-instruction-as-data','text-similar-numbers'):
                add(engine,case,None,engine,profile)
    return rows


def is_cloud(row):
    return any(e and e not in LOCAL for e in (row['speech'], row['text']))


def report(rows, observations, manifest, cleanup, metadata):
    audio = {c['id']:c for c in manifest['cases']}
    texts = {c['id']:c for c in cleanup['cases']}
    results=[]
    for row in rows:
        obs=observations.get(row['id'])
        result=dict(row, schema_version=1, metadata=metadata, status='not_tested', reason='not executed')
        case=audio.get(row['case'],texts.get(row['case']))
        result['fixture_sha256']=case.get('sha256', hashlib.sha256(json.dumps(case,sort_keys=True).encode()).hexdigest())
        result['reference_origin']=case.get('reference',{}).get('origin',cleanup['provenance']['origin'])
        result['broader_asr_quality']='not_tested: baseline-only, no broad WER threshold' if row['speech'] else 'not_applicable'
        result['reference_status']=case.get('reference',{}).get('status',cleanup['reference_status'])
        if obs:
            result.update(observation=obs,reason=None)
            error=obs.get('error')
            non_speech=row['case'] in ('silence','noise')
            expected_no_speech=non_speech and error in ('engine.no_speech','speech.file_empty')
            if error and not expected_no_speech:
                result.update(status='fail',failure_categories=[f'{obs.get("error_stage","runtime")}_error'])
                if row['speech'] and obs.get('raw') is not None:
                    result['asr']=score(case,obs['raw'])
            else:
                raw=obs.get('raw') or ''
                if row['speech']:
                    raw_score=score(case,raw)
                    result['asr']=raw_score
                    result['status']=raw_score['status']
                if obs.get('final') is not None and row['text']:
                    expected=texts[row['case']]['expected'][row['profile']] if not row['speech'] else raw
                    final_score=score(case,obs['final'], expected, exact_words=row['profile']=='punctuation_only')
                    final_score['language_adherence']='not_tested: no validated language detector; lexical/fact checks only'
                    if row['profile']=='app_specific_prompt':
                        wrapper=obs['final'].splitlines()[0]=='Hej,' and obs['final'].splitlines()[-1]=='Venlig hilsen'
                        final_score['email_wrapper_pass']=wrapper
                        if not wrapper: final_score['status']='fail';final_score['failure_categories'].append('email_wrapper')
                    result['cleanup']=final_score
                    result['status']='fail' if result['status']=='fail' or final_score['status']=='fail' else final_score['status']
                if 'anchors' in case or 'turns' in case:
                    result['structure']=structure(case,obs.get('transcript'),obs.get('validation_error'))
                    result['status']=result['structure']['status'] if result['status']!='fail' else 'fail'
        results.append(result)
    return results


def markdown(results):
    lines=['# Issue 47 — deterministic quality and latency baseline','',
           'NFC + Unicode casefold; punctuation/whitespace delimit tokens. Numbers and negations are retained. Only versioned per-case orthographic aliases apply. WER is descriptive: no broad acceptance threshold has been set from this baseline. Pass means the listed fixed checks passed. Broader ASR quality is baseline-only and not accepted; no WER threshold exists yet. A high-WER row can pass a narrow fact check. Insertions are candidate hallucinations; substitutions can reflect provisional source spelling. No listening or LLM judge.','',
           '| Path | Case / profile / language | Fixed checks | Raw WER | Final WER | Total s |','|---|---|---|---:|---:|---:|']
    for r in results:
        def wer(stage):
            value=r.get(stage,{}).get('wer');return '—' if value is None else f'{value:.3f}'
        total=r.get('observation',{}).get('total_seconds')
        lines.append(f'| {r["speech"] or "text"} → {r["text"] or "direct"} | {r["case"]} / {r["profile"] or "direct"} / {r.get("language") or ("auto" if r["case"] in ("silence","noise","mixed-spliced") else "fixture")} | {r["status"]} | {wer("asr")} | {wer("cleanup")} | {total:.3f} |' if total is not None else f'| {r["speech"] or "text"} → {r["text"] or "direct"} | {r["case"]} / {r["profile"] or "direct"} / {r.get("language") or ("auto" if r["case"] in ("silence","noise","mixed-spliced") else "fixture")} | not_tested | — | — | — |')
    lines+=['','## Short repeated timings','', 'Cold = first request to an engine in one process; for local engines includes loading (OS cache uncontrolled), for cloud engines does not establish server model load. Warm = reused engine. Load cannot be separated from ASR/AI by this adapter. Three short observations support a median and range only; no percentiles or ranking.','']
    for engine in ('local.parakeet','local.whisper','openai.speech','elevenlabs.speech','gemini.speech'):
        observations=[r['observation'] for r in results if r['speech']==engine and not r['text'] and r['case']=='da-000' and 'observation' in r and not r['observation'].get('error')]
        warm=[o['total_seconds'] for o in observations if o['phase']=='warm']
        cold=[o['total_seconds'] for o in observations if o['phase']=='cold']
        lines.append(f'- {engine}: cold {cold}; warm n={len(warm)}'+(f', median {statistics.median(warm):.3f} s, range {min(warm):.3f}–{max(warm):.3f} s.' if warm else '. Not tested.'))
    lines += ['','## Limits','', 'Source references are provisional public dataset transcripts. Structural composites are repeated/spliced clips, not natural long speech or conversation. Anchor checks compare full lexical sequences inside known clip intervals (±2 s); errors, duplicated and missing sequences remain inspectable in JSONL. Segment-only timing cannot establish word anchors. Raw provider speaker IDs are retained without equating IDs across chunks or to source identities. Exact acoustic word alignment and Who labeling remain not_tested. This report does not run the independent UI player fixture; its separate result is in 47-quality-run.md. No automatic prompt tuning, model download, user settings/history or private audio. Cloud rows require explicit fixture and provider filters plus opt-in. Inference errors and unavailable models are failures, never passes.','']
    return '\n'.join(lines)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path)
    parser.add_argument('--observations',type=Path,help='Rescore existing public observations without inference')
    parser.add_argument('--models',type=Path)
    parser.add_argument('--cloud-opt-in',action='store_true')
    parser.add_argument('--providers',help='comma-separated exact engine IDs')
    parser.add_argument('--fixtures',help='comma-separated exact public fixture IDs')
    parser.add_argument('--desktop',action='store_true')
    args=parser.parse_args()
    spec=importlib.util.spec_from_file_location('validator',ROOT/'scripts/validate-acceptance-corpus.py');validator=importlib.util.module_from_spec(spec);spec.loader.exec_module(validator)
    errors=validator.validate(CORPUS)
    if errors: raise SystemExit('\n'.join(errors))
    manifest=json.loads((CORPUS/'manifest.json').read_text());cleanup=json.loads((CORPUS/'cleanup.json').read_text())
    rows=inventory(manifest,cleanup)
    providers=set(args.providers.split(',')) if args.providers else None
    fixtures=set(args.fixtures.split(',')) if args.fixtures else None
    if args.cloud_opt_in and (not providers or not fixtures or not args.desktop): parser.error('live cloud requires explicit providers, fixtures and signed --desktop harness')
    selected=[r for r in rows if (not is_cloud(r) or args.cloud_opt_in) and (not providers or all(not e or e in providers for e in (r['speech'],r['text']))) and (not fixtures or r['case'] in fixtures)]
    plan={'models':str(args.models or ''),'rows':selected,'cloud_opt_in':args.cloud_opt_in,'max_requests':40,'max_audio_seconds':180.0,'timeout_seconds':120}
    metadata={'report_timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'working_tree_dirty':bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip()),'app_revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'hardware':platform.platform()+' '+platform.machine(),'manifest_sha256':hashlib.sha256((CORPUS/'manifest.json').read_bytes()).hexdigest(),'corpus_version':manifest['corpus_version'],'plan':plan,'observations_note':'model revisions/hashes: see pinned engine catalogue and 45-local-runtime.jsonl; cloud model aliases are mutable'}
    if platform.system()=='Darwin':metadata['cpu']=subprocess.check_output(['sysctl','-n','machdep.cpu.brand_string'],text=True).strip()
    if args.binary and args.observations: parser.error('choose inference or rescore')
    observations={}
    if args.observations:
        for line in args.observations.read_text().splitlines():
            observation=json.loads(line);observations[observation['id']]=observation
        metadata['rescored_observations_sha256']=hashlib.sha256(args.observations.read_bytes()).hexdigest()
    metadata['report_source_sha256']={str(path.relative_to(ROOT)):hashlib.sha256(path.read_bytes()).hexdigest() for path in (ROOT/'crates/vara-engine/src/acceptance.rs', ROOT/'scripts/evaluate-acceptance.py', ROOT/'Cargo.lock')}
    metadata['model_artifact_evidence']='45-local-runtime.jsonl; read-only artifacts reused without download'
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.with_suffix('.plan.json').write_text(json.dumps(plan,indent=2)+'\n')
    if args.binary:
        metadata['execution_binary_sha256']=hashlib.sha256(args.binary.read_bytes()).hexdigest()
        if not args.models:parser.error('--binary requires --models')
        # External hard process ceiling also bounds blocking native local inference.
        with tempfile.TemporaryDirectory(prefix='vara-acceptance-') as temp:
            plan_path=Path(temp)/'plan.json';plan_path.write_text(json.dumps(plan))
            command=[str(args.binary.resolve())]+(['--acceptance'] if args.desktop else [])+[str(plan_path)]
            try:
                completed=subprocess.run(command,capture_output=True,text=True,timeout=600)
            except subprocess.TimeoutExpired as error:
                completed=subprocess.CompletedProcess(command,124,error.stdout or '',error.stderr or '')
            stdout=completed.stdout.decode() if isinstance(completed.stdout,bytes) else completed.stdout
            stderr=completed.stderr.decode() if isinstance(completed.stderr,bytes) else completed.stderr
            args.output.with_suffix('.runtime.log').write_text(stderr)
            args.output.with_suffix('.observations.jsonl').write_text(stdout)
            metadata['process_exit_code']=completed.returncode
            for line in stdout.splitlines():
                try:observation=json.loads(line)
                except json.JSONDecodeError:continue
                if 'id' in observation:observations[observation['id']]=observation
    results=report(rows,observations,manifest,cleanup,metadata)
    args.output.with_suffix('.jsonl').write_text(''.join(json.dumps(r,ensure_ascii=False)+'\n' for r in results))
    args.output.with_suffix('.md').write_text(markdown(results))
    print(json.dumps({'rows':len(rows),'executed':len(observations),'statuses':{s:sum(r['status']==s for r in results) for s in ('pass','fail','not_tested')},'output':str(args.output)}))

if __name__=='__main__':main()
