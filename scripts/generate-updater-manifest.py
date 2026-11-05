#!/usr/bin/env python3
"""Generate a stable GitHub updater manifest after verifying real signed artifacts.

Requires OpenSSL 3 (OPENSSL may override the executable). Does not publish files.
"""
import argparse
import base64
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
from urllib.parse import quote

spec = importlib.util.spec_from_file_location('version_check', Path(__file__).with_name('check-version.py'))
version_check = importlib.util.module_from_spec(spec)
spec.loader.exec_module(version_check)
PLATFORMS = {
    'darwin-aarch64': '.app.tar.gz', 'darwin-x86_64': '.app.tar.gz',
    'windows-x86_64-nsis': '.exe', 'windows-aarch64-nsis': '.exe',
    'linux-x86_64-appimage': '.AppImage', 'linux-aarch64-appimage': '.AppImage',
}
REPOSITORY = 'https://github.com/mikkelkrogsholm/vara'


def decode(value):
    try:
        return base64.b64decode(value, validate=True)
    except (ValueError, TypeError) as error:
        raise ValueError('Invalid base64 signing metadata') from error


def verify(artifact: Path, signature: str, public_key: str, version: str):
    try:
        key_lines = decode(public_key.strip()).decode('utf-8').splitlines()
        lines = decode(signature.strip()).decode('utf-8').splitlines()
        key = decode(key_lines[1])
        signed = decode(lines[1])
        global_signature = decode(lines[3])
    except (IndexError, UnicodeError) as error:
        raise ValueError('Malformed Tauri signing metadata') from error
    if len(key) != 42 or key[:2] != b'Ed' or len(signed) != 74 or signed[:2] not in (b'ED', b'Ed') or len(global_signature) != 64:
        raise ValueError('Malformed Minisign key/signature')
    if key[2:10] != signed[2:10] or not lines[2].startswith('trusted comment: '):
        raise ValueError('Signing key mismatch or missing trusted comment')
    comment = lines[2][17:]
    if f'version:{version}' not in comment.split('\t'):
        raise ValueError('Signature must bind the exact release version')
    # Minisign signs the artifact's BLAKE2b-512 digest (ED), or raw bytes (Ed).
    data = artifact.read_bytes()
    message = hashlib.blake2b(data).digest() if signed[:2] == b'ED' else data
    with tempfile.TemporaryDirectory(prefix='vara-verify-') as directory:
        root = Path(directory)
        (root / 'key.der').write_bytes(bytes.fromhex('302a300506032b6570032100') + key[10:])
        for name, message, sig in [('artifact', message, signed[10:]), ('comment', signed[10:] + comment.encode(), global_signature)]:
            (root / 'message').write_bytes(message)
            (root / 'signature').write_bytes(sig)
            result = subprocess.run([os.environ.get('OPENSSL', 'openssl'), 'pkeyutl', '-verify', '-pubin', '-keyform', 'DER', '-inkey', str(root/'key.der'), '-rawin', '-in', str(root/'message'), '-sigfile', str(root/'signature')], capture_output=True)
            if result.returncode:
                raise ValueError(f'Invalid {name} signature (OpenSSL 3 required)')


def generate(version, public_key, artifacts, notes=''):
    if not version_check.SEMVER.fullmatch(version) or '-' in version or '+' in version:
        raise ValueError('Stable release requires a plain MAJOR.MINOR.PATCH version')
    platforms = {}
    for platform, artifact, signature_path in artifacts:
        if platform not in PLATFORMS or platform in platforms:
            raise ValueError('Unknown or duplicate platform')
        artifact, signature_path = Path(artifact), Path(signature_path)
        if not artifact.name.endswith(PLATFORMS[platform]) or version not in artifact.name:
            raise ValueError('Artifact name must contain version and match platform bundle type')
        if not artifact.is_file() or artifact.stat().st_size == 0:
            raise ValueError('Artifact is missing or empty')
        signature = signature_path.read_text().strip()
        verify(artifact, signature, public_key, version)
        platforms[platform] = {'signature': signature, 'url': f'{REPOSITORY}/releases/download/v{version}/{quote(artifact.name, safe="")}'}
    if not platforms:
        raise ValueError('At least one real platform artifact is required')
    return {'version': version, 'notes': notes, 'platforms': dict(sorted(platforms.items()))}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--version', required=True)
    parser.add_argument('--public-key-file', type=Path, required=True)
    parser.add_argument('--artifact', nargs=3, action='append', metavar=('PLATFORM', 'ARTIFACT', 'SIGNATURE'), required=True)
    parser.add_argument('--notes-file', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.version != version_check.check(version_check.ROOT, 'v' + args.version):
            raise ValueError('Workspace version mismatch')
        manifest = generate(args.version, args.public_key_file.read_text(), args.artifact, args.notes_file.read_text() if args.notes_file else '')
        args.output.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    except (OSError, ValueError) as error:
        parser.exit(1, f'Manifest generation failed: {error}\n')


if __name__ == '__main__':
    main()
