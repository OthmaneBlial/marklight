#!/usr/bin/env python3
"""Smoke-test locally built macOS archives without claiming download trust."""
import argparse
import hashlib
import json
import platform
import plistlib
import subprocess
import tarfile
import tempfile
from pathlib import Path

root = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser()
parser.add_argument('--output-dir', type=Path, default=root/'artifacts/release')
args = parser.parse_args()
if platform.system() != 'Darwin':
    raise SystemExit('The macOS package smoke test requires macOS.')

output = args.output_dir.resolve()
version = json.loads((root/'apps/desktop/src-tauri/tauri.conf.json').read_text())['version']
machine = platform.machine()
prefix = f'Marklight-{version}-macOS-{machine}'
zip_path = output/f'{prefix}.app.zip'
dmg_path = output/f'{prefix}.dmg'
cli_path = output/f'marklight-{version}-darwin-{machine}.tar.gz'
expected = {path.name: path for path in [zip_path, dmg_path, cli_path]}

lines = (output/'SHA256SUMS').read_text().splitlines()
assert len(lines) == len(expected), 'wrong number of checksum entries'
recorded = {}
for line in lines:
    digest, separator, name = line.partition('  ')
    assert separator and len(digest) == 64 and name not in recorded, line
    recorded[name] = digest
assert set(recorded) == set(expected), 'checksum names differ from package assets'
for name, path in expected.items():
    assert path.is_file(), name
    assert hashlib.sha256(path.read_bytes()).hexdigest() == recorded[name], name

with tempfile.TemporaryDirectory(prefix='macos-package-smoke-', dir=root/'artifacts') as temp:
    work = Path(temp)
    extracted = work/'zip'
    extracted.mkdir()
    subprocess.run(['ditto', '-xk', str(zip_path), str(extracted)], check=True)
    app = extracted/'Marklight.app'
    info = plistlib.loads((app/'Contents/Info.plist').read_bytes())
    assert info['CFBundleShortVersionString'] == version
    assert info['CFBundleIdentifier'] == 'dev.marklight.reader'
    executable = app/'Contents/MacOS'/info['CFBundleExecutable']
    assert executable.is_file()
    subprocess.run(['codesign', '--verify', '--deep', '--strict', str(app)], check=True)
    for name in ['sample.md', 'sample-guide.md']:
        bundled = app/'Contents/Resources/sample'/name
        fixture = root/'fixtures/markdown'/name
        assert bundled.read_bytes() == fixture.read_bytes(), name

    mount = work/'mounted-dmg'
    mount.mkdir()
    subprocess.run(['hdiutil', 'attach', '-readonly', '-nobrowse', '-mountpoint', str(mount), str(dmg_path)], check=True, stdout=subprocess.DEVNULL)
    try:
        mounted = mount/'Marklight.app'/'Contents/MacOS'/info['CFBundleExecutable']
        assert hashlib.sha256(mounted.read_bytes()).digest() == hashlib.sha256(executable.read_bytes()).digest()
        assert (mount/'Applications').is_symlink()
        assert (mount/'Applications').readlink() == Path('/Applications')
    finally:
        subprocess.run(['hdiutil', 'detach', str(mount)], check=True, stdout=subprocess.DEVNULL)

    with tarfile.open(cli_path) as archive:
        assert set(archive.getnames()) == {'marklight', 'LICENSE', 'INSTALLATION.md'}
        cli = work/'marklight'
        cli.write_bytes(archive.extractfile('marklight').read())
        cli.chmod(0o755)
    assert subprocess.check_output([str(cli), '--version'], text=True).strip() == f'marklight {version}'
    fixture = root/'fixtures/markdown/gfm.md'
    rendered = subprocess.check_output([str(cli), str(fixture), '--plain', '--no-pager', '--width', '60'])
    assert rendered == (root/'fixtures/markdown/gfm.terminal.txt').read_bytes()

print(f'PASS: {prefix} checksums, extracted app seal/resources, DMG contents and CLI golden')
