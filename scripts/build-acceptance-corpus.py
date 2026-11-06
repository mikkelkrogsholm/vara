#!/usr/bin/env python3
"""Explicit, bounded public download; never uploads or reads user recordings."""
import hashlib
import json
from pathlib import Path
import random
import struct
import subprocess
import urllib.parse
import urllib.request
import wave

ROOT = Path(__file__).resolve().parents[1] / 'Samples/acceptance'
REVISION = '70bb2e84b976b7e960aa89f1c648e09c59f894dd'
DA_ROWS = [0, 1, 2, 3, 4, 6, 7, 9, 10, 12, 23, 25]
RATE = 16000


def dump(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def fetch(url):
    with urllib.request.urlopen(url, timeout=30) as response:
        data = response.read(10_000_001)
    if len(data) > 10_000_000:
        raise ValueError('Narrow asset size limit exceeded')
    return data


def pcm16(data):
    """Convert the server's little-endian float WAV to deterministic signed PCM16."""
    if data[:4] != b'RIFF' or data[8:12] != b'WAVE':
        raise ValueError('Not a RIFF WAV')
    chunks, offset = {}, 12
    while offset + 8 <= len(data):
        tag, size = struct.unpack_from('<4sI', data, offset)
        chunks[tag] = data[offset + 8:offset + 8 + size]
        offset += 8 + size + size % 2
    fmt, channels, rate = struct.unpack_from('<HHI', chunks[b'fmt '])
    bits = struct.unpack_from('<H', chunks[b'fmt '], 14)[0]
    if (fmt, channels, rate, bits) != (3, 1, RATE, 32):
        raise ValueError('Unexpected FLEURS asset format')
    samples = struct.iter_unpack('<f', chunks[b'data'])
    return b''.join(struct.pack('<h', max(-32768, min(32767, round(s[0] * 32768)))) for s in samples)


def save_audio(case_id, pcm):
    path = ROOT / 'audio' / (case_id + '.wav')
    with wave.open(str(path), 'wb') as out:
        out.setparams((1, 2, RATE, 0, 'NONE', 'not compressed'))
        out.writeframes(pcm)
    return {'path': str(path.relative_to(ROOT)), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'duration_seconds': len(pcm) / (RATE * 2)}


def rights(generated=False):
    return {'license': 'CC0-1.0' if generated else 'CC-BY-4.0',
            'redistribution': True, 'cloud_upload_permitted': True,
            'permission_basis': 'Project-authored non-speech signal' if generated else 'Public FLEURS CC-BY-4.0; attribution in README.md',
            'automatic_upload': False}


def review(text, kind='source_reference'):
    return {'text': text, 'origin': kind, 'human_listening_review': 'pending',
            'reviewer': None, 'reviewed_at': None}


def main():
    ROOT.joinpath('audio').mkdir(parents=True, exist_ok=True)
    cases, buffers, texts = [], {}, {}
    source_records = []
    for config, indices in [('da_dk', DA_ROWS), ('en_us', [0])]:
        query = urllib.parse.urlencode({'dataset': 'google/fleurs', 'config': config,
                                      'split': 'validation', 'offset': 0, 'length': max(indices) + 1})
        rows = json.loads(fetch('https://datasets-server.huggingface.co/rows?' + query))['rows']
        for idx in indices:
            row = rows[idx]['row']
            url = row['audio'][0]['src']
            if '/--/' + REVISION + '/--/' not in url:
                raise ValueError('Source revision changed; review corpus update explicitly')
            case_id = ('da' if config == 'da_dk' else 'en') + f'-{idx:03d}'
            source = fetch(url)
            pcm = pcm16(source)
            if len(pcm) // 2 != row['num_samples']:
                raise ValueError('FLEURS source sample count mismatch')
            buffers[case_id], texts[case_id] = pcm, row['raw_transcription']
            record = {'dataset': 'google/fleurs', 'revision': REVISION, 'config': config,
                      'split': 'validation', 'row_index': idx, 'utterance_id': row['id'],
                      'source_filename': Path(row['path']).name,
                      'raw_transcription': row['raw_transcription'], 'normalized_source_transcription': row['transcription'],
                      'source_num_samples': row['num_samples'],
                      'source_sha256': hashlib.sha256(source).hexdigest(),
                      'source_url': url.split('?')[0], 'acquired_on': '2026-11-06',
                      'transformation': 'Float32 WAV converted to PCM16: round(sample*32768), clamp signed16',
                      'speaker_identity': 'not supplied by row metadata'}
            source_records.append(record)
            cases.append({'id': case_id, **save_audio(case_id, pcm), 'language': config[:2],
                          'scenario': ['natural_read_speech'], 'fixture_kind': 'source_speech',
                          'provenance': record, 'rights': rights(), 'reference': review(texts[case_id]),
                          'annotation_extent': 'whole_clip_source_reference_pending_listening',
                          'critical_facts': [], 'allowed_orthographic_variants': []})
    def derived(case_id, pcm, sources, scenario, text, language='da', **extra):
        cases.append({'id': case_id, **save_audio(case_id, pcm), 'language': language,
                      'scenario': scenario, 'fixture_kind': 'structural_derivation',
                      'provenance': {'source_cases': sources, 'transformation': extra.pop('transformation')},
                      'rights': rights(not sources), 'reference': review(text, 'derived_source_reference' if sources else 'generated_non_speech'),
                      'annotation_extent': 'whole_clip_pending_listening',
                      'critical_facts': [], 'allowed_orthographic_variants': [], **extra})
    derived('da-paused', b'\0' * RATE * 2 + buffers['da-000'] + b'\0' * RATE * 4,
            ['da-000'], ['leading_trailing_pause'], texts['da-000'], transformation='1s silence + source + 2s silence')
    # Sample skipping tests faster delivery structurally; pitch rises too.
    accelerated = b''.join(buffers['da-001'][i:i+2] for i in range(0, len(buffers['da-001']), 4))
    derived('da-fast-structural', accelerated, ['da-001'], ['accelerated_speech'], texts['da-001'], transformation='Every second PCM sample retained at 16kHz: 2x speed and pitch; not natural fast speech')
    derived('mixed-spliced', buffers['da-009'] + b'\0' * RATE + buffers['en-000'], ['da-009', 'en-000'],
            ['mixed_language_splice'], texts['da-009'] + ' ' + texts['en-000'], language='da+en', transformation='Danish source + 0.5s silence + English source; not natural code switching')
    derived('silence', b'\0' * RATE * 2 * 3, [], ['silence', 'no_hallucination'], '', language='none', transformation='3s digital silence')
    rng = random.Random(42)
    noise = b''.join(struct.pack('<h', rng.randrange(-600, 601)) for _ in range(RATE * 3))
    derived('noise', noise, [], ['noise', 'no_hallucination'], '', language='none', transformation='3s seeded integer white noise, seed42, amplitude600')
    # Source intervals cross nominal 30-second boundaries, including five minutes.
    long_pcm = bytearray(RATE * 2 * 321)
    anchors = []
    for boundary in [30, 60, 150, 300]:
        start = boundary - 2
        pcm = buffers['da-000']
        long_pcm[start * RATE * 2:start * RATE * 2 + len(pcm)] = pcm
        anchors.append({'start_seconds': start, 'end_seconds': start + len(pcm)/(RATE*2),
                        'source_case': 'da-000', 'text': texts['da-000'], 'review_status': 'pending', 'reviewer': None, 'reviewed_at': None})
    derived('long-boundaries', bytes(long_pcm), ['da-000'], ['chunk_boundary_30s', 'over_five_minutes'], None,
            transformation='321s silence bed with full da-000 at28,58,148,298s; repeated structural input',
            annotation_extent='listed_source_placement_anchors_only_pending_listening', anchors=anchors,
            nominal_chunk_seconds=30)
    turns, parts, cursor, speaker_ids = [], [], 0.0, []
    libri_revision = '71cacbfb7e2354c4226d01e70d77d5fca3d04ba1'
    for offset in [0, 100, 200, 300]:
        query = urllib.parse.urlencode({'dataset': 'openslr/librispeech_asr', 'config': 'clean',
                                      'split': 'test', 'offset': offset, 'length': 1})
        row = json.loads(fetch('https://datasets-server.huggingface.co/rows?' + query))['rows'][0]['row']
        url = row['audio'][0]['src']
        if '/--/' + libri_revision + '/--/' not in url:
            raise ValueError('LibriSpeech revision changed; review explicitly')
        source = fetch(url)
        pcm = subprocess.run(['ffmpeg', '-v', 'error', '-i', 'pipe:0', '-f', 's16le',
                              '-ac', '1', '-ar', str(RATE), 'pipe:1'], input=source,
                             stdout=subprocess.PIPE, check=True, timeout=30).stdout
        source_id = 'libri-' + row['id']
        record = {'dataset': 'openslr/librispeech_asr', 'revision': libri_revision,
                  'config': 'clean', 'split': 'test', 'row_index': offset,
                  'utterance_id': row['id'], 'speaker_id': str(row['speaker_id']),
                  'source_transcription': row['text'],
                  'source_sha256': hashlib.sha256(source).hexdigest(),
                  'source_url': url.split('?')[0], 'acquired_on': '2026-11-06',
                  'transformation': 'FLAC decoded to mono16kHz PCM16 by ffmpeg'}
        source_records.append(record)
        duration = len(pcm)/(RATE*2)
        turns.append({'start_seconds': cursor, 'end_seconds': cursor + duration,
                      'source_label': source_id, 'speaker_id': str(row['speaker_id']),
                      'provenance': record, 'text': row['text'], 'review_status': 'pending', 'reviewer': None, 'reviewed_at': None})
        speaker_ids.append(row['speaker_id'])
        parts.extend([pcm, b'\0' * RATE])
        cursor += duration + 0.5
    if len(set(speaker_ids)) != 4:
        raise ValueError('Expected four metadata-distinct speakers')
    derived('four-speakers', b''.join(parts), [], ['four_speakers_structural'], None, language='en',
            transformation='Four LibriSpeech speakers separated by0.5s silence; not natural conversation',
            annotation_extent='listed_source_turns_only_pending_listening', turns=turns,
            distinct_speaker_count=4)
    cases[-1]['rights'] = rights()
    cases[-1]['rights']['permission_basis'] = 'Public LibriSpeech CC-BY-4.0; attribution in README.md'
    cases[-1]['reference']['origin'] = 'derived_source_reference'
    facts = {
        'da-000': [('negation', 'ikke', ['ikke'])],
        'da-001': [('percentage', '80%', ['firs procent', '80 procent', '80 %', '80%'])],
        'da-007': [('score', '21–20', ['21 til 20', 'enogtyve til tyve']), ('count', '15', ['15', 'femten'])],
        'da-010': [('negation', 'ikke', ['ikke'])],
        'da-012': [('date', '1789-10-06', ['6. oktober 1789', 'sjette oktober sytten hundrede niogfirs'])],
        'da-023': [('negation', 'ikke', ['ikke']), ('name', 'Number One', ['Number One'])],
        'da-025': [('height', '4892m', ['4.892 m', '4892 meter', 'fire tusind otte hundrede tooghalvfems meter'])]}
    by_id = {case['id']: case for case in cases}
    for case_id, values in facts.items():
        by_id[case_id]['critical_facts'] = [{'kind': kind, 'value': value, 'accepted_surfaces': variants} for kind,value,variants in values]
    for case in cases:
        if case['fixture_kind'] == 'structural_derivation':
            case['critical_facts'] = [fact for source in case['provenance']['source_cases'] for fact in by_id[source]['critical_facts']]
    dump(ROOT / 'sources.json', source_records)
    dump(ROOT / 'manifest.json', {'schema_version': 1, 'corpus_version': '0.1.0',
                                 'supporting_files': [{'path': name, 'sha256': hashlib.sha256((ROOT / name).read_bytes()).hexdigest()} for name in ['sources.json', 'cleanup.json']],
                                 'status': 'prepared_pending_human_review',
                                 'critical_fact_annotation_status': 'highlighted_facts_pending_review_not_exhaustive', 'cases': cases})
    print(f'Prepared {len(cases)} cases; no human listening review claimed.')


if __name__ == '__main__':
    main()
