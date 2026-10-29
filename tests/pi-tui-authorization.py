#!/usr/bin/env python3
"""Actual TUI denial/confirmation, checked against disposable canonical SQLite."""
import argparse,json,os,pty,selectors,signal,sqlite3,subprocess,time
from pathlib import Path
p=argparse.ArgumentParser()
for name in ('node','peers','state-root'):p.add_argument('--'+name,type=Path,required=True)
a=p.parse_args();root=a.state_root;assert str(root).startswith('/private/tmp/mirket-pi-package-')
env={**os.environ,'HOME':str(root/'home'),'PI_CODING_AGENT_DIR':str(root/'agent'),'PI_CODING_AGENT_SESSION_DIR':str(root/'sessions'),'PI_SKIP_UPDATE_CHECK':'1','PI_TELEMETRY_DISABLED':'1','TERM':'xterm-256color'}
subprocess.run([str(a.node),str(Path(__file__).with_name('pi-domain-session.mjs')),str(a.peers),str(root),'package'],env={**env,'MIRKET_PI_TUI_AUTH_SEED':'1'},check=True)
intent=json.loads((root/'tui-auth-intent.json').read_text())
prepared_file=root/('tui-auth-prepared-'+str(time.time_ns())+'.json')
provider_path=root/'provider-tui-auth.mjs'
provider_path.write_text('import fs from "node:fs"; import * as ai from '+json.dumps((a.peers/'@earendil-works/pi-ai/dist/index.js').as_uri())+'; import {makeFixtureProvider} from '+json.dumps(Path(__file__).with_name('pi-offline-provider.mjs').resolve().as_uri())+'; export default pi=>{makeFixtureProvider(ai)(pi);pi.on("tool_result",e=>{if(e.toolName==="mcp__mirket__memory_forget_prepare"&&!e.isError)fs.writeFileSync('+json.dumps(str(prepared_file))+',JSON.stringify(e.content));});};')

def exists():
 for file in (root/'agent/mirket/state/memory').glob('*.sqlite3'):
  with sqlite3.connect(file.as_uri()+'?mode=ro',uri=True) as db:
   if db.execute('select count(*) from records where record_id=?',(intent['recordId'],)).fetchone()[0]:return True
 return False
assert exists()
master,slave=pty.openpty();child=subprocess.Popen([str(a.node),str(a.peers/'@earendil-works/pi-coding-agent/dist/cli.js'),'--offline','--provider','offline-fixture','--model','deterministic','-e',str(provider_path)],cwd=root/'project',env=env,stdin=slave,stdout=slave,stderr=slave,start_new_session=True);os.close(slave)
sel=selectors.DefaultSelector();sel.register(master,selectors.EVENT_READ);transcript=bytearray();report={'recordId':intent['recordId'],'checks':[]}
def observe(seconds):
 end=time.monotonic()+seconds
 while time.monotonic()<end:
  if sel.select(min(.1,max(0,end-time.monotonic()))):
   try:chunk=os.read(master,65536)
   except OSError:return
   if not chunk:return
   transcript.extend(chunk)
def send(value):os.write(master,value if isinstance(value,bytes) else value.encode()+b'\r')
def until(text,start):
 end=time.monotonic()+15
 while text not in transcript[start:]:
  assert time.monotonic()<end,'Missing TUI state: '+repr(text)
  observe(.1)
try:
 observe(3)
 if b'Trust project folder?' in transcript:send(b'\x1b[B\x1b[B\r');observe(4)
 send(json.dumps({'tool':'tool_search','args':{'query':'memory_forget_prepare memory_forget_commit','limit':4}}));observe(3)
 send(json.dumps(intent['prepare']));deadline=time.monotonic()+10
 while not prepared_file.exists():
  assert time.monotonic()<deadline,'No same-connection prepare result'
  observe(.1)
 prepared=json.loads(next(c['text'] for c in json.loads(prepared_file.read_text()) if c['type']=='text'))
 intent['request']['args']['operation_id']=prepared['operation_id']
 observe(.5)

 start=len(transcript);send(json.dumps(intent['request']));until(b'Authorize Mirket operation?',start);send(b'\x1b');observe(2);assert exists();report['checks'].append('Escape denies actual confirmation; canonical record remains')
 start=len(transcript);send(json.dumps(intent['request']));until(b'Authorize Mirket operation?',start);send(b'\r');observe(3)
 assert not exists(),'Accepted confirmation did not forget canonical record';report['checks'].append('Enter accepts actual confirmation; canonical record removed')
 send(b'\x04');observe(2);child.wait(timeout=8);assert child.returncode==0;report['passed']=True
finally:
 if child.poll() is None:os.killpg(child.pid,signal.SIGKILL);child.wait()
 (root/('tui-authorization-transcript-'+str(time.time_ns())+'.txt')).write_bytes(transcript);(root/'tui-authorization-report.json').write_text(json.dumps(report,indent=2));sel.close();os.close(master)
print(json.dumps(report))
