"""Prepare or verify the bundled native executable. Developer/CI tool only."""
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
paths = [root/'Cargo.toml', root/'Cargo.lock', root/'packaging/runtime-files.txt']
paths += sorted((root/'src').glob('*.rs')) + sorted((root/'data').glob('*.json'))
digest = hashlib.sha256()
for path in sorted(paths):
    digest.update(path.relative_to(root).as_posix().encode() + b'\0' + path.read_bytes() + b'\0')
binary = root/'bin/keycorner'
metadata = root/'packaging/native.json'
if sys.argv[1:] == ['prepare']:
    binary.parent.mkdir(exist_ok=True)
    shutil.copyfile(root/'target/release/keycorner', binary)
    binary.chmod(0o755)
    metadata.write_text(json.dumps({'target':'x86_64-unknown-linux-gnu', 'version':json.loads((root/'manifest.json').read_text())['version'], 'sha256':hashlib.sha256(binary.read_bytes()).hexdigest(), 'source_sha256':digest.hexdigest()},indent=2)+'\n')
info = json.loads(metadata.read_text())
assert info['target'] == 'x86_64-unknown-linux-gnu'
assert info['source_sha256'] == digest.hexdigest(), 'Bundled executable is stale; run scripts/prepare-native.sh'
payload = binary.read_bytes()
assert hashlib.sha256(payload).hexdigest() == info['sha256'], 'Native checksum mismatch'
assert payload[:4] == b'\x7fELF' and payload[4:6] == b'\x02\x01' and payload[18:20] == b'\x3e\0', 'Expected x86_64 ELF'
assert not re.search(rb'/home/[^/\x00 ]+/|/Users/[^/\x00 ]+/',payload), 'Private build path'
version = json.loads(subprocess.check_output([binary,'--version']))['version']
assert version == info['version'] == json.loads((root/'manifest.json').read_text())['version']
print('Bundled native executable matches its checksum, source fingerprint and version.')
