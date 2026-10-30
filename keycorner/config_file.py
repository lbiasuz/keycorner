"""Create first-run settings without replacing an existing user's configuration."""
import json
import os
from pathlib import Path


def ensure_config(path):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    except FileExistsError:
        return False
    with os.fdopen(fd, 'w') as stream:
        json.dump({'version': 1, 'backend': 'standard'}, stream)
        stream.write('\n')
    return True


if __name__ == '__main__':
    root = Path(os.environ.get('XDG_CONFIG_HOME', Path.home() / '.config'))
    ensure_config(root / 'omarchy/keycorner.json')
