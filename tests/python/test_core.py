import sys
from pathlib import Path
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'reference/python/keycorner'))
from core import Vial, model, label, parse_report, infer_layer, geometry

class KeyboardTests(unittest.TestCase):
    def test_read_only_transport_guard(self):
        vial = Vial.__new__(Vial)
        for command in (b'\x05', b'\xfe\x06', b'\xfe\x08', b'\x13'):
            with self.assertRaises(ValueError): vial.query(command)

    def test_boot_press_and_release(self):
        self.assertEqual(parse_report(bytes([2,0,4,5,0,0,0,0]),'boot'), ('keyboard',{225,4,5}))
        self.assertEqual(parse_report(bytes(8),'boot'), ('keyboard',set()))

    def test_nkro(self):
        packet = bytearray(32); packet[0]=6; packet[1]=1; packet[2]=1<<4; packet[12]=1
        self.assertEqual(parse_report(packet,'shared'), ('keyboard',{224,4,80}))
        self.assertIsNone(parse_report(b'\x06\x00','shared'))
        self.assertIsNone(parse_report(b'\x02'+bytes(10),'shared'))

    def test_media_release(self):
        self.assertEqual(parse_report(b'\x04\xe9\x00','shared'), ('media',{169}))
        self.assertEqual(parse_report(b'\x04\x00\x00','shared'), ('media',set()))

    def test_kle_and_transparency(self):
        fixture = {'definition':{'name':'Test','matrix':{'rows':2,'cols':2},
                     'layouts':{'keymap':[[{'x':1},'0,0','0,1'],[{'x':2,'y':-.5},'1,0','1,1']]}},
                   'layers':2,'keycodes':[4,5,6,7,1,0,80,81]}
        layout = model(fixture)
        self.assertEqual(layout['keys'][0]['labels'], ['A','A'])
        self.assertEqual(layout['keys'][1]['usages'], [5,None])
        self.assertEqual(layout['keys'][2]['y'], .5)
        self.assertEqual(infer_layer(layout,{80}),1)
        self.assertIsNone(infer_layer(layout,{4}))
        self.assertIsNone(infer_layer(layout,{224}))
        self.assertIsNone(infer_layer(layout,{99}))

    def test_key_labels(self):
        self.assertEqual(label(0x5221),'MO 1')
        self.assertEqual(label(0x52c5),'TT 5')
        self.assertEqual(label(0x2207),'D\nShift')
        self.assertEqual(label(0x4217),'T\nL2')
        self.assertEqual(label(0x0222),'%')
        self.assertEqual(label(0x7e00,[{'name':'M-Inc'}]),'M-Inc')

if __name__ == '__main__': unittest.main()
