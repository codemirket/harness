#!/usr/bin/env python3
"""Actual offline unbundled pi CLI, RPC and PTY tests against an assembled fixture."""
import argparse, errno, json, math, os, pty, re, selectors, signal, subprocess, time
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
  with (root/f'cli-start-{i}.stderr').open('w') as err, (root/f'cli-start-{i}.jsonl').open('w') as raw:
   start=time.monotonic();p=subprocess.Popen([*base,'--mode','json','--print',json.dumps({'tool':'mirket_doctor'})],env=env,cwd=root/'project',stdout=subprocess.PIPE,stderr=err,text=True)
   first=None
   for line in p.stdout:
    raw.write(line);raw.flush();event=json.loads(line)
    if event.get('type')=='tool_execution_end' and event.get('toolName')=='mirket_doctor':
     assert not event['isError'],event
     result=json.loads(event['result']['content'][0]['text']);assert result['status']=='ready',result
     first=(time.monotonic()-start)*1000
   p.wait(timeout=10);assert p.returncode==0 and first is not None,(root/f'cli-start-{i}.stderr').read_text()
  report['first_tool_ms'].append(first)
  (root/'cli-lifecycle-report.json').write_text(json.dumps(report,indent=2))
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
transcript_path=root/f'tui-transcript-{time.time_ns()}.txt'
terminal_escape=re.compile(rb'\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07]*(?:\x07|\x1b\\))')
def terminal_text(start):
 # Match visible text across ANSI styling and terminal-width wrapping; retain raw bytes separately.
 return b' '.join(terminal_escape.sub(b'',bytes(transcript[start:])).split())
def receive(timeout):
 if not sel.select(timeout):return True
 try:chunk=os.read(master,65536)
 except OSError as error:
  if error.errno!=errno.EIO:raise
  return False
 if not chunk:return False
 transcript.extend(chunk);return True
def until(description,predicate,start=0,timeout=15,raw=False):
 observed=lambda:bytes(transcript[start:]) if raw else terminal_text(start)
 end=time.monotonic()+timeout
 while not predicate(observed()):
  remaining=end-time.monotonic()
  if remaining<=0:
   raise TimeoutError(f'TUI timed out waiting for {description}; transcript: {transcript_path}; tail: {terminal_text(start)[-1000:]!r}')
  if not receive(min(.1,remaining)) or p.poll() is not None:
   if predicate(observed()):return
   raise RuntimeError(f'TUI exited while waiting for {description} (returncode={p.poll()}); transcript: {transcript_path}; tail: {terminal_text(start)[-1000:]!r}')
try:
 until('project trust prompt or loaded extensions',lambda text:b'Trust project folder?' in text or b'[Extensions]' in text)
 if b'Trust project folder?' in terminal_text(0):
  start=len(transcript);os.write(master,b'\x1b[B\x1b[B\r')
  until('loaded extensions after session-only trust',lambda text:b'[Extensions]' in text,start)
 start=len(transcript);os.write(master,b'/mcp\r')
 until('connected Mirket MCP with 25 tools',lambda text:b'mirket' in text and b'connected' in text and b'25 tools' in text,start)
 start=len(transcript);os.write(master,b'\x1b')
 # Pi renders the focused empty editor as a reverse-video blank, or shows its hardware cursor.
 until('focused editor after MCP menu dismissal',lambda raw:b'\x1b[7m \x1b[27m' in raw or b'\x1b[?25h' in raw,start,raw=True)
 start=len(transcript);os.write(master,b'/reload\r')
 until('reload completion',lambda text:b'Reloaded keybindings, extensions, skills, prompts, themes, and context files' in text,start)
 start=len(transcript);os.write(master,b'/mirket doctor\r')
 until('doctor report completion',lambda text:b'"model": "host-owned"' in text and b'"nativeOnly": false' in text,start)
 os.write(master,b'\x04');deadline=time.monotonic()+8
 while p.poll() is None:
  if time.monotonic()>=deadline:raise TimeoutError(f'TUI Ctrl-D shutdown timed out; transcript: {transcript_path}')
  if not receive(.1):p.wait(timeout=max(.001,deadline-time.monotonic()))
 assert p.returncode==0,p.returncode
 save('interactive PTY /mcp, reload, doctor and Ctrl-D shutdown; no pixel/image-delivery claim')
finally:
 if p.poll() is None:os.killpg(p.pid,signal.SIGKILL);p.wait()
 while sel.select(0):
  if not receive(0):break
 transcript_path.write_bytes(transcript);sel.close();os.close(master)
report['original_p0_first_tool_budget_ms']=1000
report['prior_first_tool_budget_ms']=1500;report.pop('first_tool_budget_ms',None)
report['startup_budget_ms']={'first_observed_process':5000,'repeat_process':2500}
report['startup_sample_definition']='Three fresh processes in order: first observed process, then two repeats in the same disposable installation. OS caches are not flushed; this is not disk-cold or percentile evidence.'
report['budget_basis']='User authorized startup gate recalibration on 2026-10-30; 5000 ms first-process and 2500 ms repeat-process limits preserve full host and native identity verification.'
report['startup_budget_results']=[]
for i,t in enumerate(report['first_tool_ms']):
 phase='first_observed_process' if i==0 else 'repeat_process';budget=report['startup_budget_ms'][phase]
 passed=isinstance(t,(int,float)) and not isinstance(t,bool) and math.isfinite(t) and 0<=t<=budget
 report['startup_budget_results'].append({'sample':i,'phase':phase,'budget_ms':budget,'pass':passed})
report['startup_budget_pass']=len(report['first_tool_ms'])==3 and all(sample['pass'] for sample in report['startup_budget_results'])
report['prior_1500_ms_budget_pass']=report['startup_budget_pass'] and all(t<=1500 for t in report['first_tool_ms'])
(root/'cli-lifecycle-report.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2))

if not report['startup_budget_pass']:
 raise SystemExit('FAIL startup budget: require three valid first-tool observations, first process <=5000 ms and each repeat <=2500 ms; functional report and transcripts retained')
