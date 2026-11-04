#!/usr/bin/env python3
"""Check the workspace version mirrors without invoking Cargo."""
import argparse
import glob
import json
from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent
SEMVER = re.compile(r'^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*))*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$', re.ASCII)


def check(root: Path, tag: str | None = None) -> str:
    workspace = tomllib.loads((root / 'Cargo.toml').read_text())['workspace']
    version = workspace['package']['version']
    if not isinstance(version, str) or not SEMVER.fullmatch(version):
        raise ValueError(f'Invalid workspace SemVer: {version!r}')
    members = []
    for pattern in workspace['members']:
        paths = glob.glob(str(root / pattern))
        if not paths:
            raise ValueError(f'Workspace member matches nothing: {pattern}')
        members.extend(Path(path) for path in paths)
    excluded = {p.resolve() for pattern in workspace.get('exclude', []) for p in map(Path, glob.glob(str(root / pattern)))}
    lock = tomllib.loads((root / 'Cargo.lock').read_text())['package']
    for member in members:
        if member.resolve() in excluded:
            continue
        package = tomllib.loads((member / 'Cargo.toml').read_text())['package']
        if package.get('version') != {'workspace': True}:
            raise ValueError(f'{member.relative_to(root)} must inherit workspace version')
        matches = [p for p in lock if p['name'] == package['name'] and 'source' not in p]
        if len(matches) != 1 or matches[0]['version'] != version:
            raise ValueError(f'Cargo.lock version mismatch for {package["name"]}')
    config = json.loads((root / 'desktop/src-tauri/tauri.conf.json').read_text())
    if config.get('version') != version:
        raise ValueError('Tauri version does not match workspace version')
    if tag is not None and tag != 'v' + version:
        raise ValueError(f'Tag must be v{version}, got {tag!r}')
    return version


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tag', help='validate a release tag against the workspace version')
    args = parser.parse_args()
    try:
        version = check(ROOT, args.tag)
    except (OSError, KeyError, TypeError, ValueError) as exc:
        print(f'Version check failed: {exc}', file=sys.stderr)
        return 1
    print(f'Version check passed: {version}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
