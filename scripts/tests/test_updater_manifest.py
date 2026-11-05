import base64
import hashlib
import importlib.util
import io
import tarfile
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('manifest', Path(__file__).resolve().parents[1] / 'generate-updater-manifest.py')
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
OPENSSL = os.environ.get('OPENSSL', 'openssl')


class ManifestTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.artifact = self.root / 'Vara_1.0.0_aarch64.app.tar.gz'
        self.write_macos_archive()
        self.signature = self.root / 'artifact.sig'
        subprocess.run([OPENSSL, 'genpkey', '-algorithm', 'ED25519', '-out', str(self.root/'private')], check=True, capture_output=True)
        public = subprocess.run([OPENSSL, 'pkey', '-in', str(self.root/'private'), '-pubout', '-outform', 'DER'], check=True, capture_output=True).stdout[-32:]
        self.key_id = b'12345678'
        self.public = base64.b64encode(b'untrusted comment: test key\n' + base64.b64encode(b'Ed'+self.key_id+public)).decode()
        self.sign()

    def write_macos_archive(self, entries=None):
        entries = entries or [('Vara.app', tarfile.DIRTYPE, b''),
                              ('Vara.app/Contents', tarfile.DIRTYPE, b''),
                              ('Vara.app/Contents/Info.plist', tarfile.REGTYPE, b'fixture plist')]
        with tarfile.open(self.artifact, 'w:gz') as archive:
            for name, kind, content in entries:
                info = tarfile.TarInfo(name)
                info.type = kind
                if kind == tarfile.SYMTYPE: info.linkname = content.decode()
                info.size = len(content) if kind == tarfile.REGTYPE else 0
                archive.addfile(info, io.BytesIO(content) if info.size else None)

    def sign_bytes(self, data):
        (self.root/'input').write_bytes(data)
        return subprocess.run([OPENSSL, 'pkeyutl', '-sign', '-inkey', str(self.root/'private'), '-rawin', '-in', str(self.root/'input')], check=True, capture_output=True).stdout

    def sign(self, version='1.0.0'):
        sig = self.sign_bytes(hashlib.blake2b(self.artifact.read_bytes()).digest())
        comment = f'timestamp:1\tfile:{self.artifact.name}\tversion:{version}'
        text = b'untrusted comment: test signature\n' + base64.b64encode(b'ED'+self.key_id+sig) + b'\ntrusted comment: ' + comment.encode() + b'\n' + base64.b64encode(self.sign_bytes(sig+comment.encode()))
        self.signature.write_text(base64.b64encode(text).decode())

    def generate(self, platform='darwin-aarch64', version='1.0.0'):
        return m.generate(version,self.public,[(platform,self.artifact,self.signature)])

    def test_real_signature_and_github_mapping(self):
        result = self.generate()
        self.assertEqual(result['version'], '1.0.0')
        self.assertEqual(result['platforms']['darwin-aarch64']['url'], 'https://github.com/mikkelkrogsholm/vara/releases/download/v1.0.0/Vara_1.0.0_aarch64.app.tar.gz')
        self.assertEqual(result,self.generate())

    def test_windows_linux_and_intel_mapping(self):
        for platform in ['darwin-x86_64','windows-x86_64-nsis','windows-aarch64-nsis','linux-x86_64-appimage','linux-aarch64-appimage']:
            self.artifact = self.root / ('Vara_1.0.0_' + platform + m.PLATFORMS[platform])
            if platform.startswith('darwin-'): self.write_macos_archive()
            else: self.artifact.write_bytes(b'actual platform fixture')
            self.sign()
            self.assertIn(platform,self.generate(platform)['platforms'])

    def test_signed_appledouble_root_fails_before_manifest(self):
        self.write_macos_archive([('._Vara.app', tarfile.REGTYPE, b'AppleDouble metadata'),
                                  ('Vara.app/Contents/Info.plist', tarfile.REGTYPE, b'fixture')])
        self.sign()
        with self.assertRaisesRegex(ValueError, 'AppleDouble'): self.generate()

    def test_valid_app_root_and_relative_symlink(self):
        self.write_macos_archive([('Vara.app', tarfile.DIRTYPE, b''),
                                  ('Vara.app/Contents/_CodeSignature/CodeResources', tarfile.REGTYPE, b'fixture'),
                                  ('Vara.app/Contents/CodeResources', tarfile.SYMTYPE, b'_CodeSignature/CodeResources')])
        with tarfile.open(self.artifact, 'r:gz') as archive:
            entries = archive.getmembers()
        self.assertTrue(entries[0].isdir())
        self.sign()
        self.assertIn('darwin-aarch64', self.generate()['platforms'])
        m.validate_macos_archive(self.artifact)

    def test_unsafe_or_invalid_app_roots(self):
        for name in ['Vara.app', 'Vara.app/../escape', '/Vara.app/Contents/file',
                     './Vara.app/Contents/file', 'Other/Contents/file']:
            with self.subTest(name=name):
                self.write_macos_archive([(name, tarfile.REGTYPE, b'fixture')])
                self.sign()
                with self.assertRaises(ValueError): self.generate()
        self.write_macos_archive([('Vara.app/Contents/file', tarfile.REGTYPE, b'a'),
                                  ('Other.app/Contents/file', tarfile.REGTYPE, b'b')])
        self.sign()
        with self.assertRaisesRegex(ValueError, 'one .app root'): self.generate()

    def test_symlinks_cannot_escape_app_root(self):
        for target in [b'/tmp/outside', b'../../outside']:
            self.write_macos_archive([('Vara.app/Contents/Info.plist', tarfile.REGTYPE, b'fixture'),
                                      ('Vara.app/Contents/link', tarfile.SYMTYPE, target)])
            self.sign()
            with self.assertRaisesRegex(ValueError, 'Unsafe.*symlink'): self.generate()

    def test_tampered_artifact_or_signature(self):
        self.artifact.write_bytes(b'corrupt')
        with self.assertRaisesRegex(ValueError,'signature'): self.generate()
        self.signature.write_text('not a signature')
        with self.assertRaises(ValueError): self.generate()

    def test_missing_signature_and_artifact(self):
        self.signature.unlink()
        with self.assertRaises(OSError): self.generate()
        self.artifact.unlink()
        with self.assertRaises(ValueError): self.generate()

    def test_version_platform_and_bundle_validation(self):
        for version in ['v1.0.0','1.0.0-rc.1','01.0.0','1.0.0+build','1.1.0']:
            with self.subTest(version=version), self.assertRaises(ValueError): self.generate(version=version)
        for platform in ['macos-arm64','windows-x86_64-nsis','linux-x86_64-appimage']:
            with self.subTest(platform=platform), self.assertRaises(ValueError): self.generate(platform=platform)
        with self.assertRaises(ValueError): m.generate('1.0.0',self.public,[])
        with self.assertRaises(ValueError): m.generate('1.0.0',self.public,[('darwin-aarch64',self.artifact,self.signature)]*2)

    def test_signed_version_and_public_key_validation(self):
        self.sign('0.9.0')
        with self.assertRaisesRegex(ValueError,'exact release version'): self.generate()
        with self.assertRaises(ValueError): m.generate('1.0.0','bad', [('darwin-aarch64',self.artifact,self.signature)])


if __name__ == '__main__': unittest.main()
