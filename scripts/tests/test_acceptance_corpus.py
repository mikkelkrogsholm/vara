import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'validate-acceptance-corpus.py'
SPEC = importlib.util.spec_from_file_location('corpus_validator', SCRIPT)
validator = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(validator)


class AcceptanceCorpusTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / 'corpus'
        shutil.copytree(validator.DEFAULT_ROOT, self.root)
        self.manifest_path = self.root / 'manifest.json'
        self.manifest = json.loads(self.manifest_path.read_text())

    def save_manifest(self):
        self.manifest_path.write_text(json.dumps(self.manifest))

    def test_checked_in_corpus_valid(self):
        self.assertEqual(validator.validate(self.root), [])
        self.assertEqual(len(self.manifest['cases']), 20)

    def test_missing_asset(self):
        (self.root / self.manifest['cases'][0]['path']).unlink()
        self.assertTrue(any('missing audio' in error for error in validator.validate(self.root)))

    def test_changed_audio_hash(self):
        path = self.root / self.manifest['cases'][0]['path']
        data = bytearray(path.read_bytes())
        data[-1] ^= 1
        path.write_bytes(data)
        self.assertTrue(any('hash mismatch' in error for error in validator.validate(self.root)))

    def test_changed_cleanup_hash(self):
        with (self.root / 'cleanup.json').open('a') as out:
            out.write('\n')
        self.assertIn('cleanup.json: hash mismatch', validator.validate(self.root))

    def test_annotation_outside_recording(self):
        case = next(c for c in self.manifest['cases'] if 'anchors' in c)
        case['anchors'][0]['end_seconds'] = case['duration_seconds'] + 1
        self.save_manifest()
        self.assertTrue(any('invalid/overlapping anchors interval' in error for error in validator.validate(self.root)))

    def test_annotation_overlap(self):
        case = next(c for c in self.manifest['cases'] if 'turns' in c)
        case['turns'][1]['start_seconds'] = 0
        self.save_manifest()
        self.assertTrue(any('invalid/overlapping turns interval' in error for error in validator.validate(self.root)))

    def test_four_speakers_require_metadata_distinct_ids(self):
        case = next(c for c in self.manifest['cases'] if 'turns' in c)
        case['turns'][1]['speaker_id'] = case['turns'][0]['speaker_id']
        self.save_manifest()
        errors = validator.validate(self.root)
        self.assertTrue(any('four speaker IDs required' in error for error in errors))
        self.assertTrue(any('speaker provenance mismatch' in error for error in errors))

    def test_claimed_annotation_review_requires_attribution(self):
        case = next(c for c in self.manifest['cases'] if 'anchors' in c)
        case['anchors'][0]['review_status'] = 'reviewed'
        self.save_manifest()
        self.assertTrue(any('annotation review lacks attribution' in error for error in validator.validate(self.root)))

    def test_claimed_case_review_requires_attribution(self):
        self.manifest['cases'][0]['reference']['human_listening_review'] = 'reviewed'
        self.save_manifest()
        self.assertTrue(any('review lacks attribution' in error for error in validator.validate(self.root)))

    def test_rejects_outside_path(self):
        self.manifest['cases'][0]['path'] = '../private.wav'
        self.save_manifest()
        self.assertTrue(any('path escapes corpus' in error for error in validator.validate(self.root)))

    def test_human_review_gate_fails_for_prepared_corpus(self):
        errors = validator.validate(self.root, require_reviewed=True)
        self.assertTrue(any('listening review pending' in error for error in errors))
        self.assertIn('Cleanup expectations human review pending', errors)


if __name__ == '__main__':
    unittest.main()
