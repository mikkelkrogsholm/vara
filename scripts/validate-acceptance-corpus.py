#!/usr/bin/env python3
"""Offline manifest, asset and annotation validation. No provider calls."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import sys
import wave

DEFAULT_ROOT = Path(__file__).resolve().parents[1] / 'Samples/acceptance'


def validate(root, require_reviewed=False):
    root = Path(root).resolve()
    errors = []
    def check(ok, message):
        if not ok:
            errors.append(message)
    manifest = json.loads((root / 'manifest.json').read_text())
    check(manifest.get('schema_version') == 1, 'Unsupported manifest schema')
    check({f['path'] for f in manifest['supporting_files']} == {'sources.json', 'cleanup.json'}, 'Missing supporting file declarations')
    for supporting in manifest['supporting_files']:
        path = (root / supporting['path']).resolve()
        if not path.is_relative_to(root) or Path(supporting['path']).is_absolute():
            errors.append('Supporting file path escapes corpus')
            continue
        check(path.is_file(), f'Missing supporting file: {supporting["path"]}')
        if path.is_file():
            check(hashlib.sha256(path.read_bytes()).hexdigest() == supporting['sha256'], f'{supporting["path"]}: hash mismatch')
    cases = manifest['cases']
    ids = [case['id'] for case in cases]
    check(len(ids) == len(set(ids)), 'Duplicate case IDs')
    known = set(ids)
    paths = []
    for case in cases:
        cid = case['id']
        path = (root / case['path']).resolve()
        check(path.is_relative_to(root) and not Path(case['path']).is_absolute(), f'{cid}: path escapes corpus')
        if not path.is_relative_to(root):
            continue
        paths.append(path)
        if not path.is_file():
            errors.append(f'{cid}: missing audio {case["path"]}')
            continue
        check(hashlib.sha256(path.read_bytes()).hexdigest() == case['sha256'], f'{cid}: hash mismatch')
        try:
            with wave.open(str(path)) as audio:
                check((audio.getnchannels(), audio.getsampwidth(), audio.getframerate()) == (1, 2, 16000), f'{cid}: invalid audio format')
                duration = audio.getnframes() / audio.getframerate()
        except (wave.Error, EOFError) as error:
            errors.append(f'{cid}: invalid WAV: {error}')
            continue
        check(isinstance(case['duration_seconds'], (int, float)) and math.isfinite(case['duration_seconds'])
              and abs(duration - case['duration_seconds']) <= 1/16000, f'{cid}: duration mismatch')
        check(bool(case['language']) and bool(case['scenario']), f'{cid}: missing language/scenario')
        rights = case['rights']
        check(rights.get('license') in ('CC-BY-4.0', 'CC0-1.0') and rights.get('redistribution') is True
              and isinstance(rights.get('cloud_upload_permitted'), bool) and bool(rights.get('permission_basis'))
              and rights.get('automatic_upload') is False, f'{cid}: incomplete rights/permission')
        ref = case['reference']
        check(ref.get('human_listening_review') in ('pending', 'reviewed'), f'{cid}: invalid review status')
        if ref.get('human_listening_review') == 'reviewed':
            check(bool(ref.get('reviewer')) and bool(ref.get('reviewed_at')), f'{cid}: review lacks attribution')
        if require_reviewed:
            check(ref.get('human_listening_review') == 'reviewed', f'{cid}: listening review pending')
        text = ref.get('text')
        check(isinstance(text, str) or (text is None and ('anchors' in case or 'turns' in case)), f'{cid}: invalid reference extent')
        check(bool(case['annotation_extent']), f'{cid}: missing annotation extent')
        for source in case['provenance'].get('source_cases', []):
            check(source in known, f'{cid}: unknown source case {source}')
        for fact in case['critical_facts']:
            check(bool(fact.get('kind')) and bool(fact.get('value')) and bool(fact.get('accepted_surfaces')), f'{cid}: invalid critical fact')
        for group in ('anchors', 'turns'):
            previous_end = 0
            for annotation in case.get(group, []):
                start, end = annotation['start_seconds'], annotation['end_seconds']
                valid = all(isinstance(n, (float, int)) and math.isfinite(n) for n in [start, end])
                check(valid and 0 <= start < end <= duration and start >= previous_end, f'{cid}: invalid/overlapping {group} interval')
                previous_end = end
                check(bool(annotation['text']), f'{cid}: empty annotation')
                check(annotation.get('review_status') in ('pending', 'reviewed'), f'{cid}: invalid annotation review')
                if annotation.get('review_status') == 'reviewed':
                    check(bool(annotation.get('reviewer')) and bool(annotation.get('reviewed_at')), f'{cid}: annotation review lacks attribution')
                if require_reviewed:
                    check(annotation.get('review_status') == 'reviewed', f'{cid}: annotation review pending')
                if 'source_case' in annotation:
                    check(annotation['source_case'] in known, f'{cid}: unknown annotation source')
        if 'distinct_speaker_count' in case:
            speakers = {turn.get('speaker_id') for turn in case.get('turns', [])}
            check(None not in speakers and len(speakers) == case['distinct_speaker_count'] == 4, f'{cid}: four speaker IDs required')
            check(all(turn.get('provenance', {}).get('speaker_id') == turn.get('speaker_id') for turn in case.get('turns', [])), f'{cid}: speaker provenance mismatch')
        if 'over_five_minutes' in case['scenario']:
            check(duration > 300, f'{cid}: long fixture must exceed five minutes')
            for boundary in [30, 60, 150, 300]:
                check(any(anchor['start_seconds'] < boundary < anchor['end_seconds'] for anchor in case.get('anchors', [])), f'{cid}: no source placement across {boundary}s')
    check(len(paths) == len(set(paths)), 'Duplicate asset paths')
    actual_paths = {p.resolve() for p in root.joinpath('audio').glob('*.wav')}
    check(set(paths) == actual_paths, 'Unlisted WAV assets')
    cleanup = json.loads((root / 'cleanup.json').read_text())
    profiles = {'punctuation_only', 'general_prompt', 'app_specific_prompt'}
    check(set(cleanup['profiles']) == profiles, 'Invalid cleanup profiles')
    cleanup_ids = [case['id'] for case in cleanup['cases']]
    check(len(set(cleanup_ids)) == len(cleanup_ids), 'Duplicate cleanup IDs')
    for case in cleanup['cases']:
        check(bool(case['input']) and set(case['expected']) == profiles
              and all(isinstance(text, str) and text for text in case['expected'].values())
              and bool(case['critical_facts']) and isinstance(case['allowed_orthographic_variants'], list), f'{case["id"]}: incomplete cleanup expectation')
    if cleanup['review_status'] == 'human_reviewed':
        check(bool(cleanup.get('reviewer')) and bool(cleanup.get('reviewed_at')), 'Cleanup review lacks attribution')
    if require_reviewed:
        check(cleanup['review_status'] == 'human_reviewed', 'Cleanup expectations human review pending')
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=DEFAULT_ROOT)
    parser.add_argument('--require-reviewed', action='store_true', help='Fail until human review is complete')
    args = parser.parse_args()
    try:
        errors = validate(args.root, args.require_reviewed)
    except (OSError, ValueError, KeyError, TypeError) as error:
        errors = [f'Invalid corpus: {error}']
    if errors:
        print('\n'.join(errors), file=sys.stderr)
        return 1
    print('Corpus structure, hashes, durations, rights and annotations valid; listening-review gate ' + ('passed.' if args.require_reviewed else 'not requested.'))
    return 0


if __name__ == '__main__':
    sys.exit(main())
