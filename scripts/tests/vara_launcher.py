#!/usr/bin/env python3
"""Check the installable launcher's forwarding and missing-app diagnostic."""
from pathlib import Path
import json
import subprocess
import tempfile

launcher = Path(__file__).resolve().parents[1] / 'vara'
with tempfile.TemporaryDirectory(prefix='vara-launcher-') as tmp:
    root = Path(tmp)
    app = root / 'Vara.app'
    binary = app / 'Contents/MacOS/vara-desktop'
    binary.parent.mkdir(parents=True)
    # Substitute only the fixed installation location so the test never touches
    # the user's installed app. The installed launcher itself has no override.
    test_launcher = root / 'vara'
    test_launcher.write_text(launcher.read_text().replace('/Applications/Vara.app', str(app)))
    test_launcher.chmod(0o755)
    missing = subprocess.run([test_launcher, 'models'], text=True, capture_output=True)
    assert missing.returncode == 5 and 'missing or moved' in missing.stderr
    assert json.loads(missing.stdout)['error']['code'] == 'cli.app_missing'
    binary.write_text('#!/bin/sh\nprintf "%s\\n" "$@"\n')
    binary.chmod(0o755)
    old = subprocess.run([test_launcher, 'models'], text=True, capture_output=True)
    assert old.returncode == 5 and json.loads(old.stdout)['error']['code'] == 'cli.app_incompatible'
    marker = app / 'Contents/Resources/vara-cli-v1'
    marker.parent.mkdir()
    marker.write_text('Vara agent CLI launcher protocol 1\n')
    forwarded = subprocess.run([test_launcher, 'transcribe', 'a file.wav', '--json'], text=True, capture_output=True, check=True)
    assert forwarded.stdout.splitlines() == ['cli', 'transcribe', 'a file.wav', '--json']
print('vara launcher checks passed')
