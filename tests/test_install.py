import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from install import copy_runtime, setup_hyprland, setup_shortcut, INCLUDE
from keycorner.config_file import ensure_config


class InstallTests(unittest.TestCase):
    def test_first_run_preserves_settings_even_if_invalid(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'omarchy/keycorner.json'
            self.assertTrue(ensure_config(path))
            self.assertEqual(json.loads(path.read_text()), {'version': 1, 'backend': 'standard'})
            path.write_text('broken user configuration')
            self.assertFalse(ensure_config(path))
            self.assertEqual(path.read_text(), 'broken user configuration')

    def test_setup_is_idempotent_and_preserves_custom_rules(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp); source=root/'source'; hypr=root/'hypr'
            source.mkdir(); hypr.mkdir()
            (source/'hyprland.lua').write_text('-- default rule')
            (hypr/'hyprland.lua').write_text('-- existing user config\n')
            setup_hyprland(source,hypr,lambda:None)
            self.assertIn(INCLUDE,(hypr/'hyprland.lua').read_text())
            (hypr/'keycorner.lua').write_text('-- customized')
            setup_hyprland(source,hypr,lambda:None)
            self.assertEqual((hypr/'hyprland.lua').read_text().count(INCLUDE),1)
            self.assertEqual((hypr/'keycorner.lua').read_text(),'-- customized')

    def test_validation_failure_restores_original_config(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp); source=root/'source'; hypr=root/'hypr'
            source.mkdir(); hypr.mkdir()
            (source/'hyprland.lua').write_text('-- rule')
            original='-- user configuration\n'
            (hypr/'hyprland.lua').write_text(original)
            calls=[]
            def validate():
                calls.append(True)
                if len(calls)==1: raise RuntimeError('Invalid rule')
            with self.assertRaisesRegex(RuntimeError,'Invalid rule'):
                setup_hyprland(source,hypr,validate)
            self.assertEqual((hypr/'hyprland.lua').read_text(),original)
            self.assertFalse((hypr/'keycorner.lua').exists())
            self.assertEqual(len(calls),2)

    def test_install_from_plugin_directory_is_safe(self):
        with tempfile.TemporaryDirectory() as tmp:
            source=Path(tmp)
            (source/'sentinel').write_text('existing checkout')
            copy_runtime(source,source)
            self.assertEqual((source/'sentinel').read_text(),'existing checkout')


class ShortcutTests(unittest.TestCase):
    def test_conflicting_binding_leaves_files_untouched(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'bindings.lua').write_text('-- user bindings')
            with patch('install.subprocess.check_output', return_value='[{"modmask":69,"key":"O"}]'):
                with self.assertRaisesRegex(RuntimeError, 'already bound'):
                    setup_shortcut(root, root, lambda: None)
            self.assertEqual((root / 'bindings.lua').read_text(), '-- user bindings')
            self.assertFalse((root / 'keycorner-shortcuts.lua').exists())

    def test_shortcuts_preserve_customization_and_roll_back_failure(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'bindings.lua').write_text('-- user bindings')
            (root / 'shortcuts.lua').write_text('-- default')
            with patch('install.subprocess.check_output', return_value='[]'):
                with self.assertRaises(RuntimeError):
                    setup_shortcut(root, root, lambda: (_ for _ in ()).throw(RuntimeError('invalid')))
                self.assertEqual((root / 'bindings.lua').read_text(), '-- user bindings')
                self.assertFalse((root / 'keycorner-shortcuts.lua').exists())
                setup_shortcut(root, root, lambda: None)
                (root / 'keycorner-shortcuts.lua').write_text('-- customized')
                setup_shortcut(root, root, lambda: None)
                self.assertEqual((root / 'keycorner-shortcuts.lua').read_text(), '-- customized')
                self.assertEqual((root / 'bindings.lua').read_text().count('require('), 1)
