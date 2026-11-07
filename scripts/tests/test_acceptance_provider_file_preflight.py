"""Offline guard regressions; importing this module never issues paid requests."""
import importlib.util
import json
import pathlib
import shutil
import tempfile
import unittest
from unittest.mock import patch

SCRIPT = pathlib.Path(__file__).with_name('provider_file_acceptance.py')
spec = importlib.util.spec_from_file_location('provider_file_acceptance', SCRIPT)
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
REPO = SCRIPT.resolve().parents[2]

class PublicSourcePreflight(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = pathlib.Path(self.tmp.name)
        self.corpus = self.repo/'Samples/acceptance'
        (self.corpus/'audio').mkdir(parents=True)
        self.manifest = json.loads((REPO/'Samples/acceptance/manifest.json').read_text())
        for case in module.PINS:
            shutil.copyfile(REPO/'Samples/acceptance/audio'/(case+'.wav'), self.corpus/'audio'/(case+'.wav'))
        self.save()
    def save(self):
        (self.corpus/'manifest.json').write_text(json.dumps(self.manifest))
    def row(self):
        return next(c for c in self.manifest['cases'] if c['id']=='four-speakers')
    def test_approved_fixed_sources(self):
        self.assertEqual(len(module.approved_sources(self.repo)),2)
    def test_second_source_drift_rejected(self):
        with (self.corpus/'audio/four-speakers.wav').open('ab') as f: f.write(b'changed')
        with self.assertRaisesRegex(ValueError,'hash mismatch'):
            module.approved_sources(self.repo)
    def test_permission_removed_rejected(self):
        self.row()['rights']['cloud_upload_permitted']=False; self.save()
        with self.assertRaisesRegex(ValueError,'permission'):
            module.approved_sources(self.repo)
    def test_oversized_duration_rejected(self):
        self.row()['duration_seconds']=301; self.save()
        with self.assertRaisesRegex(ValueError,'duration exceeded'):
            module.approved_sources(self.repo)
    def test_changed_pin_cannot_authorize_replacement(self):
        self.row()['sha256']='0'*64; self.save()
        with self.assertRaisesRegex(ValueError,'hash mismatch'):
            module.approved_sources(self.repo)
    def test_main_checks_both_before_any_process_or_profile(self):
        self.row()['rights']['cloud_upload_permitted']=False; self.save()
        # Resolve the existing fixed repo location into our temporary copy.
        fake_script=self.repo/'scripts/tests/provider_file_acceptance.py'
        out=self.repo/'must-not-be-created'
        with patch.object(module,'__file__',str(fake_script)), patch.object(module.sys,'argv',[str(SCRIPT),'/missing-exe',str(out),'--allow-two-cloud-requests']), patch.object(module.subprocess,'run') as run:
            with self.assertRaisesRegex(ValueError,'permission'):
                module.main()
            run.assert_not_called()
        self.assertFalse(out.exists())

if __name__ == '__main__': unittest.main()
