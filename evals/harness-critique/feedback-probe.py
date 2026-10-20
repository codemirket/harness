#!/usr/bin/env python3
"""Exercise exact owned evaluation/recall functions with controlled local fakes.

These are discriminating component probes, not Mem0 or container integration tests.
"""
import ast
import contextlib
import hashlib
import io
import json
from pathlib import Path
import tempfile
import time
import types
import unittest

ROOT=Path(__file__).resolve().parents[2]

def selected_function(path,name):
 text=path.read_text();tree=ast.parse(text)
 func=next(n for n in ast.walk(tree) if isinstance(n,ast.FunctionDef) and n.name==name)
 return compile(ast.Module(body=[func],type_ignores=[]),str(path),'exec')

class EmptySearch(unittest.TestCase):
 def __init__(self):
  super().__init__();self.ids=[];self.searches=0
 def call(self,operation,args,error=None):
  if error:
   assert operation=='data_export' and error=='changed';return {}
  if operation=='memory_put':
   ident=str(len(self.ids));self.ids.append(ident);return {'memory':{'id':ident}}
  if operation=='data_export':
   second='cursor' in args
   return {'total':len(self.ids),'next_cursor':None if second else 'second','items':[{'value':{'id':x}} for x in (self.ids[20:] if second else self.ids[:20])]}
  if operation=='memory_read':self.searches+=1;return {'results':[]}
  raise AssertionError(operation)

ns={'time':time,'json':json}
exec(selected_function(ROOT/'runtime/agent/test_runtime.py','test_retrieval_baseline_and_export_pagination'),ns)
fake=EmptySearch();out=io.StringIO()
with contextlib.redirect_stdout(out):ns['test_retrieval_baseline_and_export_pagination'](fake)
observations=json.loads(out.getvalue())
assert fake.searches==7
assert all(q['recall_at_3']==0 for q in observations['queries'] if q['expected'])

queries=[]
def recall(directory,scope,args):queries.append(args['query']);return {'results':[]}
ns={'recall':recall,'digest':lambda text:hashlib.sha256(text.encode()).hexdigest(), 'embedding_model':lambda:types.SimpleNamespace(model=types.SimpleNamespace(tokenizer=types.SimpleNamespace(encode=lambda text:types.SimpleNamespace(overflowing=[]))))}
exec(selected_function(ROOT/'runtime/agent/workflow.py','recall_node'),ns)
with tempfile.TemporaryDirectory(prefix='mirket-recall-probe-') as temp:
 p=Path(temp);(p/'vectors').mkdir()
 prefix='Common task boilerplate. '*20
 for ending in ['The critical task is Turkish offline behavior.','The critical task is German financial reporting.']:
  result=ns['recall_node']({'envelope':{'request':{'objective':prefix+ending,'memory_recall':'required'}}},p,'synthetic')
 assert queries[0]==queries[1] and len(queries[0])==256
result={'date':'2026-10-20','scope':'Exact owned function bodies executed with controlled memory/tokenizer fakes; no actual embedding or runtime claim.','empty_search_baseline':{'test_returned_normally':True,'search_calls':fake.searches,'observations':observations},'recall_prefix_collision':{'same_query':queries[0]==queries[1],'query_characters':len(queries[0]),'different_suffixes_ignored':True}}
(ROOT/'docs/harness-critique/feedback-probes.json').write_text(json.dumps(result,indent=2)+'\n')
print('Baseline test returns normally with six answerable queries at recall@3 = 0.')
print('Different task suffixes after character 256 produce identical recall queries.')
