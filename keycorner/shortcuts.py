"""Read/apply the three Keycorner shortcuts without executing user-supplied code."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

ACTIONS = {
    'toggle': 'Keycorner: show/resume or hide/pause',
    'toggleSettings': 'Keycorner: toggle settings',
    'pause': 'Keycorner: pause/resume input',
}
DEFAULTS = dict(zip(ACTIONS, ('SUPER + CTRL + SHIFT + K', 'SUPER + CTRL + SHIFT + O', 'SUPER + CTRL + SHIFT + P')))
MODS = {'SUPER': 64, 'CTRL': 4, 'ALT': 8, 'SHIFT': 1}
BIND = re.compile(r'o\.bind\("([A-Za-z0-9_ +]*)", "([^"\n]*)", "omarchy-shell keycorner (toggle|toggleSettings|pause)"\)')


def normalize(value):
    if not isinstance(value, str) or len(value) > 100:
        raise ValueError('Invalid shortcut')
    if not value.strip():
        return ''
    parts = [p.strip().upper() for p in value.split('+')]
    if len(parts) < 2 or any(p not in MODS for p in parts[:-1]) or len(set(parts[:-1])) != len(parts[:-1]):
        raise ValueError('Use modifiers plus a key, for example SUPER + CTRL + P')
    if not re.fullmatch(r'[A-Z0-9_]+', parts[-1]) or parts[-1] in MODS:
        raise ValueError('Use a letter, digit or Hyprland key name')
    # Preserve key-name casing (e.g. Return, Escape) for XKB keysym lookup.
    key = value.split('+')[-1].strip()
    if len(key) == 1 or re.fullmatch(r'[fF]\d+', key):
        key = key.upper()
    return ' + '.join([m for m in MODS if m in parts[:-1]] + [key])


def identity(value):
    parts = value.upper().split(' + ')
    return sum(MODS[m] for m in parts[:-1]), parts[-1]


def read(path):
    result = dict.fromkeys(ACTIONS, '')
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith('--'):
            continue
        match = BIND.fullmatch(line)
        if not match or match[2] != ACTIONS[match[3]] or result[match[3]]:
            raise ValueError('Shortcut file contains custom Lua; edit it manually or restore the standard file first')
        result[match[3]] = normalize(match[1])
    return result


def render(values):
    lines = ['-- Managed by Keycorner settings. Blank shortcuts are disabled.',
             '-- Bindings are unavailable while the screen is locked.']
    for action, description in ACTIONS.items():
        if values[action]:
            lines.append(f'o.bind("{values[action]}", "{description}", "omarchy-shell keycorner {action}")')
    return '\n'.join(lines) + '\n'


def check_conflicts(values, old, binds):
    selected = [identity(v) for v in values.values() if v]
    if len(selected) != len(set(selected)):
        raise ValueError('Each action needs a different shortcut')
    owned = {(identity(v), ACTIONS[a]) for a, v in old.items() if v}
    for binding in binds:
        combo = (binding.get('modmask'), str(binding.get('key', '')).upper())
        if combo in selected and (combo, binding.get('description')) not in owned:
            raise ValueError(f"Shortcut already used by {binding.get('description') or binding.get('dispatcher') or 'another action'}")


def run(*args):
    return subprocess.check_output(args, text=True, stderr=subprocess.PIPE, timeout=10)


def validate():
    run('hyprctl', 'reload')
    errors = run('hyprctl', 'configerrors').strip()
    if errors and errors != 'ok':
        raise ValueError('Hyprland rejected the configuration: ' + errors)


def write_atomic(path, data):
    fd, tmp = tempfile.mkstemp(prefix='.keycorner-shortcuts-', dir=path.parent)
    try:
        with os.fdopen(fd, 'w') as stream:
            stream.write(data)
        os.replace(tmp, path)
    finally:
        Path(tmp).unlink(missing_ok=True)


def apply(path, requested):
    if not isinstance(requested, dict) or set(requested) != set(ACTIONS):
        raise ValueError('Expected all three shortcut actions')
    values = {a: normalize(requested[a]) for a in ACTIONS}
    old = read(path)  # Refuse to replace arbitrary custom Lua.
    check_conflicts(values, old, json.loads(run('hyprctl', '-j', 'binds')))
    original = path.read_text()
    if original == render(values):
        return values
    # Reload rebuilds the bindings from scratch; removed combinations disappear.
    # No unrelated binding is unbound or overridden.
    write_atomic(path.with_suffix('.lua.backup'), original)
    try:
        write_atomic(path, render(values))
        validate()
    except Exception:
        write_atomic(path, original)
        validate()
        raise
    return values


def main():
    root = Path(os.environ.get('XDG_CONFIG_HOME', Path.home() / '.config'))
    path = root / 'hypr/keycorner-shortcuts.lua'
    try:
        values = apply(path, json.loads(sys.argv[2])) if len(sys.argv) > 1 and sys.argv[1] == 'apply' else read(path)
        print(json.dumps({'ok': True, 'shortcuts': values}))
    except Exception as error:
        print(json.dumps({'ok': False, 'error': str(error)}))
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
