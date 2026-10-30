"""Refresh synthetic golden cases from the unchanged Python reference (development only)."""
import hashlib, json, lzma, random, sys
from pathlib import Path
root=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(root/'keycorner'))
import core, shortcuts, permissions, standard
for preset in ("ansi104", "ansi87", "ansi60"):
 assert json.loads((root/"data"/(preset+".json")).read_text()) == standard.layout(preset), preset
random.seed(414)
d={'name':'Synthetic','matrix':{'rows':2,'cols':2},'layouts':{'keymap':[[{'x':1},'0,0','0,1'],[{'x':2,'y':-.5},'1,0','1,1']]}}
codes=[4,5,6,7,1,0,80,81]
custom=[{'name':'Custom'}, {'name':'⌨'*90}]
h=hashlib.sha256()
for code in range(65536):
 h.update(f'{code}:{core.label(code,custom)}:{core.usage(code) if core.usage(code) is not None else "null"}\n'.encode())
reports=[]
for kind, size in [('boot',8),('shared',32),('shared',3),('boot',7),('shared',4)]:
 for _ in range(100):
  raw=bytes(random.randrange(256) for i in range(size))
  if kind=='shared':raw=bytes([6 if size==32 else 4])+raw[1:]
  parsed=core.parse_report(raw,kind)
  reports.append({'kind':kind,'data':list(raw),'expected':[parsed[0],sorted(parsed[1])] if parsed else None})
valid=lzma.compress(json.dumps(d).encode())
device={'id':'a'*24,'match':{'usb_port':'1-6','vendor':'3034','product':'5659','serial':'vial:synthetic'}}
fixture={'labels_sha256':h.hexdigest(),'custom':custom,'definition':d,'codes':codes,'model':core.model({'definition':d,'layers':2,'keycodes':codes}),'reports':reports,'compressed':list(valid),'rules':{b:permissions.make_rule(device,b) for b in ('standard','vial')},'device':device,'shortcuts':shortcuts.DEFAULTS,'rendered_shortcuts':shortcuts.render(shortcuts.DEFAULTS)}
encoded=json.dumps(fixture,ensure_ascii=False,separators=(',',':'))+'\n'
if '--check' in sys.argv:
 assert (root/'tests/rust-golden.json').read_text()==encoded, 'Python reference parity fixtures changed'
 print('Python golden fixtures and all standard preset coordinates match.')
else:
 (root/'tests/rust-golden.json').write_text(encoded)
