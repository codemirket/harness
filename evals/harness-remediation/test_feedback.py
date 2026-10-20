"""Dependency-free discrimination checks on exact owned functions, not embeddings."""
import ast
import json
import time
import types
import unittest
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]

def function(file, name, namespace):
    tree = ast.parse((ROOT / file).read_text())
    body = next(n for n in ast.walk(tree) if isinstance(n, ast.FunctionDef) and n.name == name)
    exec(compile(ast.Module(body=[body], type_ignores=[]), str(file), 'exec'), namespace)
    return namespace[name]

class FeedbackTests(unittest.TestCase):
    def test_late_requirements_survive_and_explicit_queries_are_preserved(self):
        query = function('runtime/agent/workflow.py','recall_queries',{})
        tokenizer = types.SimpleNamespace(encode=lambda _: types.SimpleNamespace(overflowing=[]))
        prefix = 'Common boilerplate. ' * 40
        first, _ = query({'objective':prefix + 'Turkish offline behavior'}, tokenizer)
        second, _ = query({'objective':prefix + 'German financial reporting'}, tokenizer)
        self.assertNotEqual(first, second)
        self.assertTrue(any('Turkish offline behavior' in q for q in first))
        explicit = ['late requirement', 'German financial reporting']
        self.assertEqual(query({'objective':prefix,'memory_queries':explicit},tokenizer), (explicit,False))
        long, truncated = query({'objective': ''.join(str(i) + 'x'*255 for i in range(100))},tokenizer)
        self.assertLessEqual(len(long),8)
        self.assertTrue(truncated)
        needs = [{'reason': f'unique requirement {i}'} for i in range(8)]
        chosen, incomplete = query({'objective': 'short objective', 'task_needs': needs}, tokenizer)
        self.assertLessEqual(len(chosen), 8)
        self.assertTrue(incomplete, 'omitted task-need reasons must be disclosed')

    def test_pause_budget_distinguishes_safe_idle_from_uncertain_work(self):
        ns = {'MAX_PAUSE_SECONDS':86400}
        effective = function('runtime/agent/workflow.py','effective_deadline',ns)
        transition = function('runtime/agent/workflow.py','budget_transition',ns)
        state = {'deadline':1120}
        paused = {**state, **transition(state, 'worker_turn', {'decision':'pause'}, 1010)}
        resumed = transition(paused, 'worker_turn', {'decision':'continue'}, 2010)
        self.assertEqual(resumed, {'deadline':2120,'paused_at':0,'pause_used':1000})
        self.assertEqual(effective(resumed, 2015), 2120)
        with self.assertRaisesRegex(ValueError, 'budget expired'):
            transition(state, 'worker_turn', {'decision':'continue'}, 1120)
        with self.assertRaisesRegex(ValueError, 'only an active'):
            transition(state, 'effect', {'decision':'pause'}, 1010)
        with self.assertRaisesRegex(ValueError, 'only an active'):
            transition(paused, 'worker_turn', {'decision':'pause'}, 1020)
        with self.assertRaisesRegex(ValueError, 'pause allowance'):
            effective({**paused,'pause_used':86400},1011)
        with self.assertRaisesRegex(ValueError, 'clock moved backward'):
            effective(paused,1009)

    def test_empty_retrieval_cannot_pass_the_baseline(self):
        baseline = function('runtime/agent/test_runtime.py','test_retrieval_baseline_and_export_pagination',{'time':time,'json':json})
        class Empty(unittest.TestCase):
            def __init__(self): super().__init__(); self.ids=[]
            def call(self, op, args, **kwargs):
                if op=='memory_put':
                    ident=str(len(self.ids));self.ids.append(ident);return {'memory':{'id':ident}}
                if op=='data_export':
                    second='cursor' in args
                    return {'total':len(self.ids),'next_cursor':None if second else 'next','items':[{'value':{'id':x}} for x in (self.ids[20:] if second else self.ids[:20])]}
                if op=='memory_read': return {'results':[]}
                raise AssertionError(op)
        with self.assertRaisesRegex(AssertionError,'expected.*top three'):
            baseline(Empty())

if __name__ == '__main__': unittest.main()
