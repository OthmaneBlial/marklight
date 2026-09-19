#!/usr/bin/env python3
"""Bundle already-built local macOS artifacts; no signing, upload or CI."""
import argparse
import hashlib
import json
import platform
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path

root = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser()
parser.add_argument('--output-dir', type=Path, default=root/'artifacts/release')
args = parser.parse_args()
if platform.system() != 'Darwin':
    raise SystemExit('This packaging script requires macOS.')
version = json.loads((root/'apps/desktop/src-tauri/tauri.conf.json').read_text())['version']
machine = platform.machine()
app = root/'target/release/bundle/macos/Marklight.app'
cli = root/'target/release/marklight'
if not app.is_dir() or not cli.is_file():
    raise SystemExit('Build both the native app and release CLI first (scripts/package.sh).')
subprocess.run([str(cli), '--version'], check=True)
# Seal the application bundle with a local ad-hoc identity. This uses no
# Developer ID credential and does not notarize or satisfy Gatekeeper trust.
subprocess.run(['codesign', '--force', '--deep', '--sign', '-', str(app)], check=True)
subprocess.run(['codesign', '--verify', '--deep', '--strict', str(app)], check=True)
output = args.output_dir.resolve()
output.mkdir(parents=True, exist_ok=True)
prefix = f'Marklight-{version}-macOS-{machine}'
zip_path = output/f'{prefix}.app.zip'
subprocess.run(['ditto', '-c', '-k', '--sequesterRsrc', '--keepParent', str(app), str(zip_path)], check=True)
with tempfile.TemporaryDirectory(prefix='dmg-stage-', dir=root/'artifacts') as staging:
    stage = Path(staging)
    subprocess.run(['ditto', str(app), str(stage/'Marklight.app')], check=True)
    (stage/'Applications').symlink_to('/Applications')
    dmg = output/f'{prefix}.dmg'
    subprocess.run(['hdiutil', 'create', '-ov', '-volname', 'Marklight', '-srcfolder', staging, '-format', 'UDZO', str(dmg)], check=True)
    subprocess.run(['hdiutil', 'verify', str(dmg)], check=True)
cli_archive = output/f'marklight-{version}-darwin-{machine}.tar.gz'
with tarfile.open(cli_archive, 'w:gz') as archive:
    archive.add(cli, arcname='marklight')
    archive.add(root/'LICENSE', arcname='LICENSE')
    archive.add(root/'docs/INSTALLATION.md', arcname='INSTALLATION.md')
checksums = []
for path in [zip_path, dmg, cli_archive]:
    checksums.append(f'{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}')
(output/'SHA256SUMS').write_text('\n'.join(checksums)+'\n')
print('\n'.join(str(path) for path in [zip_path, dmg, cli_archive, output/'SHA256SUMS']))
