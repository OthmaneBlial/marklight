#!/usr/bin/env python3
"""Install the actual npm tarball into an isolated prefix and exercise its CLI."""
import argparse
import json
import subprocess
import tarfile
import tempfile
from pathlib import Path

root = Path(__file__).resolve().parent.parent
version = json.loads((root/'packages/npm/package.json').read_text())['version']
parser = argparse.ArgumentParser()
parser.add_argument('package', nargs='?', default=str(root/f'artifacts/npm/marklight-{version}.tgz'))
args = parser.parse_args()
package = args.package
if package.endswith('.tgz'):
    path = Path(package).resolve()
    with tarfile.open(path) as archive:
        files = {member.name for member in archive.getmembers() if member.isfile()}
        assert files == {'package/package.json','package/README.md','package/LICENSE','package/bin/marklight'}, files
        metadata = json.load(archive.extractfile('package/package.json'))
        assert metadata['os'] == ['darwin'] and metadata['cpu'] == ['arm64']
        assert not metadata.get('scripts') and not metadata.get('dependencies')
    package = str(path)
(root/'artifacts').mkdir(exist_ok=True)
with tempfile.TemporaryDirectory(prefix='npm-smoke-', dir=root/'artifacts') as temporary:
    prefix = Path(temporary)
    subprocess.run(['npm','install','--global','--prefix',str(prefix),'--ignore-scripts',package], check=True)
    cli = prefix/'bin/marklight'
    installed = json.loads((prefix/'lib/node_modules/marklight/package.json').read_text())
    assert subprocess.check_output([str(cli),'--version'], text=True).strip() == f"marklight {installed['version']}"
    assert '--no-pager' in subprocess.check_output([str(cli),'--help'], text=True)
    fixture = root/'fixtures/markdown/gfm.md'
    rendered = subprocess.check_output([str(cli), str(fixture), '--plain','--no-pager','--width','60'])
    assert rendered == (root/'fixtures/markdown/gfm.terminal.txt').read_bytes()
    piped = subprocess.check_output([str(cli),'-','--plain','--no-pager','--width','60'], input=fixture.read_bytes())
    assert piped == rendered
    spaced = prefix/'a document with spaces.md'
    spaced.write_bytes(fixture.read_bytes())
    assert subprocess.check_output([str(cli),str(spaced),'--plain','--no-pager','--width','60']) == rendered
print('PASS: isolated npm install, native binary, version/help, GFM golden, stdin, spaced file path')
