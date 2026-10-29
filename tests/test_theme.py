import contextlib
import io
import json
from pathlib import Path
import runpy
import sys
import tempfile
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SOURCE / 'keycorner'))


class ThemeTests(unittest.TestCase):
    def test_theme_only_loads_colors_without_accessing_keyboard(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            theme = root / '.local/state/omarchy/current/theme/colors.toml'
            theme.parent.mkdir(parents=True)
            theme.write_text('background = "#123456"\naccent = "#abcdef"\n')
            output = io.StringIO()
            with patch('pathlib.Path.home', return_value=root), patch('core.devices', side_effect=AssertionError('HID accessed')) as devices, patch('sys.argv', ['bridge.py', '--theme-only']), contextlib.redirect_stdout(output):
                runpy.run_path(str(SOURCE / 'keycorner/bridge.py'), run_name='__main__')
            self.assertEqual(json.loads(output.getvalue())['theme']['background'], '#123456')
            devices.assert_not_called()
