import base64
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('release_config', Path(__file__).resolve().parents[1] / 'generate-release-config.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ReleaseConfigurationTests(unittest.TestCase):
    def test_runtime_key_also_reaches_cli_bundler(self):
        key = base64.b64encode(b'untrusted comment: disposable fixture\nfixture-public-key\n').decode()
        overlay = module.configuration(key)
        self.assertEqual(overlay['plugins']['updater']['pubkey'], key)
        self.assertTrue(overlay['bundle']['createUpdaterArtifacts'])
        self.assertEqual(overlay['bundle']['targets'], ['app', 'dmg'])

    def test_missing_malformed_and_commentless_keys_fail_before_build(self):
        for key in ['', 'not base64!', base64.b64encode(b'bare-key').decode()]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                module.configuration(key)
