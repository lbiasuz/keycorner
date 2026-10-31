"""Explicit physical-keyboard selection without opening input streams."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

SYS = Path('/sys')


def read(path):
    try: return path.read_text().strip()
    except OSError: return ''


def properties(node):
    result = subprocess.run(['udevadm', 'info', '--query=property', '--name=' + str(node)],
                            capture_output=True, text=True, timeout=5, check=True)
    return dict(line.split('=', 1) for line in result.stdout.splitlines() if '=' in line)


def usb_parent(path):
    for parent in path.resolve().parents:
        if (parent / 'idVendor').is_file() and (parent / 'idProduct').is_file():
            return parent
    return None


def keyboard_capable(bits):
    # sysfs renders most-significant machine words first; pad each word.
    words = bits.split()
    width = 16 if sys.maxsize > 2**32 else 8
    value = int(''.join(w.zfill(width) for w in words), 16) if words else 0
    return all(value & (1 << k) for k in (28, 30, 44, 42))  # Enter, A, Z, Shift


def token(identity):
    return hashlib.sha256(identity.encode()).hexdigest()[:24]


def discover():
    groups = {}
    for entry in sorted((SYS / 'class/input').glob('event*')):
        if not keyboard_capable(read(entry / 'device/capabilities/key')): continue
        node = Path('/dev/input') / entry.name
        try: props = properties(node)
        except (OSError, subprocess.SubprocessError): continue
        if props.get('ID_INPUT_KEYBOARD') != '1': continue
        parent = usb_parent(entry / 'device')
        name = read(entry / 'device/name')[:160] or 'Keyboard'
        if parent:
            vid, pid, serial = (read(parent / f) for f in ('idVendor', 'idProduct', 'serial'))
            identity = f'usb:{parent}:{vid}:{pid}:{serial}'
            match = {'usb_port': parent.name, 'vendor': vid, 'product': pid, 'serial': serial}
            name = (read(parent / 'product') or name)[:160]
        else:
            phys, uniq = read(entry / 'device/phys'), read(entry / 'device/uniq')
            # Never use volatile event/input numbers as a saved identity.
            if not phys: continue
            identity = f'input:{name}:{phys}:{uniq}'
            match = {'name': name, 'phys': phys, 'uniq': uniq}
        key = token(identity)
        group = groups.setdefault(key, {'id': key, 'name': name, 'match': match,
                                       'events': [], 'eventIdentity': {}, 'hidraw': [], 'vial': False})
        try: st = node.stat()
        except OSError: continue
        group['events'].append(str(node))
        group['eventIdentity'][str(node)] = [st.st_dev, st.st_ino, st.st_rdev]
        if parent: group['_usb'] = str(parent)
    for entry in sorted((SYS / 'class/hidraw').glob('hidraw*')):
        parent = usb_parent(entry / 'device')
        if parent is None: continue
        for group in groups.values():
            if group.get('_usb') == str(parent): group['hidraw'].append('/dev/' + entry.name)
    for group in groups.values():
        group.pop('_usb', None)
        m = group['match']
        # Other Vial devices can use Standard mode; do not claim their report formats work.
        group['vial'] = (m.get('vendor'), m.get('product')) == ('3034', '5659') and 'vial:' in m.get('serial', '')
        group['standardAccessible'] = all(os.access(p, os.R_OK) for p in group['events'])
        group['vialAccessible'] = bool(group['hidraw']) and all(os.access(p, os.R_OK | os.W_OK) for p in group['hidraw'])
        group['connection'] = m.get('usb_port', m.get('phys', ''))
    return [d for d in groups.values() if d['events']]


def select(device_id):
    if not re.fullmatch(r'[a-f0-9]{24}', device_id): raise ValueError('Select a keyboard in Settings')
    found = [d for d in discover() if d['id'] == device_id]
    if len(found) != 1: raise OSError('Selected keyboard disconnected; reconnect or choose a keyboard')
    return found[0]


def selected_hid(device):
    """Classify only hidraw interfaces belonging to the selected physical device."""
    result = {}
    for node in device['hidraw']:
        desc = (SYS / 'class/hidraw' / Path(node).name / 'device/report_descriptor').read_bytes()
        if desc.startswith(bytes.fromhex('0660ff0961')): result['vial'] = node
        elif desc.startswith(bytes.fromhex('05010906')): result['boot'] = node
        elif bytes.fromhex('8506050719e0') in desc: result['shared'] = node
    return result


if __name__ == '__main__':
    try: print(json.dumps({'ok': True, 'devices': discover()}))
    except Exception as exc: print(json.dumps({'ok': False, 'error': str(exc)})); sys.exit(1)
