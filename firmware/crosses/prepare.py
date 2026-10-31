#!/usr/bin/env python3
"""Prepare the pinned Crosses Rev2 source; never flash a device."""
import argparse,subprocess,shutil
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('source',type=Path);p.add_argument('--trackballs',choices=('right','left','both'),required=True);a=p.parse_args()
root=a.source.resolve();expected='6bfefe2057b7e2b4ea084ece7e64520aeb6b108c'
assert subprocess.check_output(['git','-C',str(root),'rev-parse','HEAD'],text=True).strip()==expected,'Unexpected manufacturer source commit'
def original(path):return subprocess.check_output(['git','-C',str(root),'show',f'{expected}:{path}'],text=True)
base='keyboards/gggw/crosses'
config=original(base+'/config.h')
if a.trackballs!='both':
 config=config.replace('#define POINTING_DEVICE_COMBINED','// #define POINTING_DEVICE_COMBINED')
 target='RIGHT' if a.trackballs=='right' else 'LEFT'
 config=config.replace('// #define POINTING_DEVICE_'+target,'#define POINTING_DEVICE_'+target)
(root/base/'config.h').write_text(config)
source=original(base+'/lib/crosses.c')
needle='void housekeeping_task_kb(void) {\n'
assert source.count(needle)==1
source=source.replace(needle,needle+'#ifdef KEYCORNER_LAYERS_ENABLE\n    housekeeping_task_user();\n#endif\n',1)
(root/base/'lib/crosses.c').write_text(source)
keymaps=root/base/'4x6/keymaps';dst=keymaps/'keycorner'
shutil.copytree(keymaps/'vial',dst,dirs_exist_ok=True)
shutil.copy(Path(__file__).resolve().parents[1]/'common/keycorner_layers.c',dst)
f=dst/'config.h';f.write_text(f.read_text()+'\n#define KEYCORNER_LAYERS_ENABLE\n')
f=dst/'rules.mk';f.write_text(f.read_text()+'\nSRC += keycorner_layers.c\n')
print('Prepared gggw/crosses/4x6/rev2:keycorner with',a.trackballs,'trackball configuration')
