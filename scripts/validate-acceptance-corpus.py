#!/usr/bin/env python3
"""Offline manifest, asset and annotation validation. No provider calls."""
import argparse
import hashlib
import json
import math
import random
import re
import struct
from pathlib import Path
import sys
import wave

DEFAULT_ROOT = Path(__file__).resolve().parents[1] / 'Samples/acceptance'


def validate(root):
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
                check(annotation.get('alignment') == 'deterministic_clip_placement', f'{cid}: invalid alignment basis')
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
    errors.extend(reference_checks(root, manifest, json.loads((root / 'sources.json').read_text()), cleanup))
    return errors


def reference_checks(root, manifest, sources, cleanup):
    errors = []
    def check(ok, message):
        if not ok:
            errors.append(message)
    def key(record):
        return record['dataset'], record['revision'], str(record['utterance_id'])
    canonical = {key(record): record for record in sources}
    check(len(canonical) == len(sources), 'Duplicate source records')
    for record in sources:
        check(bool(re.fullmatch(r'[a-f0-9]{40}', record['revision']))
              and bool(re.fullmatch(r'[a-f0-9]{64}', record['source_sha256']))
              and '/--/' + record['revision'] + '/--/' in record['source_url'], 'Invalid pinned source provenance')
    cases = {case['id']: case for case in manifest['cases']}
    pcm = {}
    for cid, case in cases.items():
        path = root / case['path']
        if not path.resolve().is_relative_to(root) or not path.is_file():
            continue
        try:
            with wave.open(str(path), 'rb') as audio:
                pcm[cid] = audio.readframes(audio.getnframes())
        except (wave.Error, EOFError):
            continue
    for cid, case in cases.items():
        reference = case['reference']
        origin = reference['origin']
        check(origin in ['dataset_provided', 'derived_dataset_provided', 'generated_non_speech'], f'{cid}: invalid reference origin')
        check(reference.get('word_alignment') == 'not_available', f'{cid}: unsupported word alignment claim')
        expected_status = 'deterministic_generated_expectation' if origin == 'generated_non_speech' else 'provisional_source_reference'
        check(reference.get('status') == expected_status, f'{cid}: invalid reference status')
        if case['fixture_kind'] == 'source_speech':
            source = canonical.get(key(case['provenance']))
            check(source is not None and source == case['provenance'], f'{cid}: missing or changed source reference')
            check(origin == 'dataset_provided' and source is not None and reference['text'] == source.get('raw_transcription'), f'{cid}: source transcript mismatch')
            if source and cid in pcm:
                check(len(pcm[cid]) // 2 == source['source_num_samples'], f'{cid}: source sample count mismatch')
        elif origin == 'generated_non_speech':
            check(reference['text'] == '', f'{cid}: generated non-speech reference must be empty')
        elif reference['text'] is not None:
            source_ids = case['provenance']['source_cases']
            check(bool(source_ids) and all(source in cases for source in source_ids), f'{cid}: missing derived sources')
            if all(source in cases for source in source_ids):
                check(reference['text'] == ' '.join(cases[source]['reference']['text'] for source in source_ids), f'{cid}: derived transcript mismatch')
        content = reference['text'] or ' '.join(a['text'] for group in ['anchors', 'turns'] for a in case.get(group, []))
        for fact in case['critical_facts']:
            check(has_surface(content, fact.get('accepted_surfaces', [])), f'{cid}: critical fact absent from reference: {fact.get("value")}')
        for annotation in case.get('anchors', []):
            source = annotation['source_case']
            if source in cases:
                check(annotation['text'] == cases[source]['reference']['text'], f'{cid}: anchor transcript mismatch')
            if source in pcm and cid in pcm:
                start, end = (round(annotation[k] * 16000) * 2 for k in ['start_seconds', 'end_seconds'])
                check(pcm[cid][start:end] == pcm[source], f'{cid}: source placement PCM mismatch')
        for turn in case.get('turns', []):
            record = canonical.get(key(turn['provenance']))
            check(record is not None and record == turn['provenance'], f'{cid}: missing or changed turn source')
            check(record is not None and turn['text'] == record.get('source_transcription'), f'{cid}: turn transcript mismatch')
            check(record is not None and turn['speaker_id'] == record.get('speaker_id'), f'{cid}: speaker metadata mismatch')
            if record and cid in pcm:
                start, end = (round(turn[k] * 16000) * 2 for k in ['start_seconds', 'end_seconds'])
                check(hashlib.sha256(pcm[cid][start:end]).hexdigest() == record['pcm_sha256'], f'{cid}: turn placement PCM mismatch')
    # Byte-for-byte recipes verify structural claims without word-level alignment.
    if {'da-000', 'da-001', 'da-009', 'en-000'} <= pcm.keys():
        expected = {'da-paused': b'\0' * 32000 + pcm['da-000'] + b'\0' * 64000,
                    'da-fast-structural': b''.join(pcm['da-001'][i:i+2] for i in range(0, len(pcm['da-001']), 4)),
                    'mixed-spliced': pcm['da-009'] + b'\0' * 16000 + pcm['en-000'],
                    'silence': b'\0' * 96000}
        rng = random.Random(42)
        expected['noise'] = b''.join(struct.pack('<h', rng.randrange(-600, 601)) for _ in range(48000))
        long_pcm = bytearray(321 * 32000)
        for start in [28, 58, 148, 298]:
            long_pcm[start * 32000:start * 32000 + len(pcm['da-000'])] = pcm['da-000']
        expected['long-boundaries'] = bytes(long_pcm)
        for cid, data in expected.items():
            check(pcm.get(cid) == data, f'{cid}: deterministic recipe mismatch')
    if 'four-speakers' in pcm:
        cursor = 0
        for turn in cases['four-speakers']['turns']:
            start, end = (round(turn[k] * 16000) * 2 for k in ['start_seconds', 'end_seconds'])
            check(start == cursor and pcm['four-speakers'][end:end+16000] == b'\0' * 16000, 'four-speakers: turn/gap recipe mismatch')
            cursor = end + 16000
        check(cursor == len(pcm['four-speakers']), 'four-speakers: unannotated content')
    check(cleanup.get('reference_status') == 'versioned_authored_expectations', 'Invalid cleanup reference origin')
    for case in cleanup['cases']:
        for label, text in {'input': case['input'], **case['expected']}.items():
            for fact in case['critical_facts']:
                check(has_surface(text, fact.get('accepted_surfaces', [])), f'{case["id"]}/{label}: critical fact absent: {fact["value"]}')
        check(re.findall(r'\w+', case['input'].casefold()) == re.findall(r'\w+', case['expected']['punctuation_only'].casefold()), f'{case["id"]}: punctuation-only changed words')
        check(case['expected']['app_specific_prompt'] == 'Hej,\n\n' + case['expected']['general_prompt'] + '\n\nVenlig hilsen', f'{case["id"]}: email framing mismatch')
    return errors


def has_surface(text, surfaces):
    text = ' '.join(text.casefold().split())
    return any(re.search(r'(?<!\w)' + re.escape(' '.join(surface.casefold().split())) + r'(?!\w)', text) for surface in surfaces)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=DEFAULT_ROOT)
    args = parser.parse_args()
    try:
        errors = validate(args.root)
    except (OSError, ValueError, KeyError, TypeError) as error:
        errors = [f'Invalid corpus: {error}']
    if errors:
        print('\n'.join(errors), file=sys.stderr)
        return 1
    print('Corpus assets, source references, deterministic placements, speakers and cleanup assertions valid. Source transcripts remain provisional; exact word alignment is not claimed.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
