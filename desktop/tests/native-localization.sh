#!/bin/bash
# Isolated AppKit/Foundation verification. Never opens Vara, the microphone or a permission dialog.
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
output="${VARA_NATIVE_TEST_OUTPUT:-/tmp/vara-22b-native}"
header_dir="${VARA_DESIGN_HEADER_DIR:?Set VARA_DESIGN_HEADER_DIR to the existing cargo-generated vara_design.h directory}"
mkdir -p "$output"
cd "$repo"
clang -fobjc-arc -fblocks -I "$header_dir" -framework AppKit -framework QuartzCore desktop/tests/hud-native.m -o "$output/hud-native"
"$output/hud-native" crates/vara-core/messages/runtime.json "$output/hud-da.png" "$output/hud-en.png"
clang -fobjc-arc -framework Foundation desktop/tests/info-plist-native.m -o "$output/info-native"
python3 - "$output" <<'PY'
import json, pathlib, plistlib, shutil, sys
output=pathlib.Path(sys.argv[1]); root=pathlib.Path('desktop/src-tauri')
config=json.loads((root/'tauri.conf.json').read_text()); mapping=config['bundle']['macOS']['files']
assert set(mapping)=={'Resources/da.lproj/InfoPlist.strings','Resources/en.lproj/InfoPlist.strings'}
contents=output/'Fixture.app/Contents'; contents.mkdir(parents=True,exist_ok=True)
with (root/'Info.plist').open('rb') as f: plist=plistlib.load(f)
plist.update({'CFBundleIdentifier':'dk.vara.fixture','CFBundleDevelopmentRegion':'English','CFBundleName':'Vara Fixture','CFBundleExecutable':'fixture','CFBundlePackageType':'APPL'})
(contents/'Info.plist').write_bytes(plistlib.dumps(plist))
for target,source in mapping.items():
    dest=contents/target; dest.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(root/source,dest)
(contents/'MacOS').mkdir(exist_ok=True); shutil.copy(output/'info-native',contents/'MacOS/fixture')
PY
bundle="$output/Fixture.app"
plutil -lint "$bundle/Contents/Info.plist" "$bundle/Contents/Resources/da.lproj/InfoPlist.strings" "$bundle/Contents/Resources/en.lproj/InfoPlist.strings"
"$bundle/Contents/MacOS/fixture" "$bundle" 'Vara bruger mikrofonen, når du starter en diktering. Lokal behandling er standard.' -AppleLanguages '(da)'
"$bundle/Contents/MacOS/fixture" "$bundle" 'Vara uses the microphone when you start dictation. On-device processing is the default.' -AppleLanguages '(en)'
