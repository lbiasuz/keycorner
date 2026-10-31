"""Check a locally built native archive without installing or opening input devices."""
import json
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile

root = Path(__file__).resolve().parents[1]
with tarfile.open(sys.argv[1]) as archive:
    names = set(archive.getnames())
    assert all(not p.startswith('/') and '..' not in Path(p).parts for p in names)
    assert not any(p.endswith(('.vil', '.uf2', '.bin', '.py')) or '/.git/' in p for p in names)
    manifest = json.loads(archive.extractfile('keycorner-rust/manifest.json').read())
    for entry in manifest['entryPoints'].values():
        assert 'keycorner-rust/' + entry in names, entry
    for member in archive.getmembers():
        assert member.isfile() or member.isdir(), 'Unexpected archive link/device'
        if member.name.endswith('.qml'):
            source = archive.extractfile(member).read().decode()
            for name in re.findall(r'''import\s+['"]([^'"]+\.js)['"]''', source):
                assert str(Path(member.name).parent / name) in names, (member.name, name)
    with tempfile.TemporaryDirectory() as temp:
        archive.extractall(temp, filter='data')
        binary = Path(temp) / 'keycorner-rust/bin/keycorner'
        payload = binary.read_bytes()
        assert not re.search(rb'/home/[^/\x00 ]+/|/Users/[^/\x00 ]+/', payload), 'Private build path in executable'
        version = json.loads(subprocess.check_output([binary, '--version']))['version']
        assert version == json.loads((root / 'manifest.json').read_text())['version']
print('Archive paths, UI dependencies, contents and executable version verified.')
