import json,os,statistics,subprocess,sys,time
from pathlib import Path
import argparse
parser=argparse.ArgumentParser(description='Measure installed Keycorner using /proc counters; temporarily toggles visibility/pause and restores them. Does not record keystrokes. Output includes local process IDs.')
parser.add_argument('--output', type=Path, required=True, help='Directory for local measurement files (do not commit personal captures)')
parser.add_argument('--label', default='sample')
parser.add_argument('--seconds', type=int, default=15)
args=parser.parse_args()
if not 5 <= args.seconds <= 60: parser.error('--seconds must be 5..60')
if not args.label.replace('-', '').replace('_', '').isalnum(): parser.error('Use letters/numbers in the label')
label=args.label;duration=args.seconds
out=args.output;out.mkdir(parents=True,exist_ok=True)
hz=os.sysconf('SC_CLK_TCK')
def ipc(method,*args):return subprocess.check_output(['omarchy-shell','keycorner',method,*args],text=True)
def state():return json.loads(ipc('state'))
def pids():
 shell=[];bridge=[]
 for path in Path('/proc').iterdir():
  if not path.name.isdigit():continue
  try:args=(path/'cmdline').read_bytes().split(b'\0')
  except OSError:continue
  if any(x==b'/usr/share/omarchy/shell' for x in args) and b'quickshell' in args[0]:shell.append(int(path.name))
  if any(x.endswith(b'/lbiasuz.keycorner/keycorner/bridge.py') for x in args):bridge.append(int(path.name))
 return {'shell':shell,'bridge':bridge}
def sample(pid):
 root=Path('/proc')/str(pid)
 try:
  fields=(root/'stat').read_text().rsplit(')',1)[1].split()
  result={'cpu_seconds':(int(fields[11])+int(fields[12]))/hz}
  for line in (root/'smaps_rollup').read_text().splitlines():
   if line.split(':')[0] in ['Rss','Pss','Private_Clean','Private_Dirty']:result[line.split(':')[0]+'_KiB']=int(line.split()[1])
  status=(root/'status').read_text()
  for line in status.splitlines():
   if line.startswith(('voluntary_ctxt_switches:','nonvoluntary_ctxt_switches:')):result[line.split(':')[0]]=int(line.split()[1])
  try:
   for line in (root/'io').read_text().splitlines():result['io_'+line.split(':')[0]]=int(line.split()[1])
  except PermissionError:pass
  return result
 except (OSError,ProcessLookupError):return None
original=state()['config'];results=[]
try:
 for name,patch in [('hidden',{'shown':False,'paused':False}),('visible_capture',{'shown':True,'paused':False}),('visible_paused',{'shown':True,'paused':True}),('visible_capture_repeat',{'shown':True,'paused':False}),('hidden_repeat',{'shown':False,'paused':False})]:
  ipc('configure',json.dumps(patch));time.sleep(1.5)
  processes=pids();start=time.monotonic();samples=[]
  print('Measuring '+label+' '+name,flush=True)
  while time.monotonic()-start<duration:
   samples.append({'t':time.monotonic(),'processes':{str(pid):sample(pid) for group in processes.values() for pid in group}})
   time.sleep(1)
  last={'t':time.monotonic(),'processes':{str(pid):sample(pid) for group in processes.values() for pid in group}};samples.append(last)
  summaries={}
  for role,group in processes.items():
   for pid in group:
    values=[x['processes'][str(pid)] for x in samples if x['processes'].get(str(pid))]
    if not values:continue
    a,b=values[0],values[-1];elapsed=last['t']-samples[0]['t']
    summaries[role]={'pid':pid,'cpu_pct_one_core':100*(b['cpu_seconds']-a['cpu_seconds'])/elapsed,'median_pss_MiB':statistics.median(x.get('Pss_KiB',0) for x in values)/1024,'peak_rss_MiB':max(x.get('Rss_KiB',0) for x in values)/1024,'main_thread_voluntary_switches_per_s':(b['voluntary_ctxt_switches']-a['voluntary_ctxt_switches'])/elapsed,'io':{k:b[k]-a[k] for k in a if k.startswith('io_') and k in b}}
  row={'state':name,'duration_s':last['t']-samples[0]['t'],'processes':processes,'summaries':summaries,'capture':state()['capture']};results.append(row)
  (out/(label+'.json')).write_text(json.dumps({'label':label,'sample_duration_s':duration,'measurements':results},indent=2)+'\n')
  print(json.dumps(row),flush=True)
finally:
 ipc('configure',json.dumps({k:original[k] for k in ['shown','paused']}))
