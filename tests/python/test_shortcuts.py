from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from keycorner import shortcuts as s


class ShortcutTests(unittest.TestCase):
    def test_normalization_and_injection_rejection(self):
        self.assertEqual(s.normalize('shift + super + p'), 'SUPER + SHIFT + P')
        self.assertEqual(s.normalize('ctrl + Return'), 'CTRL + Return')
        self.assertEqual(s.normalize(''), '')
        for value in ['P', 'CTRL + CTRL + P', 'SUPER + "); os.execute("bad', 'CTRL + SHIFT', 5]:
            with self.assertRaises(ValueError): s.normalize(value)

    def test_conflicts_and_swapping_owned_combinations(self):
        binds = [{'modmask':69, 'key':'K', 'description':s.ACTIONS['toggle']}]
        swapped = dict(s.DEFAULTS, toggle=s.DEFAULTS['pause'], pause=s.DEFAULTS['toggle'])
        s.check_conflicts(swapped, s.DEFAULTS, binds)
        with self.assertRaisesRegex(ValueError, 'already used'):
            s.check_conflicts(s.DEFAULTS, {}, binds)
        with self.assertRaisesRegex(ValueError, 'different shortcut'):
            s.check_conflicts(dict(s.DEFAULTS, pause=s.DEFAULTS['toggle']), {}, [])

    def test_disabled_shortcuts_roundtrip_and_custom_lua_protection(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'shortcuts.lua'
            values = dict(s.DEFAULTS, pause='')
            path.write_text(s.render(values))
            self.assertEqual(s.read(path), values)
            path.write_text('print("custom configuration")')
            with self.assertRaisesRegex(ValueError, 'custom Lua'): s.read(path)

    def test_apply_rolls_back_when_hyprland_rejects_key(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'shortcuts.lua'
            original = s.render(s.DEFAULTS); path.write_text(original)
            with patch.object(s, 'run', return_value='[]'), patch.object(s, 'validate', side_effect=[ValueError('bad key'), None]):
                with self.assertRaisesRegex(ValueError, 'bad key'):
                    s.apply(path, dict(s.DEFAULTS, pause='CTRL + NotARealKey'))
            self.assertEqual(path.read_text(), original)
            self.assertEqual(path.with_suffix('.lua.backup').read_text(), original)

    def test_apply_disabling_and_rebinding(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'shortcuts.lua'; path.write_text(s.render(s.DEFAULTS))
            values = dict(s.DEFAULTS, pause='', toggle='SUPER + CTRL + F12')
            with patch.object(s, 'run', return_value='[]'), patch.object(s, 'validate') as validate:
                self.assertEqual(s.apply(path, values), values)
                validate.assert_called_once()
            self.assertEqual(s.read(path), values)
