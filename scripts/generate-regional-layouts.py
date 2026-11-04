"""Generate fixed ISO full-size/TKL presets; no runtime XKB dependency.

Base legends follow xkeyboard-config symbols/{gb,de,fr,es,latam,it,pt}
(basic variants) and keycodes/evdev. Enter uses a nonoverlapping rectangle.
Run from any directory; review generated JSON before committing.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGIONS = {
    'gb': ('UK', {43:'#', 86:'\\'}),
    'de': ('German QWERTZ', {41:'^',12:'ß',13:'´',21:'Z',26:'Ü',27:'+',39:'Ö',40:'Ä',43:'#',44:'Y',53:'-',83:','}),
    'fr': ('French AZERTY (traditional)', {41:'²',2:'&',3:'é',4:'"',5:"'",6:'(',7:'-',8:'è',9:'_',10:'ç',11:'à',12:')',16:'A',17:'Z',26:'^',27:'$',30:'Q',39:'M',40:'ù',43:'*',44:'W',50:',',51:';',52:':',53:'!'}),
    'es': ('Spanish (Spain)', {41:'º',12:"'",13:'¡',26:'`',27:'+',39:'Ñ',40:'´',43:'Ç',53:'-'}),
    'latam': ('Spanish (Latin America)', {41:'|',12:"'",13:'¿',26:'´',27:'+',39:'Ñ',40:'{',43:'}',53:'-'}),
    'it': ('Italian', {41:'\\',12:"'",13:'ì',26:'è',27:'+',39:'ò',40:'à',43:'ù',53:'-'}),
    'pt': ('Portuguese (Portugal)', {41:'\\',12:"'",13:'«',26:'+',27:'´',39:'Ç',40:'º',43:'~',53:'-'}),
}

for region, (name, labels) in REGIONS.items():
    for suffix, base in [('105','ansi104'), ('88','ansi87')]:
        layout = json.loads((ROOT / f'data/{base}.json').read_text())
        layout['name'] = f'{name} ISO ' + ('105' if suffix == '105' else 'TKL 88')
        keys = {k['codes'][0]:k for k in layout['keys']}
        keys[43].update(x=12.75,y=3.5,w=1)
        keys[28].update(x=13.75,y=2.5,w=1.25,h=2)
        keys[42]['w'] = 1.25
        extra = dict(x=1.25,y=4.5,w=1,h=1,labels=['<'],codes=[86],usages=[86])
        layout['keys'].append(extra)
        keys[86] = extra
        keys[100]['labels'] = ['AltGr']
        for code, label in labels.items():
            if code in keys:
                keys[code]['labels'] = [label]
        (ROOT / f'data/{region}{suffix}.json').write_text(json.dumps(layout,ensure_ascii=False,separators=(',',':'))+'\n')
