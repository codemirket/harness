#!/usr/bin/env python3
"""Measure installed pi status-only idle and owned-process shutdown, offline."""
import argparse,json,os,selectors,subprocess,time
from pathlib import Path
p=argparse.ArgumentParser()
for name in ('node','peers','state-root'):p.add_argument('--'+name,type=Path,required=True)
p.add_argument('--payload',choices=['core','full'],default='core')
a=p.parse_args();root=a.state_root
rust_limit=40 if a.payload=='full' else 32
assert all(x.is_absolute() for x in (a.node,a.peers,root))
env={**os.environ,'HOME':str(root/'home'),'PI_CODING_AGENT_DIR':str(root/'agent'),'PI_CODING_AGENT_SESSION_DIR':str(root/'sessions'),'PI_SKIP_UPDATE_CHECK':'1','PI_TELEMETRY_DISABLED':'1'}
err=(root/'idle.stderr').open('w')
child=subprocess.Popen([str(a.node),str(a.peers/'@earendil-works/pi-coding-agent/dist/cli.js'),'--offline','--provider','offline-fixture','--model','deterministic','-e',str(root/'provider.mjs'),'--mode','rpc'],env=env,cwd=root/'project',stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err)
selector=selectors.DefaultSelector();selector.register(child.stdout,selectors.EVENT_READ)
buf=b'';report={'payload':a.payload,'samples':[],'budgets':{'aggregateMiB':450,'rustMiB':rust_limit,'growthMiB':16,'shutdownSeconds':3}}
def processes():
 rows=[]
 for line in subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,rss=,command='],text=True).splitlines():
  fields=line.split(None,3)
  if len(fields)==4:rows.append(dict(pid=int(fields[0]),ppid=int(fields[1]),rssKiB=int(fields[2]),command=fields[3]))
 return rows
def tree():
 rows=processes();ids={child.pid}
 while True:
  added={r['pid'] for r in rows if r['ppid'] in ids}-ids
  if not added:break
  ids.update(added)
 return [r for r in rows if r['pid'] in ids]
def save(): (root/'idle-report.json').write_text(json.dumps(report,indent=2)+'\n')
try:
 child.stdin.write((json.dumps({'id':'1','type':'prompt','message':json.dumps({'tool':'mirket_doctor'})})+'\n').encode());child.stdin.flush();deadline=time.monotonic()+30;ready=False
 while not ready:
  assert time.monotonic()<deadline,'Doctor timeout'
  if not selector.select(1):continue
  data=os.read(child.stdout.fileno(),65536);assert data,'Unexpected pi exit';buf+=data
  while b'\n' in buf:
   line,buf=buf.split(b'\n',1);event=json.loads(line)
   if event.get('type')=='tool_execution_end' and event.get('toolName')=='mirket_doctor':
    assert not event['isError'];assert json.loads(event['result']['content'][0]['text'])['status']=='ready';ready=True
 start=time.monotonic();owned=set()
 for seconds in [0,30,60,120]:
  time.sleep(max(0,seconds-(time.monotonic()-start)));rows=tree();owned.update(r['pid'] for r in rows)
  native=[r for r in rows if str(root/'agent/mirket/bin/mirket')+' serve' in r['command']]
  assert len(rows)==2 and len(native)==1,rows
  sample={'seconds':seconds,'observedSeconds':time.monotonic()-start,'tree':rows,'aggregateMiB':sum(r['rssKiB'] for r in rows)/1024,'rustMiB':native[0]['rssKiB']/1024};report['samples'].append(sample);save();print('SAMPLE',seconds,sample['aggregateMiB'],flush=True)
  # The frozen gate applies to warm idle at 30/60/120 seconds.
  # Preserve startup RSS as an observation, not an invented extra gate.
 assert report['samples'][-1]['aggregateMiB']-report['samples'][1]['aggregateMiB']<=16
 start=time.monotonic();child.stdin.close();child.wait(timeout=3)
 while any(r['pid'] in owned for r in processes()) and time.monotonic()-start<3:time.sleep(.05)
 report['remainingPids']=[r['pid'] for r in processes() if r['pid'] in owned];report['shutdownSeconds']=time.monotonic()-start
 assert not report['remainingPids'] and report['shutdownSeconds']<=3 and child.returncode==0
 report['budgetFailures']=[sample for sample in report['samples'] if sample['seconds']>=30 and (sample['aggregateMiB']>450 or sample['rustMiB']>rust_limit)]
 save();assert not report['budgetFailures'],report['budgetFailures']
 report['passed']=True;save();print('PASS idle and shutdown budgets',flush=True)
finally:
 if child.poll() is None:child.terminate();child.wait(timeout=5)
 selector.close();err.close()
