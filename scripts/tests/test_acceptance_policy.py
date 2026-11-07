import copy
import importlib.util
import json
from pathlib import Path
import unittest
from unittest import mock
import tempfile

SCRIPT = Path(__file__).resolve().parents[1] / 'evaluate-acceptance-policy.py'
SPEC = importlib.util.spec_from_file_location('policy_scoring', SCRIPT)
scoring = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(scoring)


class PolicyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.policy, cls.rows, cls.results = scoring.run()
        cls.manifest = json.loads((scoring.fixed.CORPUS / 'manifest.json').read_text())
        cls.cleanup = json.loads((scoring.fixed.CORPUS / 'cleanup.json').read_text())
        cls.cases = {c['id']: c for c in cls.manifest['cases'] + cls.cleanup['cases']}
        cls.by_id = {r['id']: r for r in cls.results}

    def evaluate(self, row, case=None):
        return scoring.evaluate(row, case or self.cases[row['case']], self.policy)

    def test_actual_whisper_fails_despite_passed_negation(self):
        row = next(r for r in self.rows if r['id'] == 'whisper/da-000/direct/0')
        self.assertEqual(row['status'], 'pass')  # frozen narrow baseline
        self.assertTrue(all(f['pass'] for f in row['asr']['critical_facts']))
        result = self.evaluate(row)
        self.assertEqual(result['gates']['raw_quality']['status'], 'fail')
        self.assertAlmostEqual(result['gates']['raw_quality']['value'], 17 / 19)
        self.assertEqual(result['status'], 'fail')
        self.assertEqual(self.by_id['parakeet/da-fast-structural/direct/0']['status'], 'fail')

    def test_wer_and_latency_boundaries_inclusive(self):
        row = copy.deepcopy(self.rows[0])
        for value, status in ((0.1, 'pass'), (0.100001, 'fail')):
            row['asr']['wer'] = value
            self.assertEqual(self.evaluate(row)['gates']['raw_quality']['status'], status)
        for maximum in (0.3, 2.0, 5.0, 10.0):
            self.assertEqual(scoring.limit(maximum, maximum)['status'], 'pass')
            self.assertEqual(scoring.limit(maximum + 0.000001, maximum)['status'], 'fail')

    def test_empty_no_speech_and_phantom(self):
        row = copy.deepcopy(next(r for r in self.rows if r['id'] == 'parakeet/silence/direct/0'))
        for raw, error, status in (('', None, 'pass'), (None, 'engine.no_speech', 'pass'),
                                   (None, 'speech.file_empty', 'pass'), ('hej', None, 'fail'),
                                   ('...', None, 'fail'), ('hej', 'engine.no_speech', 'fail'),
                                   ('', 'provider.http', 'fail')):
            row['observation'].update(raw=raw, error=error)
            self.assertEqual(self.evaluate(row)['status'], status)
        self.assertEqual(self.by_id['whisper-explicit-da/silence/direct/0']['status'], 'fail')

    def test_missing_reference_or_word_timing_never_passes(self):
        row = copy.deepcopy(self.rows[0])
        case = copy.deepcopy(self.cases[row['case']])
        case['reference']['text'] = None
        self.assertEqual(self.evaluate(row, case)['status'], 'not_tested')
        self.assertEqual(self.by_id['whisper/four-speakers/direct/0']['gates']['structure']['status'], 'not_tested')
        self.assertEqual(self.by_id['whisper/four-speakers/direct/0']['status'], 'not_tested')

    def test_captured_relation_and_wrapper_failures_retained(self):
        payment = self.by_id['gemini.text/text-names-amount-date/general_prompt/0']
        self.assertEqual(payment['gates']['cleanup_critical_facts']['status'], 'fail')
        self.assertEqual(payment['status'], 'fail')
        self.assertEqual(self.by_id['openai-openai/da-000/punctuation_only/0']['status'], 'fail')
        wrapper_failures = [r for r in self.results if r['gates'].get('email_wrapper', {}).get('status') == 'fail']
        self.assertTrue(wrapper_failures)
        self.assertTrue(all(r['status'] == 'fail' for r in wrapper_failures))

    def test_unknown_metadata_and_invalid_metrics_never_pass(self):
        row = copy.deepcopy(self.rows[0])
        for value in (None, float('nan'), float('inf'), -1, True, '0'):
            row['asr']['wer'] = value
            self.assertNotEqual(self.evaluate(row)['status'], 'pass')
        row = copy.deepcopy(self.rows[0])
        for field, value in (('phase', 'unknown'), ('total_seconds', float('nan'))):
            changed = copy.deepcopy(row)
            changed['observation'][field] = value
            self.assertNotEqual(self.evaluate(changed)['status'], 'pass')
        row['metadata']['cpu'] = 'unknown'
        self.assertNotEqual(self.evaluate(row)['status'], 'pass')
        row['metadata']['cpu'] = self.policy['latency_cpu']
        row['speech'] = 'unknown.engine'
        self.assertNotEqual(self.evaluate(row)['status'], 'pass')

    def test_good_cleanup_cannot_hide_failed_raw(self):
        row = copy.deepcopy(next(r for r in self.rows if r['id'] == 'parakeet-qwen/da-000/punctuation_only/0'))
        row['asr']['wer'] = 0.9
        self.assertEqual(row['cleanup']['status'], 'pass')
        self.assertEqual(self.evaluate(row)['status'], 'fail')

    def test_mismatched_policy_corpus_is_rejected(self):
        policy = dict(self.policy, corpus_version='unreviewed-future-reference')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'policy.json'
            path.write_text(json.dumps(policy))
            with mock.patch.object(scoring, 'POLICY', path):
                with self.assertRaisesRegex(ValueError, 'frozen 0.2.1 references'):
                    scoring.run()

    def test_missing_observation_is_not_tested(self):
        for case_id in ('silence', 'da-000', 'text-names-amount-date'):
            row = copy.deepcopy(next(r for r in self.rows if r['case'] == case_id))
            row.pop('observation')
            self.assertEqual(self.evaluate(row)['status'], 'not_tested')
            self.assertEqual(self.evaluate(row)['gates']['execution']['reason'], 'not observed')

    def test_frozen_source_hashes_and_counts(self):
        self.assertEqual(len(self.results), 70)
        provenance = self.results[0]['provenance']['frozen_inference_provenance']
        for source in provenance['original_observations']:
            self.assertEqual(scoring.sha(scoring.REPORTS / source['path']), source['sha256'])
        self.assertEqual(scoring.sha(scoring.REPORTS / '47-reference021.observations.jsonl'), provenance['combined_observations_sha256'])
        self.assertEqual(self.results[0]['provenance']['inference_calls_this_run'], 0)


if __name__ == '__main__':
    unittest.main()
