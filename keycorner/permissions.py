"""Opt-in, device-scoped udev setup. No daemon or persistent privilege policy."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

RULE_DIR = Path('/etc/udev/rules.d')


def literal(value):
    # udev values are patterns and can expand substitutions. Reject these, not escape them.
    if not isinstance(value, str) or not value or len(value) > 256 or re.search(r'[\x00-\x1f\x7f"\\*?\[\]$%]', value):
        raise ValueError('Device identity contains unsupported rule characters')
    return value


def make_rule(device, backend):
    if backend not in ('standard', 'vial'): raise ValueError('Invalid backend')
    if not re.fullmatch('[a-f0-9]{24}', device['id']): raise ValueError('Invalid device identifier')
    match = device['match']
    if 'usb_port' in match:
        if not re.fullmatch(r'\d+-[\d.]+', match['usb_port']): raise ValueError('Invalid USB port')
        if not all(re.fullmatch('[a-fA-F0-9]{4}', match[k]) for k in ('vendor','product')): raise ValueError('Invalid USB IDs')
        selectors = [f'KERNELS=="{match["usb_port"]}"', f'ATTRS{{idVendor}}=="{match["vendor"]}"', f'ATTRS{{idProduct}}=="{match["product"]}"']
        if match.get('serial'): selectors.append(f'ATTRS{{serial}}=="{literal(match["serial"])}"')
    else:
        if backend == 'vial': raise ValueError('Vial access requires a supported USB keyboard')
        selectors = [f'ATTRS{{name}}=="{literal(match["name"])}"', f'ATTRS{{phys}}=="{literal(match["phys"])}"']
        if match.get('uniq'): selectors.append(f'ATTRS{{uniq}}=="{literal(match["uniq"])}"')
    if backend == 'vial':
        if (match.get('vendor'), match.get('product')) != ('3034','5659') or 'vial:' not in match.get('serial',''):
            raise ValueError('This Vial device is not supported yet; use Standard mode')
        prefix = ['SUBSYSTEM=="hidraw"', 'KERNEL=="hidraw*"']
    else:
        prefix = ['SUBSYSTEM=="input"', 'KERNEL=="event*"', 'ENV{ID_INPUT_KEYBOARD}=="1"']
    return '# Keycorner: selected keyboard only; active local session access.\n' + ', '.join(prefix + selectors + ['TAG+="uaccess"']) + '\n'


def destination(device, backend):
    make_rule(device, backend)  # Validate before deriving any filename.
    return RULE_DIR / f'69-keycorner-{device["id"]}-{backend}.rules'


def root_install(device, backend):
    if os.geteuid() != 0: raise PermissionError('Administrator authentication required')
    rule = make_rule(device, backend)
    path = destination(device, backend)
    nodes = device.get('nodes', [])
    expected = r'/dev/input/event\d+' if backend == 'standard' else r'/dev/hidraw\d+'
    if not nodes or any(not isinstance(node,str) or not re.fullmatch(expected,node) for node in nodes):
        raise ValueError('Invalid device nodes')
    # Atomic replacement cannot follow a destination symlink. Never edit vendor rules.
    fd, temporary = tempfile.mkstemp(prefix='.keycorner-', suffix='.rules', dir=RULE_DIR)
    try:
        with os.fdopen(fd, 'w') as output: output.write(rule)
        os.chmod(temporary, 0o644)
        subprocess.run(['/usr/bin/udevadm', 'verify', temporary], check=True, capture_output=True, text=True)
        os.replace(temporary, path)
    finally: Path(temporary).unlink(missing_ok=True)
    subprocess.run(['/usr/bin/udevadm', 'control', '--reload-rules'], check=True, timeout=15)
    # Apply only to nodes matching the selected rule, rather than triggering all devices.
    for node in nodes:
        subprocess.run(['/usr/bin/udevadm','trigger','--settle','--action=change',node], check=True, timeout=15)
    return str(path)


def main():
    action = sys.argv[1]
    if action == '--root-install':
        request = json.loads(sys.argv[2]); device, backend = request['device'], request['backend']
        return {'ok': True, 'path': root_install(device, backend)}
    # -I on the elevated path prevents loading user-site or PYTHONPATH code.
    # Discovery is only imported in the unprivileged branch.
    from keyboard_devices import select
    device, backend = select(sys.argv[2]), sys.argv[3]
    device['nodes'] = device['events'] if backend == 'standard' else device['hidraw']
    rule = make_rule(device, backend)
    path = destination(device, backend)
    if action == 'preview': return {'ok':True,'rule':rule,'path':str(path),'device':device['name']}
    if action != 'install': raise ValueError('Unknown permission action')
    request = json.dumps({'device':{'id':device['id'],'match':device['match'],'nodes':device['nodes']},'backend':backend})
    result = subprocess.run(['/usr/bin/pkexec','/usr/bin/python3','-I',str(Path(__file__).resolve()),'--root-install',request], capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError('Access setup cancelled or failed. Reconnect the keyboard if a rule was installed.')
    return {'ok':True,'message':'Access enabled. Reconnect the keyboard if access is still unavailable.','path':str(path)}


if __name__ == '__main__':
    try: print(json.dumps(main()))
    except Exception as error:
        print(json.dumps({'ok':False,'error':str(error)}));sys.exit(1)
