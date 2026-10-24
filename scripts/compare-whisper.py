#!/usr/bin/env python3
"""Developer-only local comparison. Requires already downloaded/verified models."""
import argparse
import json
import os
from pathlib import Path
import statistics
import subprocess

ROOT = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--cpu-only', action='store_true')
args = parser.parse_args()
family = ROOT / 'target/experiments/model-candidates/whisper-family'
models = {
    'tiny': family / 'ggml-tiny.bin',
    'base': ROOT / 'target/probe-models/ggml-base.bin',
    'small': ROOT / 'target/experiments/model-candidates/whisper-small/ggml-small.bin',
    'medium': family / 'ggml-medium.bin',
    'large-v3': family / 'ggml-large-v3.bin',
    'large-v3-turbo': family / 'ggml-large-v3-turbo.bin',
}
for model in models.values():
    if not model.is_file():
        parser.error(f'Missing verified model: {model}')
results = ROOT / 'target/experiments/whisper-family-results'
results.mkdir(exist_ok=True)
for model, path in models.items():
    for device in (['cpu'] if args.cpu_only else ['cpu', 'metal']):
        name = f'{model}-{device}'
        command = [str(ROOT/'target/release/examples/whisper_bench'), str(path),
                   str(ROOT/'Samples/da_test.wav'), '6']
        with (results/f'{name}.jsonl').open('w') as output, (results/f'{name}.log').open('w') as log:
            subprocess.run(command, cwd=ROOT, env={**os.environ, 'VARA_PROBE_GPU': '1' if device == 'metal' else '0'},
                           stdout=output, stderr=log, timeout=300, check=True)
        if device == 'metal' and 'using Metal backend' not in (results/f'{name}.log').read_text():
            raise RuntimeError(f'{name}: Metal was requested but not confirmed by native backend logs')
        rows = [json.loads(line) for line in (results/f'{name}.jsonl').read_text().splitlines()]
        median = statistics.median(row['elapsed_ms'] for row in rows[1:])
        print(f'{name}: warm median {median:.1f} ms; load {rows[0]["load_ms"]:.1f} ms', flush=True)
