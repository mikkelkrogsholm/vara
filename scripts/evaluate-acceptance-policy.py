#!/usr/bin/env python3
"""Apply baseline-informed fixed-corpus gates offline. Never invokes inference."""
import argparse
import hashlib
import importlib.util
import json
import math
import statistics
import subprocess
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORTS = ROOT / 'gdd/reports/epic42'
POLICY = ROOT / 'Samples/acceptance/regression-policy-v1.json'
SPEC = importlib.util.spec_from_file_location('fixed_checks', ROOT / 'scripts/evaluate-acceptance.py')
fixed = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fixed)


def finite(value):
    return type(value) in (int, float) and math.isfinite(value) and value >= 0


def combine(checks):
    values = [c['status'] for c in checks.values()]
    return 'fail' if 'fail' in values else 'not_tested' if 'not_tested' in values or not values else 'pass'


def check(status, **details):
    return dict(status=status, **details)


def limit(value, maximum):
    if not finite(value):
        return check('not_tested', reason='missing or invalid metric', value=value, maximum=maximum)
    return check('pass' if value <= maximum else 'fail', value=value, maximum=maximum)


def evaluate(row, case, policy):
    """Keep required raw, cleanup, structural and latency gates independent."""
    obs = row.get('observation', {})
    if not obs:
        return {**{k: row.get(k) for k in ('id', 'case', 'speech', 'text', 'profile')},
                'status': 'not_tested', 'gates': {'execution': check('not_tested', reason='not observed')},
                'language_adherence': 'not_tested: no validated detector', 'native_os': 'not_tested', 'who': 'not_tested'}
    gates = {}
    error = obs.get('error')
    non_speech = row['case'] in policy['non_speech_cases']
    expected_error = non_speech and error in policy['no_speech_errors']
    gates['execution'] = check('fail' if error and not expected_error else 'pass' if obs else 'not_tested', error=error)
    if row.get('speech'):
        raw = obs.get('raw')
        if non_speech:
            # Even punctuation-only phantom output is nonempty speech success.
            gates['raw_quality'] = check('pass' if (raw is None and expected_error) or isinstance(raw, str) and not raw.strip() else 'fail')
        elif case.get('reference', {}).get('text') is None:
            gates['raw_quality'] = check('not_tested', reason='no whole-clip reference')
        elif row['case'] in policy['raw_wer_max_by_case']:
            gates['raw_quality'] = limit(row.get('asr', {}).get('wer'), policy['raw_wer_max_by_case'][row['case']])
        else:
            gates['raw_quality'] = check('not_tested', reason='no accepted gate for case')
    for stage, required in (('asr', bool(row.get('speech'))), ('cleanup', bool(row.get('text')))):
        if not required:
            continue
        facts = row.get(stage, {}).get('critical_facts')
        if not isinstance(facts, list) or len(facts) != len(case.get('critical_facts', [])):
            gates[stage + '_critical_facts'] = check('not_tested', reason='missing declared fact metrics')
        else:
            violations = sum(f.get('pass') is not True for f in facts)
            gates[stage + '_critical_facts'] = limit(violations, policy['critical_fact_violations_max'])
    if row.get('text'):
        final = obs.get('final')
        gates['cleanup_output'] = check('pass' if isinstance(final, str) and final.strip() else 'fail')
        if row['profile'] == 'punctuation_only':
            gates['punctuation_only'] = limit(row.get('cleanup', {}).get('edit_count'), policy['punctuation_only_word_edits_max'])
        elif row['profile'] == 'app_specific_prompt':
            lines = final.splitlines() if isinstance(final, str) else []
            gates['email_wrapper'] = check('pass' if lines and lines[0] == policy['email_first_line'] and lines[-1] == policy['email_last_line'] else 'fail')
        elif row['profile'] != 'general_prompt':
            gates['cleanup_profile'] = check('not_tested', reason='unknown profile')
    if row['case'] in policy['structural_cases']:
        # Existing scorer uses captured production validation_error (duration +0.1s)
        # and full lexical clip anchors within placement windows (+/-2s).
        gates['structure'] = row.get('structure', check('not_tested', reason='no word timing'))
    if row['case'] in policy['short_latency_cases']:
        engines = [e for e in (row.get('speech'), row.get('text')) if e]
        known = fixed.LOCAL | {'openai.speech', 'openai.text', 'elevenlabs.speech', 'gemini.speech', 'gemini.text'}
        if not engines or any(e not in known for e in engines) or row.get('metadata', {}).get('cpu') != policy['latency_cpu'] or obs.get('phase') not in ('cold', 'warm'):
            gates['latency'] = check('not_tested', reason='unknown engine, phase or recorded environment')
        else:
            maximum = policy['cloud_seconds_max'] if any(e not in fixed.LOCAL for e in engines) else policy['local_first_use_seconds_max'] if obs['phase'] == 'cold' else policy['local_warm_seconds_max']
            gates['latency'] = limit(obs.get('total_seconds'), maximum)
    return {'id': row['id'], 'case': row['case'], 'speech': row.get('speech'), 'text': row.get('text'),
            'profile': row.get('profile'), 'status': combine(gates), 'gates': gates,
            'language_adherence': 'not_tested: no validated detector', 'native_os': 'not_tested', 'who': 'not_tested'}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run():
    policy = json.loads(POLICY.read_text())
    source = REPORTS / '47-reference021.jsonl'
    rows = [json.loads(line) for line in source.read_text().splitlines()]
    manifest = json.loads((fixed.CORPUS / 'manifest.json').read_text())
    cleanup = json.loads((fixed.CORPUS / 'cleanup.json').read_text())
    cases = {c['id']: c for c in manifest['cases'] + cleanup['cases']}
    frozen = json.loads((REPORTS / '47-reference021.provenance.json').read_text())
    if any(reference.get('corpus_version') != policy['corpus_version'] for reference in (manifest, cleanup, frozen)) or sha(fixed.CORPUS / 'manifest.json') != frozen['reference_manifest_sha256'] or sha(fixed.CORPUS / 'cleanup.json') != frozen['cleanup_sha256']:
        raise ValueError('policy requires the frozen 0.2.1 references')
    originals = {}
    for entry in frozen['original_observations']:
        path = REPORTS / entry['path']
        if sha(path) != entry['sha256']:
            raise ValueError('frozen observation hash changed: ' + entry['path'])
        originals.update({o['id']: o for o in map(json.loads, path.read_text().splitlines())})
    if len(rows) != 70 or len(originals) != 70 or {r['id']: r['observation'] for r in rows} != originals:
        raise ValueError('scored rows must preserve all 70 original observations')
    # Recompute fixed checks from the captured observations, never trust cached metrics.
    observations = {r['id']: r['observation'] for r in rows}
    selection = [{k: r[k] for k in ('id', 'case', 'speech', 'text', 'profile')} for r in rows]
    refreshed = fixed.report(selection, observations, manifest, cleanup, {})
    results = []
    provenance = {
        'policy_version': policy['policy_version'], 'policy_sha256': sha(POLICY),
        'policy_scoring_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'policy_scorer_sha256': sha(Path(__file__)), 'fixed_scorer_sha256': sha(ROOT / 'scripts/evaluate-acceptance.py'),
        'source_scored_rows_sha256': sha(source),
        'frozen_inference_provenance': frozen,
        'inference_calls_this_run': 0,
        'note': 'Scoring revision is the source base; scorer/policy hashes identify this delivery. Original local dirty 141d30f and clean cloud fdcc30b inference remain distinct; no new binary inference claim.'}
    for old, row in zip(rows, refreshed):
        row['metadata'] = old['metadata']
        result = evaluate(row, cases[row['case']], policy)
        result['provenance'] = provenance
        result['original_observation_source'] = old['metadata']['original_observation_source']
        result['observed_phase'] = old['observation'].get('phase')
        result['recorded_total_seconds'] = old['observation'].get('total_seconds')
        results.append(result)
    return policy, rows, results


def markdown(policy, rows, results):
    counts = dict(Counter(r['status'] for r in results))
    lines = ['# Issue 47 — fixed-corpus regression policy 1.0.0', '',
             f'Offline evaluation of 70 frozen observations (38 local, 32 cloud): **{counts.get("pass", 0)} pass / {counts.get("fail", 0)} fail / {counts.get("not_tested", 0)} not_tested**. Zero inference requests or downloads.', '',
             policy['basis'] + '. Gates are deliberately allowed to fail this baseline. Source references remain provisional; spliced/accelerated/repeated fixtures are controlled tests, not natural conversation or statistical production/language acceptance.', '',
             'Raw clean Danish/English/paused WER ≤0.10; mixed/accelerated WER ≤0.30. Zero declared fact/relation violations, zero punctuation-only word edits, exact email first/last lines. Non-speech requires empty/no-speech; all other inference errors fail. Raw failure remains visible regardless of cleanup. Missing reference/word timing stays not_tested. Structure retains captured production duration validation (+0.1 s) and full lexical clip anchors (±2 s); no acoustic alignment or speaker identity claim.', '',
             'Short total latency: local first-use ≤5 s, warm ≤2 s; any cloud stage ≤10 s. Applied to the explicit short case list in the policy, including fixed text; silence/noise and long/four-speaker fixtures excluded. Recorded Apple M3 Max release environment only. First-use includes loading with uncontrolled OS cache; cloud phase is process reuse, not server model load. No p95, SLO or model ranking.', '',
             '| Observation | Result | Failed / untested gates |', '|---|---|---|']
    for r in results:
        detail = ', '.join(f'{k}: {v["status"]}' for k, v in r['gates'].items() if v['status'] != 'pass') or 'all applicable gates passed'
        lines.append(f'| {r["id"]} | {r["status"]} | {detail} |')
    lines += ['', '## Recorded repeated short timings', '', '| Direct da-000 engine / phase | n | Median s | Range s |', '|---|---:|---:|---:|']
    for engine in ('local.parakeet','local.whisper','openai.speech','elevenlabs.speech','gemini.speech'):
        for phase in ('cold','warm'):
            values = [r['observation']['total_seconds'] for r in rows if r['speech'] == engine and not r['text'] and r['case'] == 'da-000' and r['observation']['phase'] == phase]
            if values:
                lines.append(f'| {engine} / {phase} | {len(values)} | {statistics.median(values):.3f} | {min(values):.3f}–{max(values):.3f} |')
    lines += ['', '## Provenance and limits', '',
              'JSONL retains frozen inference provenance and hashes separately from current policy/scorer revision and hashes. All earlier #47 artifacts and raw bytes remain unchanged. Local inference used dirty base 141d30f with an initial binary-hash gap; cloud used clean fdcc30b and its recorded signed binary hash. Current source was not used to rerun these outputs.', '',
              'The high-WER Whisper da-000 rows fail despite their passing narrow negation fact. Parakeet accelerated and mixed-spliced rows fail; captured Danish Whisper non-speech hallucinations and cleanup payment-intent/wrapper failures remain failures. Passing declared relations cannot establish unrestricted meaning preservation. Validated language detection, native OS insertion/UI, Who and exact acoustic alignment remain not_tested. Separate #48 native adapter evidence does not fill the native OS gap.', '',
              'Reproduce: `python3 scripts/evaluate-acceptance-policy.py --output gdd/reports/epic42/47-regression-policy-v1`.',
              'Normal CI: `python3 -m unittest discover -s scripts/tests -p \'test_acceptance*.py\'`. 44 tests passed, including 10 new offline policy regressions. No Cargo/product/runtime changes.', '']
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    policy, rows, results = run()
    args.output.with_suffix('.jsonl').write_text(''.join(json.dumps(r, ensure_ascii=False, allow_nan=False) + '\n' for r in results))
    args.output.with_suffix('.md').write_text(markdown(policy, rows, results))
    print(json.dumps(dict(Counter(r['status'] for r in results))))


if __name__ == '__main__':
    main()
