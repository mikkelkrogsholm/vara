#!/usr/bin/env python3
"""Collect native manual-install previews; never publish or alter a profile."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib


def main():
    root = Path(__file__).resolve().parents[1]
    os.chdir(root)
    version = tomllib.loads(Path('Cargo.toml').read_text())['workspace']['package']['version']
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    if commit != os.environ['SOURCE_COMMIT']:
        raise SystemExit('Checkout changed during build')
    subprocess.run(['git', 'diff', '--exit-code', 'HEAD', '--'], check=True)
    platform = os.environ['BUILD_PLATFORM']
    expected_host = {'linux-x86_64': 'x86_64-unknown-linux-gnu',
                     'windows-x86_64': 'x86_64-pc-windows-msvc'}[platform]
    rustc = subprocess.check_output(['rustc', '-vV'], text=True)
    if f'host: {expected_host}\n' not in rustc:
        raise SystemExit(f'Unexpected Rust host for {platform}: {rustc}')
    patterns = {
        'linux-x86_64': ['deb/*.deb', 'appimage/*.AppImage'],
        'windows-x86_64': ['nsis/*.exe'],
    }[platform]
    binary = Path('target/release/vara-desktop' + ('.exe' if platform.startswith('windows') else ''))
    result = subprocess.run([str(binary.resolve()), 'cli', '--version'],
                            check=True, capture_output=True, text=True, timeout=30)
    if result.stdout.strip() != f'vara {version}':
        raise SystemExit(f'Unexpected CLI version (or test-harness build): {result.stdout!r}')
    out = Path('release-assets')
    out.mkdir(exist_ok=True)
    (out / 'cli-version.log').write_text(result.stdout)
    packages = []
    for pattern in patterns:
        matches = sorted(Path('target/release/bundle').glob(pattern))
        if not matches:
            raise SystemExit(f'Missing native package: {pattern}')
        for package in matches:
            if not package.stat().st_size or version not in package.name:
                raise SystemExit(f'Empty or unversioned package: {package}')
            shutil.copy2(package, out / package.name)
            packages.append(package.name)
    metadata = {
        'version': version, 'commit': commit, 'platform': platform,
        'runner_os': os.environ['RUNNER_OS'], 'runner_arch': os.environ['RUNNER_ARCH'],
        'workflow_run': f"{os.environ['GITHUB_SERVER_URL']}/{os.environ['GITHUB_REPOSITORY']}/actions/runs/{os.environ['GITHUB_RUN_ID']}",
        'rustc': rustc,
        'tauri_cli': (out / 'tauri-cli-version.log').read_text().strip(),
        'cargo_lock_sha256': hashlib.sha256(Path('Cargo.lock').read_bytes()).hexdigest(),
        'features': 'default (no test-harness)', 'packages': packages,
        'updater_artifacts': False, 'authenticode_signed': False,
        'validation': 'native library tests and CLI version only; installation, GUI, recording and upgrades unverified',
        'limitations': 'Experimental preview. Manual insertion; no app detection or HUD. Shared file backend and agent CLI operations unsupported. No Who runtime.',
    }
    (out / 'build-metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')
    sums = []
    for path in sorted(out.iterdir()):
        if path.is_file() and path.name != 'SHA256SUMS':
            sums.append(f'{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n')
    (out / 'SHA256SUMS').write_text(''.join(sums))
    print(f'Collected {len(packages)} {platform} preview package(s) from {commit}')


if __name__ == '__main__':
    main()
