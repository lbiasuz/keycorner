#!/usr/bin/env python3
"""Install Keycorner or finish setup after `omarchy plugin add`. Never needs root."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

from keycorner.config_file import ensure_config

PLUGIN_ID = 'lbiasuz.keycorner'
RUNTIME_FILES = ['manifest.json', 'Overlay.qml', 'SettingsWindow.qml', 'BarWidget.qml',
                 'Config.js', 'README.md', 'LICENSE']
INCLUDE = 'require("hypr.keycorner")'
INCLUDE_BLOCK = '\n-- Floating Keycorner keyboard overlay.\n' + INCLUDE + '\n'


def copy_runtime(source, target):
    if source.resolve() == target.resolve():
        return
    target.mkdir(parents=True, exist_ok=True)
    for name in RUNTIME_FILES:
        shutil.copy2(source / name, target / name)
    shutil.copytree(source / 'keycorner', target / 'keycorner', dirs_exist_ok=True,
                    ignore=shutil.ignore_patterns('__pycache__'))


def atomic_write(path, data):
    temporary = path.with_name(path.name + f'.keycorner-tmp-{time.time_ns()}')
    try:
        temporary.write_bytes(data)
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def setup_hyprland(source, hypr_dir, validate):
    main = hypr_dir / 'hyprland.lua'
    if not main.exists():
        raise RuntimeError('Expected Omarchy Lua config at ' + str(main))
    destination = hypr_dir / 'keycorner.lua'
    old_main = main.read_bytes()
    old_rules = destination.read_bytes() if destination.exists() else None
    new_main = old_main if INCLUDE in old_main.decode() else old_main + INCLUDE_BLOCK.encode()
    # Preserve existing custom rules. Updates do not overwrite user modifications.
    new_rules = old_rules if old_rules is not None else (source / 'hyprland.lua').read_bytes()
    if old_main == new_main and old_rules == new_rules:
        validate()
        return
    stamp = time.time_ns()
    if old_main != new_main:
        shutil.copy2(main, main.with_name(f'hyprland.lua.keycorner-backup-{stamp}'))
    try:
        if old_rules is None:
            atomic_write(destination, new_rules)
        if old_main != new_main:
            atomic_write(main, new_main)
        validate()
    except Exception:
        atomic_write(main, old_main)
        if old_rules is None:
            destination.unlink(missing_ok=True)
        else:
            atomic_write(destination, old_rules)
        validate()
        raise


def setup_shortcut(source, hypr_dir, validate):
    """Install a separate, user-editable binding without replacing existing ones."""
    destination = hypr_dir / 'keycorner-shortcuts.lua'
    main = hypr_dir / 'bindings.lua'
    include = 'require("hypr.keycorner-shortcuts")'
    old = main.read_bytes() if main.exists() else b''
    if include in old.decode():
        return
    binds = json.loads(subprocess.check_output(['hyprctl', '-j', 'binds'], text=True))
    if any(b.get('modmask') == 69 and str(b.get('key', '')).upper() in ('K', 'O') for b in binds):
        raise RuntimeError('Super+Ctrl+Shift+K or O is already bound; choose another shortcut before installing')
    if destination.exists():
        raise RuntimeError('Unmanaged keycorner-shortcuts.lua exists; include it manually')
    stamp = time.time_ns()
    if main.exists():
        shutil.copy2(main, main.with_name(f'bindings.lua.keycorner-backup-{stamp}'))
    try:
        atomic_write(destination, (source / 'shortcuts.lua').read_bytes())
        atomic_write(main, old + ('\n-- Keycorner privacy toggle.\n' + include + '\n').encode())
        validate()
    except Exception:
        atomic_write(main, old)
        destination.unlink(missing_ok=True)
        validate()
        raise


def validate_hyprland():
    subprocess.run(['hyprctl', 'reload'], check=True)
    errors = subprocess.check_output(['hyprctl', 'configerrors'], text=True).strip()
    if errors and errors != 'ok':
        raise RuntimeError('Hyprland config errors: ' + errors)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--setup-only', action='store_true', help='Only create settings and floating-window rules')
    args = parser.parse_args()
    source = Path(__file__).resolve().parent
    config_root = Path(os.environ.get('XDG_CONFIG_HOME', Path.home() / '.config'))
    config_dir = config_root / 'omarchy'
    # Omarchy 4's plugin registry uses this fixed HOME path even with XDG_CONFIG_HOME set.
    target = Path.home() / '.config/omarchy/plugins' / PLUGIN_ID
    subprocess.run(['omarchy', 'plugin', 'validate', str(source)], check=True)
    if not args.setup_only:
        copy_runtime(source, target)
    ensure_config(config_dir / 'keycorner.json')
    setup_hyprland(source, config_root / 'hypr', validate_hyprland)
    setup_shortcut(source, config_root / 'hypr', validate_hyprland)
    if not args.setup_only:
        shell_config = Path.home() / '.config/omarchy/shell.json'
        if shell_config.exists():
            shutil.copy2(shell_config, shell_config.with_name(f'shell.json.keycorner-backup-{time.time_ns()}'))
        subprocess.run(['omarchy-shell', 'shell', 'rescanPlugins'], check=True)
        for _ in range(30):
            listing = subprocess.check_output(['omarchy-shell', 'shell', 'listPlugins'], text=True)
            if PLUGIN_ID in listing:
                break
            time.sleep(.2)
        else:
            raise RuntimeError('Plugin was not discovered; inspect Omarchy shell logs')
        subprocess.run(['omarchy', 'plugin', 'enable', PLUGIN_ID], check=True)
    print(f'Settings: {config_dir / "keycorner.json"}\nFloating rules: {config_root / "hypr/keycorner.lua"}')
    print('If updated QML is cached, run: omarchy restart shell')


if __name__ == '__main__':
    main()
