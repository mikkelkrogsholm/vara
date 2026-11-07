import importlib.util
import json
from pathlib import Path
import unittest

SCRIPT=Path(__file__).resolve().parents[1]/'evaluate-acceptance.py'
SPEC=importlib.util.spec_from_file_location('evaluation',SCRIPT)
evaluation=importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(evaluation)

class EvaluationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manifest=json.loads((evaluation.CORPUS/'manifest.json').read_text())
        cls.cleanup=json.loads((evaluation.CORPUS/'cleanup.json').read_text())
        cls.rows=evaluation.inventory(cls.manifest,cls.cleanup)

    def test_normalization_preserves_numbers_negation_unicode(self):
        self.assertEqual(evaluation.normalize('IKKE 15, ikke 50. Å!'),['ikke','15','ikke','50','å'])
        self.assertEqual(evaluation.normalize('A\u030a'),['å'])

    def test_inspectable_word_edits(self):
        result=evaluation.difference(['ikke','15','kroner'],['50','kroner','nu'])
        self.assertEqual(result['edit_count'],3)
        self.assertEqual({e['kind'] for e in result['edits']},{'substitution'})
        self.assertEqual(evaluation.difference(['ikke','15'],['15'])['edits'][0]['kind'],'omission')
        self.assertEqual(evaluation.difference(['15'],['15','nu'])['edits'][0]['kind'],'insertion')
        self.assertEqual(result['wer'],1)

    def test_non_speech_is_not_passed_for_nonempty_response(self):
        case=next(c for c in self.manifest['cases'] if c['id']=='silence')
        result=evaluation.score(case,'Det er ikke en del af det.')
        self.assertEqual(result['status'],'fail')
        self.assertIsNone(result['wer'])
        self.assertIn('hallucination_on_non_speech',result['failure_categories'])

    def test_fixed_critical_facts_and_number_alias(self):
        case=self.cleanup['cases'][0]
        good=case['expected']['general_prompt']
        self.assertEqual(evaluation.score(case,good,good)['status'],'pass')
        self.assertEqual(evaluation.score(case,good.replace('ikke ',''),good)['status'],'fail')
        self.assertEqual(evaluation.score(case,good.replace('1250','50'),good)['status'],'fail')
        self.assertEqual(evaluation.score(case,good.replace('1250 kroner','1.250 kroner'),good)['wer'],0)

    def test_punctuation_only_cannot_reorder_or_add_words(self):
        case=self.cleanup['cases'][0]
        expected=case['expected']['punctuation_only']
        self.assertEqual(evaluation.score(case,expected+' Tak.',expected,exact_words=True)['status'],'fail')
        self.assertEqual(evaluation.score(case,expected.upper(),expected)['status'],'pass')

    def test_payment_instruction_relation_from_captured_cloud_output(self):
        case=next(c for c in self.cleanup['cases'] if c['id']=='text-names-amount-date')
        expected=case['expected']['general_prompt']
        captured=evaluation.ROOT/'gdd/reports/epic42/47-cloud-live.observations.jsonl'
        actual=next(json.loads(line)['final'] for line in captured.read_text().splitlines()
                    if json.loads(line)['id']=='gemini.text/text-names-amount-date/general_prompt/0')
        result=evaluation.score(case,actual,expected)
        self.assertEqual(result['status'],'fail')
        self.assertFalse(next(f['pass'] for f in result['critical_facts'] if f['kind']=='payment_instruction'))
        for good in (expected, expected.replace('Betal ikke','Undlad at betale'),
                     expected.replace('1250 kroner','1.250 kroner'),
                     expected.replace('1250 kroner','1250 kr.')):
            with self.subTest(good=good):
                self.assertEqual(evaluation.score(case,good,expected)['status'],'pass')

    def test_payment_relation_rejects_changed_payer_intent_amount_or_negation(self):
        case=next(c for c in self.cleanup['cases'] if c['id']=='text-names-amount-date')
        expected=case['expected']['general_prompt']
        for replacement in ('FigCraft vil ikke betale', 'FigCraft må ikke betale',
                            'Vara betaler ikke', 'Betal', 'Undlad ikke at betale'):
            actual=expected.replace('Betal ikke',replacement)
            with self.subTest(replacement=replacement):
                self.assertEqual(evaluation.score(case,actual,expected)['status'],'fail')
        self.assertEqual(evaluation.score(case,expected.replace('1250','50'),expected)['status'],'fail')

    def test_fixed_clock_relation_accepts_captured_equivalent_orthography(self):
        case=next(c for c in self.cleanup['cases'] if c['id']=='text-similar-numbers')
        expected=case['expected']['general_prompt']
        captured=evaluation.ROOT/'gdd/reports/epic42/47-cloud-live.observations.jsonl'
        actual=next(json.loads(line)['final'] for line in captured.read_text().splitlines()
                    if json.loads(line)['id']=='gemini.text/text-similar-numbers/general_prompt/0')
        self.assertEqual(evaluation.score(case,actual,expected)['status'],'pass')
        for spelling in ('klokken 14','klokken 14.00','kl. 14.00','kl. 14:00'):
            with self.subTest(spelling=spelling):
                self.assertEqual(evaluation.score(case,expected.replace('klokken 14',spelling),expected)['status'],'pass')
        local=evaluation.ROOT/'gdd/reports/epic42/47-quality-local.observations.jsonl'
        relative_clause=next(json.loads(line)['final'] for line in local.read_text().splitlines()
                             if json.loads(line)['id']=='local.text/text-similar-numbers/general_prompt/0')
        self.assertEqual(evaluation.score(case,relative_clause,expected)['status'],'pass')
        # Equivalent clock spelling still changes words under punctuation-only.
        punctuation=case['expected']['punctuation_only']
        self.assertEqual(evaluation.score(case,punctuation.replace('klokken 14','kl. 14.00'),punctuation,exact_words=True)['status'],'fail')

    def test_fixed_clock_relation_rejects_wrong_time_and_negation(self):
        case=next(c for c in self.cleanup['cases'] if c['id']=='text-similar-numbers')
        expected=case['expected']['general_prompt']
        for replacement in ('klokken 14.30','kl. 14.30','klokken 15','ikke klokken 14'):
            with self.subTest(replacement=replacement):
                self.assertEqual(evaluation.score(case,expected.replace('klokken 14',replacement),expected)['status'],'fail')
        self.assertEqual(evaluation.score(case,expected.replace('ikke 50','50'),expected)['status'],'fail')

    def test_inventory_covers_matrix_and_profiles_without_cartesian(self):
        self.assertEqual(len({r['id'] for r in self.rows}),len(self.rows))
        for speech,text in evaluation.MATRIX.values():
            self.assertTrue(any(r['speech']==speech and r['text']==text for r in self.rows))
        self.assertEqual({r['profile'] for r in self.rows if r['speech'] is None},set(self.cleanup['profiles']))
        self.assertTrue(all(r['case']!='da_test' for r in self.rows))

    def test_not_run_or_unavailable_is_never_pass(self):
        row=self.rows[0]
        result=evaluation.report([row],{},self.manifest,self.cleanup,{})[0]
        self.assertEqual(result['status'],'not_tested')
        error={'id':row['id'],'error':'provider.http','error_stage':'asr','raw':None,'final':None}
        result=evaluation.report([row],{row['id']:error},self.manifest,self.cleanup,{})[0]
        self.assertEqual(result['status'],'fail')
        self.assertEqual(result['failure_categories'],['asr_error'])
        self.assertEqual(result['schema_version'],1)
        self.assertEqual(len(result['fixture_sha256']),64)
        self.assertIn('not_tested',evaluation.markdown(evaluation.report(self.rows,{},self.manifest,self.cleanup,{})))

    def test_ai_failure_still_scores_asr(self):
        row=next(r for r in self.rows if r['text']=='local.text' and r['speech'])
        case=next(c for c in self.manifest['cases'] if c['id']==row['case'])
        obs={'id':row['id'],'error':'ai.incomplete','error_stage':'ai','raw':case['reference']['text'],'final':None}
        result=evaluation.report([row],{row['id']:obs},self.manifest,self.cleanup,{})[0]
        self.assertEqual(result['status'],'fail')
        self.assertEqual(result['asr']['wer'],0)

    def test_coarse_anchors_missing_duplicate_and_speaker_ids(self):
        case={'anchors':[{'start_seconds':1,'end_seconds':3,'text':'hello world','source_case':'x'}]}
        transcript={'segments':[{'speaker':'0:raw','words':[{'text':'hello','start':1,'end':2,'speaker':'0:raw'}, {'text':'world','start':2,'end':3,'speaker':'0:raw'}]}]}
        good=evaluation.structure(case,transcript,None)
        self.assertEqual(good['status'],'pass')
        self.assertEqual(good['raw_speaker_ids'],['0:raw'])
        self.assertEqual(good['exact_acoustic_alignment'],'not_tested')
        transcript['segments'][0]['words']*=2
        self.assertEqual(evaluation.structure(case,transcript,None)['status'],'fail')
        transcript['segments'][0]['words']=[]
        self.assertEqual(evaluation.structure(case,transcript,None)['status'],'not_tested')
        self.assertEqual(evaluation.structure(case,None,None)['status'],'not_tested')

if __name__=='__main__':unittest.main()
