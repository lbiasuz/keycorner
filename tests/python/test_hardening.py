import json
import lzma
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'reference/python/keycorner'))
from core import decode_definition, geometry, MAX_DEFINITION_BYTES, Vial
from bridge import state_message


class HardeningTests(unittest.TestCase):
    def test_definition_round_trip(self):
        source={'name':'Test','matrix':{'rows':1,'cols':1}}
        self.assertEqual(decode_definition(lzma.compress(json.dumps(source).encode())),source)

    def test_expansion_bomb_is_rejected(self):
        compressed=lzma.compress(b'x'*(MAX_DEFINITION_BYTES+1))
        self.assertLess(len(compressed),10000)
        with self.assertRaises(ValueError):decode_definition(compressed)

    def test_truncated_and_concatenated_streams_rejected(self):
        data=lzma.compress(b'{}')
        for invalid in [data[:-2],data+data]:
            with self.assertRaises(ValueError):decode_definition(invalid)

    def test_untrusted_geometry_rejected(self):
        def fixture(row):return {'matrix':{'rows':1,'cols':1},'layouts':{'keymap':[row]}}
        for row in [['-1,0'],['0,1'],[{'w':0},'0,0'],[{'x':float('inf')},'0,0'],[{'w':'huge'},'0,0'],[]]:
            with self.subTest(row=row),self.assertRaises(ValueError):geometry(fixture(row))

    def test_missing_command_rejected_before_device_access(self):
        transport=Vial.__new__(Vial)
        for data in [b'',b'\xfe',b'\x01'*33]:
            with self.assertRaises(ValueError):transport.query(data)

    def test_state_contains_no_human_readable_typed_text(self):
        layout={'layers':1,'keys':[{'usages':[4]}]}
        message=state_message(layout,{'keyboard':{4},'media':set()},False)
        self.assertEqual(message,{'type':'state','pressed':[4],'estimate':None})
