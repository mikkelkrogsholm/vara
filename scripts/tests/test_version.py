import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('check_version', Path(__file__).resolve().parents[1] / 'check-version.py')
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class VersionCheckTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        (self.root / 'crate').mkdir()
        (self.root / 'desktop/src-tauri').mkdir(parents=True)
        self.write('Cargo.toml', '[workspace]\nmembers = ["crate"]\n[workspace.package]\nversion = "1.2.3-rc.1+build.5"\n')
        self.write('crate/Cargo.toml', '[package]\nname = "example"\nversion.workspace = true\n')
        self.write('Cargo.lock', '[[package]]\nname = "example"\nversion = "1.2.3-rc.1+build.5"\n')
        self.write('desktop/src-tauri/tauri.conf.json', json.dumps({'version': '1.2.3-rc.1+build.5'}))

    def write(self, name, text):
        (self.root / name).write_text(text)

    def test_valid_tag_and_nonmutating(self):
        before = {p: p.read_bytes() for p in self.root.rglob('*') if p.is_file()}
        self.assertEqual(checker.check(self.root, 'v1.2.3-rc.1+build.5'), '1.2.3-rc.1+build.5')
        self.assertEqual(before, {p: p.read_bytes() for p in self.root.rglob('*') if p.is_file()})

    def test_manifest_drift(self):
        self.write('crate/Cargo.toml', '[package]\nname = "example"\nversion = "1.2.3"\n')
        with self.assertRaisesRegex(ValueError, 'inherit'):
            checker.check(self.root)

    def test_config_drift(self):
        self.write('desktop/src-tauri/tauri.conf.json', '{"version": "1.2.2"}')
        with self.assertRaisesRegex(ValueError, 'Tauri'):
            checker.check(self.root)

    def test_lock_drift(self):
        self.write('Cargo.lock', '[[package]]\nname = "example"\nversion = "1.2.2"\n')
        with self.assertRaisesRegex(ValueError, 'Cargo.lock'):
            checker.check(self.root)

    def test_tag_drift(self):
        with self.assertRaisesRegex(ValueError, 'Tag'):
            checker.check(self.root, 'v1.2.3')

    def test_invalid_semver(self):
        for value in ('01.2.3', '1.2.3-01', '1.2', '1.2.3+', '1.2.3-α'):
            with self.subTest(value=value):
                self.assertIsNone(checker.SEMVER.fullmatch(value))


if __name__ == '__main__':
    unittest.main()
