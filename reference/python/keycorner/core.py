"""Read-only Vial transport and layout model. No third-party dependencies."""
import json
import lzma
import math
import os
from pathlib import Path
import select
import struct

VID_PID = '00003034:00005659'


def devices():
    found = {}
    for node in Path('/sys/class/hidraw').glob('hidraw*'):
        info = (node / 'device/uevent').read_text()
        if VID_PID not in info:
            continue
        desc = (node / 'device/report_descriptor').read_bytes()
        path = '/dev/' + node.name
        if desc.startswith(bytes.fromhex('0660ff0961')):
            found['vial'] = path
        elif desc.startswith(bytes.fromhex('05010906')):
            found['boot'] = path
        elif bytes.fromhex('8506050719e0') in desc:
            found['shared'] = path
    return found


MAX_DEFINITION_BYTES = 1024 * 1024


def decode_definition(compressed):
    decoder = lzma.LZMADecompressor(memlimit=32 * 1024 * 1024)
    raw = decoder.decompress(compressed, max_length=MAX_DEFINITION_BYTES + 1)
    if len(raw) > MAX_DEFINITION_BYTES or not decoder.eof:
        raise ValueError('Keyboard definition exceeds size limit or is truncated')
    if decoder.unused_data:
        raise ValueError('Unexpected trailing compressed data')
    definition = json.loads(raw)
    if not isinstance(definition, dict):
        raise ValueError('Keyboard definition must be an object')
    return definition


class Vial:
    def __init__(self, path):
        self.fd = os.open(path, os.O_RDWR | os.O_NONBLOCK)

    def close(self):
        os.close(self.fd)

    def query(self, payload):
        # Only these read commands are permitted by this client.
        if not payload or len(payload) > 32:
            raise ValueError('Invalid request size')
        if payload[0] not in (0x01, 0x11, 0x12, 0xFE):
            raise ValueError('Not a read command')
        if payload[0] == 0xFE and (len(payload) < 2 or payload[1] not in (0, 1, 2)):
            raise ValueError('Not a read-only Vial command')
        os.write(self.fd, b'\0' + payload.ljust(32, b'\0'))
        if not select.select([self.fd], [], [], 1)[0]:
            raise TimeoutError('Keyboard did not answer; close Vial and reconnect')
        data = os.read(self.fd, 32)
        if len(data) != 32:
            raise OSError('Short keyboard response')
        return data

    def snapshot(self):
        identity = self.query(b'\xfe\x00')
        version = int.from_bytes(identity[:4], 'little')
        if not 0 <= version <= 6:
            raise ValueError('Unsupported Vial protocol')
        size = int.from_bytes(self.query(b'\xfe\x01')[:4], 'little')
        if not 0 < size <= 262144:
            raise ValueError('Invalid keyboard definition size')
        compressed = b''.join(self.query(struct.pack('<BBI', 254, 2, i))
                              for i in range((size + 31) // 32))[:size]
        definition = decode_definition(compressed)
        layers = self.query(b'\x11')[1]
        rows, cols = (definition['matrix'][k] for k in ('rows', 'cols'))
        if not (1 <= layers <= 32 and 1 <= rows <= 32 and 1 <= cols <= 32):
            raise ValueError('Invalid matrix dimensions')
        total = layers * rows * cols * 2
        if total > 65535:
            raise ValueError('Keymap too large')
        raw = b''.join(self.query(struct.pack('>BHB', 18, offset, min(28, total-offset)))[4:4+min(28, total-offset)]
                       for offset in range(0, total, 28))
        return {'definition': definition, 'layers': layers,
                'keycodes': list(struct.unpack('>' + str(total//2) + 'H', raw))}


BASIC = {0: '—', 1: '▽', **{i+4: chr(65+i) for i in range(26)},
         **{30+i: str((i+1) % 10) for i in range(10)},
         **{58+i: 'F'+str(i+1) for i in range(12)},
         **{104+i: 'F'+str(i+13) for i in range(12)}}
BASIC.update(dict(zip(range(40,58), ['Enter','Esc','Bksp','Tab','Space','-','=','[',']','\\','#',';',"'",'`',',','.','/','Caps'])))
BASIC.update({73:'Ins',74:'Home',75:'PgUp',76:'Del',77:'End',78:'PgDn',79:'→',80:'←',81:'↓',82:'↑',
              168:'Mute',169:'Vol+',170:'Vol−',171:'Next',172:'Prev',173:'Stop',174:'Play',
              209:'Btn1',210:'Btn2',211:'Btn3',224:'Ctrl',225:'Shift',226:'Alt',227:'Super',
              228:'RCtrl',229:'RShift',230:'AltGr',231:'RSuper'})
SHIFT = dict(zip("1234567890-=[]\\;'`,./", '!@#$%^&*()_+{}|:"~<>?'))
MODS = ['Ctrl', 'Shift', 'Alt', 'Super']


def label(code, custom=()):
    if code in BASIC:
        return BASIC[code]
    if 0x7e00 <= code < 0x7e00 + len(custom):
        return str(custom[code-0x7e00]['name'])[:64]
    if 0x4000 <= code <= 0x4fff:
        return f'{label(code & 255)}\nL{(code >> 8) & 15}'
    if 0x2000 <= code <= 0x3fff:
        mods = '+'.join(m for i,m in enumerate(MODS) if (code >> 8) & (1 << i))
        return f'{label(code & 255)}\n{mods}'
    if 0x100 <= code <= 0x1fff:
        base = label(code & 255)
        if code >> 8 == 2 and base in SHIFT:
            return SHIFT[base]
        mods = '+'.join(m for i,m in enumerate(MODS) if (code >> 8) & (1 << i))
        return mods + '+' + base
    for prefix, name in [(0x5200,'TO'),(0x5220,'MO'),(0x5240,'DF'),(0x5260,'TG'),(0x5280,'OSL'),(0x52c0,'TT')]:
        if prefix <= code < prefix+32:
            return f'{name} {code & 31}'
    if 0x5700 <= code <= 0x57ff:
        return f'TD {code & 255}'
    return f'{code:04X}'


def usage(code):
    if 0x100 <= code <= 0x4fff:
        return code & 255
    return code if 3 < code < 256 else None


def geometry(definition):
    keys = []
    y = 0.0
    raw_layout = definition['layouts']['keymap']
    if not isinstance(raw_layout, list) or len(raw_layout) > 128:
        raise ValueError('Invalid physical layout')
    for row in raw_layout:
        if not isinstance(row, list):
            continue
        if len(row) > 256:
            raise ValueError('Too many layout entries')
        x, w, h = 0.0, 1.0, 1.0
        for entry in row:
            if isinstance(entry, dict):
                if any(k in entry for k in ('r','rx','ry')):
                    raise ValueError('Rotated KLE layouts are not supported yet')
                for key in ('x', 'y', 'w', 'h'):
                    value = entry.get(key, 0)
                    if type(value) not in (int, float) or not math.isfinite(value) or abs(value) > 128:
                        raise ValueError('Invalid key geometry')
                x += entry.get('x',0); y += entry.get('y',0)
                w, h = entry.get('w',w), entry.get('h',h)
            else:
                if not isinstance(entry, str) or len(entry) > 256:
                    raise ValueError('Invalid matrix position')
                r,c = map(int, entry.split('\n')[0].split(','))
                rows, cols = (definition['matrix'][key] for key in ('rows', 'cols'))
                if not (0 <= r < rows and 0 <= c < cols):
                    raise ValueError('Key is outside the declared matrix')
                if not (0 < w <= 16 and 0 < h <= 16 and abs(x) <= 128 and abs(y) <= 128):
                    raise ValueError('Invalid key bounds')
                if len(keys) >= 1024:
                    raise ValueError('Too many keys')
                keys.append(dict(id=f'{r},{c}', row=r, col=c, x=x, y=y, w=w, h=h))
                x += w; w = h = 1.0
        y += 1
    if not keys:
        raise ValueError('Keyboard layout is empty')
    min_x = min(k['x'] for k in keys)
    min_y = min(k['y'] for k in keys)
    for key in keys:
        key['x'] -= min_x; key['y'] -= min_y
    return keys


def model(snapshot):
    d = snapshot['definition']; stride = d['matrix']['rows'] * d['matrix']['cols']
    keys = geometry(d)
    for k in keys:
        offset = k['row'] * d['matrix']['cols'] + k['col']
        codes = [snapshot['keycodes'][l*stride+offset] for l in range(snapshot['layers'])]
        # Preview a single overlay layer on the base; actual layer stack is unavailable.
        effective = [c if c != 1 else codes[0] for c in codes]
        k.update(labels=[label(c,d.get('customKeycodes',[])) for c in effective],
                 usages=[usage(c) for c in effective], codes=codes)
    return dict(name=d['name'], layers=snapshot['layers'], keys=keys,
                width=max(k['x']+k['w'] for k in keys), height=max(k['y']+k['h'] for k in keys))


def parse_report(data, kind):
    if kind == 'boot':
        if len(data) != 8: return None
        mods, codes = data[0], set(data[2:]) - {0,1,2,3}
    elif kind == 'shared' and data and data[0] == 6:
        if len(data) != 32: return None
        mods = data[1]
        codes = {i*8+b for i,byte in enumerate(data[2:]) for b in range(8) if byte & (1<<b)} - {0,1,2,3}
    elif kind == 'shared' and len(data) == 3 and data[0] == 4:
        media = {0xe2:168,0xe9:169,0xea:170,0xb5:171,0xb6:172,0xb7:173,0xcd:174}
        code = media.get(int.from_bytes(data[1:], 'little'))
        return ('media', {code} if code else set())
    else:
        return None
    codes.update(224+i for i in range(8) if mods & (1<<i))
    return ('keyboard', codes)


def infer_layer(layout, pressed):
    # Never guess from a modifier alone; duplicate keycodes cannot reveal layers.
    meaningful = pressed - set(range(224,232))
    if not meaningful: return None
    candidates = [l for l in range(layout['layers'])
                  if meaningful <= {k['usages'][l] for k in layout['keys']}]
    return candidates[0] if len(candidates) == 1 else None
