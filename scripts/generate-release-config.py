#!/usr/bin/env python3
"""Emit a Tauri release overlay with the same public trust root as the runtime."""
import base64
import binascii
import json
import os
from pathlib import Path
import sys


def configuration(public_key: str) -> dict:
    public_key = public_key.strip()
    try:
        decoded = base64.b64decode(public_key, validate=True).decode('utf-8')
    except (binascii.Error, UnicodeDecodeError) as error:
        raise ValueError('VARA_UPDATER_PUBLIC_KEY must be a base64 Tauri public key') from error
    lines = decoded.splitlines()
    if len(lines) != 2 or not lines[0].startswith('untrusted comment: ') or not lines[1]:
        raise ValueError('VARA_UPDATER_PUBLIC_KEY must include the Tauri public-key comment')
    overlay = json.loads((Path(__file__).resolve().parents[1] / 'desktop/src-tauri/tauri.release.conf.json').read_text())
    overlay.setdefault('plugins', {}).setdefault('updater', {})['pubkey'] = public_key
    return overlay


if __name__ == '__main__':
    try:
        print(json.dumps(configuration(os.environ.get('VARA_UPDATER_PUBLIC_KEY', ''))))
    except ValueError as error:
        print(f'Release configuration failed: {error}', file=sys.stderr)
        sys.exit(1)
