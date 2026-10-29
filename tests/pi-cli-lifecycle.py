#!/usr/bin/env python3
"""Actual offline unbundled pi CLI, RPC and PTY tests against an assembled fixture."""
import argparse, json, os, pty, selectors, signal, subprocess, time
from pathlib import Path

parser=argparse.ArgumentParser()
for name in ('node','peers','state-root'): parser.add_argument('--'+name,type=Path,required=True)
parser.add_argument('--only',choices=['all','tui'],default='all')
a=parser.parse_args();root=a.state_root;peers=a.peers
assert all(p.is_absolute() for p in (a.node,peers,root))
os.umask(0o077)
fixture=Path(__file__).with_name('pi-offline-provider.mjs').resolve()
provider_path=root/'provider.mjs';provider_path.write_text('import * as ai from '+json.dumps((peers/'@earendil-works/pi-ai/dist/index.js').as_uri())+';\nimport {makeFixtureProvider} from '+json.dumps(fixture.as_uri())+';\nexport default makeFixtureProvider(ai);\n')
cli=peers/'@earendil-works/pi-coding-agent/dist/cli.js'
env={**os.environ,'HOME':str(root/'home'),'PI_CODING_AGENT_DIR':str(root/'agent'),'PI_CODING_AGENT_SESSION_DIR':str(root/'sessions'),'PI_SKIP_UPDATE_CHECK':'1','PI_TELEMETRY_DISABLED':'1','TERM':'xterm-256color'}
base=[str(a.node),str(cli),'--offline','--provider','offline-fixture','--model','deterministic','-e',str(provider_path)]
report=json.loads((root/'cli-lifecycle-report.json').read_text()) if a.only=='tui' else {'entry':str(cli),'cases':[],'first_tool_ms':[]}
def save(name):
 report['cases'].append(name);(root/'cli-lifecycle-report.json').write_text(json.dumps(report,indent=2));print('PASS',name,flush=True)
if a.only != 'tui':
 for i in range(3):
  start=time.monotonic();p=subprocess.Popen([*base,'--mode','json','--print',json.dumps({'tool':'mirket_doctor'})],env=env,cwd=root/'project',stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
  lines=[];first=None
  for line in p.stdout:
   lines.append(line);event=json.loads(line)
   if event.get('type')=='tool_execution_end' and event.get('toolName')=='mirket_doctor':
    assert not event['isError'],event
    result=json.loads(event['result']['content'][0]['text']);assert result['status']=='ready',result
    first=(time.monotonic()-start)*1000
  p.wait(timeout=10);assert p.returncode==0 and first is not None,p.stderr.read()
  (root/f'cli-start-{i}.jsonl').write_text(''.join(lines));report['first_tool_ms'].append(first)
 save('actual unbundled CLI print mode, nested MCP doctor, three fresh processes')
 err=(root/'rpc.stderr').open('w');log=(root/'rpc-events.jsonl').open('w')
 p=subprocess.Popen([*base,'--mode','rpc'],cwd=root/'project',env=env,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err)
 sel=selectors.DefaultSelector();sel.register(p.stdout,selectors.EVENT_READ);buf=b'';events=[];seq=0
 def read(timeout=30):
  global buf
  end=time.monotonic()+timeout
  while b'\n' not in buf:
   if not sel.select(max(0,end-time.monotonic())):raise TimeoutError('RPC output')
   chunk=os.read(p.stdout.fileno(),65536)
   if not chunk:raise RuntimeError('RPC exited: '+(root/'rpc.stderr').read_text())
   buf+=chunk
  line,buf=buf.split(b'\n',1);event=json.loads(line);log.write(json.dumps(event)+'\n');log.flush();events.append(event);return event
 
 def request(typ,**fields):
  global seq
  seq+=1;ident=str(seq);p.stdin.write((json.dumps({'id':ident,'type':typ,**fields})+'\n').encode());p.stdin.flush()
  while True:
   e=read()
   if e.get('type')=='response' and e.get('id')==ident:
    assert e.get('success'),e
    return e.get('data')
 def prompt(message):
  n=len(events);reply=request('prompt',message=message)
  if reply.get('disposition')!='handled':
   while not any(e.get('type')=='agent_settled' for e in events[n:]):read()
  return events[n:]
 def doctor():
  results=prompt(json.dumps({'tool':'mirket_doctor'}));ends=[e for e in results if e.get('type')=='tool_execution_end' and e.get('toolName')=='mirket_doctor'];assert ends and not ends[-1]['isError'],ends
  assert json.loads(ends[-1]['result']['content'][0]['text'])['status']=='ready'
 try:
  before=request('get_state');doctor();prompt('/mcp');prompt('/reload');doctor()
  entries=request('get_fork_messages')['messages'];assert entries
  request('fork',entryId=entries[-1]['entryId']);doctor();after=request('get_state');assert before['sessionFile']!=after['sessionFile']
  request('switch_session',sessionPath=before['sessionFile']);doctor()
  request('prompt',message=json.dumps({'tool':'codemode','args':{'code':'while(true) {}'}}))
  while True:
   e=read()
   if e.get('type')=='tool_execution_start' and e.get('toolName')=='codemode':break
  request('abort');deadline=time.monotonic()+10
  while request('get_state').get('isStreaming'):
   assert time.monotonic()<deadline;time.sleep(.05)
  doctor();save('actual RPC reload, fork, resume, running codemode abort and recovery')
 finally:
  p.stdin.close()
  try:p.wait(timeout=8)
  except subprocess.TimeoutExpired:p.terminate();p.wait(timeout=5)
  log.close();err.close();sel.close()
  assert p.returncode==0,p.returncode
master,slave=pty.openpty();p=subprocess.Popen(base,cwd=root/'project',env=env,stdin=slave,stdout=slave,stderr=slave,start_new_session=True);os.close(slave)
sel=selectors.DefaultSelector();sel.register(master,selectors.EVENT_READ);transcript=bytearray()
def observe(seconds):
 end=time.monotonic()+seconds
 while time.monotonic()<end:
  if sel.select(min(.1,max(0,end-time.monotonic()))):
   try:chunk=os.read(master,65536)
   except OSError:return
   if not chunk:return
   transcript.extend(chunk)
try:
 observe(2)
 if b'Trust project folder?' in transcript:
  os.write(master,b'\x1b[B\x1b[B\r');observe(4)
 os.write(master,b'/mcp\r');observe(4)
 assert b'mirket' in transcript and b'25 tools' in transcript,'TUI did not expose loaded MCP'
 os.write(master,b'\x1b');observe(.5)
 os.write(master,b'/reload\r');observe(4);os.write(master,b'/mirket doctor\r');observe(2)
 os.write(master,b'\x04');observe(2);p.wait(timeout=8)
 assert p.returncode==0,p.returncode
 save('interactive PTY /mcp, reload, doctor and Ctrl-D shutdown; no pixel/image-delivery claim')
finally:
 if p.poll() is None:os.killpg(p.pid,signal.SIGKILL);p.wait()
 (root/f'tui-transcript-{time.time_ns()}.txt').write_bytes(transcript);sel.close();os.close(master)
report['original_p0_first_tool_budget_ms']=1000;report['first_tool_budget_ms']=1500;report['budget_basis']='User approved 1500 ms on 2026-10-29 to preserve full external host verification';report['startup_budget_pass']=all(t<=1500 for t in report['first_tool_ms'])
(root/'cli-lifecycle-report.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2))
