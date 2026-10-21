#!/usr/bin/env python3
"""Local synthetic routing probes. No external inference, credentials or user projects."""
import argparse
import contextlib
from datetime import date
import http.server
import json
import os
from pathlib import Path
import subprocess
import tempfile
import threading

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser()
parser.add_argument('--binary', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
args = parser.parse_args()
binary = args.binary.resolve()
records = []
posts = []
mode = 'valid'
mode_calls = 0
models = [
 {'id':'fixture-mini','displayName':'fixture-mini','description':'fast utility','supports_tools':True,'supports_structured_outputs':True,'reasoning_efforts':['high'],'context_length':262144,'max_completion_tokens':64000},
 {'id':'fixture-frontier','displayName':'fixture-frontier','description':'frontier reasoning model','supports_tools':True,'supports_structured_outputs':True,'reasoning_efforts':['high'],'context_length':262144,'max_completion_tokens':64000},
]
class Server(http.server.BaseHTTPRequestHandler):
 def log_message(self,*args): pass
 def reply(self,value):
  body=json.dumps(value).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
 def do_GET(self):
  if self.path!='/v1/models':raise ValueError(self.path)
  self.reply({'data':models})
 def do_POST(self):
  global mode_calls
  mode_calls += 1
  payload=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
  posts.append({'path':self.path,'model':payload['model']})
  action={'action':'complete','tool':'','arguments_json':'{}','result':'Synthetic answer: 42.'}
  if mode=='large_source':
   action={'action':'tool','tool':'workspace_read','arguments_json':json.dumps({'path':'large.txt'}),'result':''} if mode_calls==1 else {'action':'blocked','tool':'','arguments_json':'{}','result':'Cannot read source with the assigned tool.'}
  self.reply({'model':payload['model'],'choices':[{'finish_reason':'stop','message':{'content':'not-json' if mode=='invalid_json' else json.dumps(action)}}], 'usage':{'prompt_tokens':1234,'completion_tokens':56,'total_tokens':1290}})
server=http.server.ThreadingHTTPServer(('127.0.0.1',0),Server)
thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
try:
 with tempfile.TemporaryDirectory(prefix='mirket-critique-') as temp:
  home=Path(temp).resolve();project=home/'project';project.mkdir();(project/'input.txt').write_text('synthetic evidence\n');(home/'.mirket').mkdir()
  # Explicit unavailable native targets suppress ambient PATH/desktop discovery.
  targets=[{'name':'fixture','protocol':'openai_chat','endpoint':f'http://127.0.0.1:{server.server_port}/v1','credential_env':None,'executable':None}]
  targets += [{'name':p+'-disabled','protocol':p,'endpoint':None,'credential_env':None,'executable':'/usr/bin/false'} for p in ['codex','claude']]
  (home/'.mirket/routing.json').write_text(json.dumps({'ttl_seconds':300,'targets':targets}))
  env={k:v for k,v in os.environ.items() if not k.startswith(('OPENAI_','ANTHROPIC_','OPENROUTER_','OLLAMA_'))}
  env['MIRKET_DIRECT']='1'
  def run(case,request,command):
   path=home/'request.json';path.write_text(json.dumps(request));start=len(posts)
   p=subprocess.run([str(binary),'--home',str(home)]+command+['--request',str(path)],env=env,capture_output=True,text=True,timeout=30)
   r={'case':case,'exit_code':p.returncode,'stdout':p.stdout,'stderr':p.stderr,'posts':posts[start:]}
   try:r['result']=json.loads(r.pop('stdout'))
   except ValueError:pass
   records.append(r);return r
  base={'objective':'Synthesize the supplied evidence: the answer is 42.','task_intent':'synthesis','complexity_class':'synthesis','assigned_tools':[],'task_needs':[{'workflow':'analysis','reason':'Synthesize supplied synthetic evidence.'}],'project':str(project)}
  editable={**base,'task_needs':[{'workflow':'editable-interface','reason':'Produce an editable design.'}],'design_input':{'mode':'native_specimen','medium':'design_document','reason':'An editable canvas is the target medium.'}}
  r=run('editable-interface-route',editable,['models','route'])
  assert r['exit_code']!=0 and not r['posts']
  assert 'run mirket setup first' in r['stderr']
  assert 'Penpot' not in r['stderr']
  mode='invalid_json'
  r=run('invalid-json-trace',base,['agent','spawn'])
  assert r['exit_code']!=0
  traces=list((home/'.mirket/runs').glob('*/trace.json'));assert len(traces)==1
  trace=json.loads(traces[0].read_text());r['trace']=trace
  r['specific_error_retained']=any(s in json.dumps(trace) for s in ['expected ident','invalid action schema','not-json'])
  mode='valid'
  r=run('provider-owned-usage',base,['agent','spawn'])
  assert r['exit_code']==0
  trace=json.loads(Path(r['result']['trace']).read_text());r['trace']=trace
  r['reported_usage_retained']='prompt_tokens' in json.dumps(trace) or 'total_tokens' in json.dumps(trace)
  r=run('required-memory-without-runtime',{**base,'memory_recall':'required'},['agent','spawn'])
  assert r['exit_code']!=0 and not r['posts']
  assert 'required memory recall' in r['stderr'].lower()
  mechanical={**base,'objective':'Sort lines','task_intent':'file_hygiene','complexity_class':'deterministic','deterministic_operation':{'action':'sort_lines'},'deterministic_input':'b\na','task_needs':[{'workflow':'structured-response','reason':'Sort supplied text with no filesystem effects.'}]}
  r=run('mechanical-model-ranking',mechanical,['models','route'])
  assert r['exit_code']==0
  assert r['result']['resolution']['min_tier']=='T1_DETERMINISTIC'
  assert len(r['result']['candidates'])==2 and r['result']['candidates'][0]['tier']=='T3_ARCHITECT'
  mode='large_source';mode_calls=0
  (project/'large.txt').write_text('x'*(128*1024+1))
  r=run('source-over-read-limit',base,['agent','spawn'])
  assert r['exit_code']!=0
  trace=max((home/'.mirket/runs').glob('*/trace.json'),key=lambda p:p.stat().st_mtime_ns)
  r['trace']=json.loads(trace.read_text())
  # Synthetic disposable paths are redacted after all dependent reads.
  data=json.dumps({'date':date.today().isoformat(),'binary':str(binary),'scope':'Synthetic localhost server, disposable home/project, explicit unavailable native targets, no external inference.','records':records},indent=2).replace(str(home),'<disposable-home>')
  args.output.write_text(data+'\n')
  for r in records:print(r['case'],r['exit_code'],{k:v for k,v in r.items() if k.endswith('_retained')})
finally:
 server.shutdown();server.server_close();thread.join(timeout=5)
