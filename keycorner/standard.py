"""Linux evdev reader and US ANSI presets. No grab, writes or third-party modules."""
import fcntl
import os
import struct

EVENT = struct.Struct('@llHHi')
KEY_BYTES = 96  # KEY_MAX=0x2ff; Linux input ABI
EVIOCGKEY = (2 << 30) | (KEY_BYTES << 16) | (ord('E') << 8) | 0x18


def key_state(fd):
    bits = bytearray(KEY_BYTES)
    fcntl.ioctl(fd, EVIOCGKEY, bits, True)
    return {i for i in range(KEY_BYTES * 8) if bits[i // 8] & (1 << (i % 8))}


class Reader:
    def __init__(self, fd):
        self.fd = fd
        self.held = key_state(fd)
        self.buffer = b''
        self.dropped = False

    def feed(self, data):
        self.buffer += data
        states = []
        while len(self.buffer) >= EVENT.size:
            _, _, kind, code, value = EVENT.unpack_from(self.buffer)
            self.buffer = self.buffer[EVENT.size:]
            if kind == 0 and code == 3:  # SYN_DROPPED
                self.dropped = True
                self.held.clear()
                states.append(set())
            elif kind == 0 and code == 0:
                if self.dropped:
                    self.held = key_state(self.fd)
                    self.dropped = False
                states.append(set(self.held))
            elif not self.dropped and kind == 1:
                if value == 1: self.held.add(code)
                elif value == 0: self.held.discard(code)
                # Repeat (2) does not change pressed state.
        return states


def layout(preset='ansi104'):
    if preset not in ('ansi104', 'ansi87', 'ansi60'): raise ValueError('Unsupported standard layout')
    keys = []
    def key(code, label, x, y, w=1, h=1):
        keys.append(dict(x=x, y=y, w=w, h=h, labels=[label], usages=[code], codes=[code]))
    def row(y, items):
        x = 0
        for code, label, width in items:
            key(code, label, x, y, width); x += width
    offset = 0 if preset == 'ansi60' else 1.5
    if offset:
        key(1, 'Esc', 0, 0)
        for i, code in enumerate([59,60,61,62,63,64,65,66,67,68,87,88]):
            key(code, 'F'+str(i+1), 2+i+(i//4)*.5, 0)
    row(offset, [(41,'`',1)]+[(i+2,str((i+1)%10),1) for i in range(10)]+[(12,'-',1),(13,'=',1),(14,'Bksp',2)])
    row(offset+1, [(15,'Tab',1.5)]+[(16+i,c,1) for i,c in enumerate('QWERTYUIOP')]+[(26,'[',1),(27,']',1),(43,'\\',1.5)])
    row(offset+2, [(58,'Caps',1.75)]+[(30+i,c,1) for i,c in enumerate('ASDFGHJKL')]+[(39,';',1),(40,"'",1),(28,'Enter',2.25)])
    row(offset+3, [(42,'Shift',2.25)]+[(44+i,c,1) for i,c in enumerate('ZXCVBNM')]+[(51,',',1),(52,'.',1),(53,'/',1),(54,'Shift',2.75)])
    row(offset+4, [(29,'Ctrl',1.25),(125,'Super',1.25),(56,'Alt',1.25),(57,'Space',6.25),(100,'Alt',1.25),(126,'Super',1.25),(127,'Menu',1.25),(97,'Ctrl',1.25)])
    if preset != 'ansi60':
        for code,label,x,y in [(99,'PrtSc',15.5,0),(70,'Scroll',16.5,0),(119,'Pause',17.5,0),
                              (110,'Ins',15.5,1.5),(102,'Home',16.5,1.5),(104,'PgUp',17.5,1.5),
                              (111,'Del',15.5,2.5),(107,'End',16.5,2.5),(109,'PgDn',17.5,2.5),
                              (103,'↑',16.5,4.5),(105,'←',15.5,5.5),(108,'↓',16.5,5.5),(106,'→',17.5,5.5)]: key(code,label,x,y)
    if preset == 'ansi104':
        for code,label,x,y,w,h in [(69,'Num',19,1.5,1,1),(98,'/',20,1.5,1,1),(55,'*',21,1.5,1,1),(74,'-',22,1.5,1,1),
            (71,'7',19,2.5,1,1),(72,'8',20,2.5,1,1),(73,'9',21,2.5,1,1),(78,'+',22,2.5,1,2),
            (75,'4',19,3.5,1,1),(76,'5',20,3.5,1,1),(77,'6',21,3.5,1,1),
            (79,'1',19,4.5,1,1),(80,'2',20,4.5,1,1),(81,'3',21,4.5,1,1),(96,'Enter',22,4.5,1,2),
            (82,'0',19,5.5,2,1),(83,'.',21,5.5,1,1)]: key(code,label,x,y,w,h)
    # 60% uses the standard grave position as Escape, with no firmware-layer inference.
    if preset == 'ansi60': keys[0].update(labels=['Esc'], usages=[1], codes=[1])
    return dict(name={'ansi104':'US ANSI 104','ansi87':'US ANSI TKL','ansi60':'US ANSI 60%'}[preset],
                layers=1, keys=keys, width=max(k['x']+k['w'] for k in keys),
                height=max(k['y']+k['h'] for k in keys), source='standard')
