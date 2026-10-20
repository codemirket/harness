#!/usr/bin/env python3
"""Synthetic collector → score lock → identity verifier → frozen analyzer. No models."""
import contextlib
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile

BASE = Path(__file__).resolve().parent


def module(name):
    spec = importlib.util.spec_from_file_location('preflight_' + name, BASE / (name + '.py'))
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def run_preflight():
    collector, packets, analyzer = (module(n) for n in ('prepare', 'make-judge-packets', 'analyze'))
    with tempfile.TemporaryDirectory(prefix='mirket-packet-preflight-') as directory, contextlib.redirect_stdout(io.StringIO()):
        root = Path(directory)
        for helper in (collector, packets, analyzer):
            helper.ROOT = root
        collector.select_experiment_root('synthetic')
        packets.select_locations('synthetic', None)
        exp = collector.EXP
        def put(name, value):
            path = exp / name
            collector.write(path, value)
            return collector.binding(path)
        def load(bound):
            return collector.read(collector.verify(bound))
        evidence = put('synthetic.json', {'synthetic_controls':True, 'model_executions':0})
        configs = [{'id':arm, 'file':put(f'{arm}.json', {'arm':arm})} for arm in collector.ARMS]
        cases, cells = [], []
        for n in range(6):
            case = {'id':f'SYNTHETIC_{n}', 'split':'held_out' if n < 4 else 'development',
                    'input':evidence, 'rubric':evidence, 'criteria':['guard']}
            cases.append(case)
            for arm in collector.ARMS:
                for repeat in (1, 2):
                    label = f'CELL_{len(cells):02}'
                    cells.append({'label':label, 'case_id':case['id'], 'configuration':arm, 'trial':repeat,
                                  'split':case['split'], 'artifact_dir':f'synthetic/artifacts/{label}',
                                  'observation_dir':f'synthetic/observations/{label}', 'prompt_binding':evidence,
                                  'receipt':f'synthetic/receipts/{label}.json', 'grade':f'synthetic/grades/{label}.json'})
        suite = put('suite.json', {'schema_version':1, 'trials_per_case':2, 'checker':evidence,
                                  'cases':cases, 'configurations':configs})
        initialization = {'schema_version':1, 'expected_cells':24, 'held_out_cells':16, 'development_cells':8,
                          'cases':cases, 'cells':cells, 'suite':suite, 'checker':evidence,
                          'protocol_bindings':[evidence], 'observer_bindings':[evidence], 'source_manifests':[put('source-manifest.json', {'files':[evidence]})]}
        init = put('initialization.json', initialization)
        put('freeze.json', initialization)
        oversized = exp / 'oversized-report.txt'
        oversized.write_bytes(b'x' * (1024 * 1024 + 1))
        try:
            collector.capture_attempt('CELL_00', 'oversized', oversized)
        except ValueError as error:
            analyzer.require('one MiB' in str(error), 'unexpected oversized-report refusal')
        else:
            raise AssertionError('oversized report accepted')
        for cell in cells:
            label = cell['label']
            report = put(f'worker-reports/{label}-failed.json', {'ok':False, 'synthetic':True})
            collector.capture_attempt(label, 'failed', collector.verify(report))
            # Distinct retry path: an overwritten failed report must never disappear.
            retry = put(f'worker-reports/{label}-retry.json', {'ok':True, 'synthetic':True})
            collector.capture_attempt(label, 'retry', collector.verify(retry))
            put(f'grades/{label}.json', {'reviewer_id':'SYNTHETIC_GRADER', 'case_id':cell['case_id'],
                 'verdicts':{'guard':label != 'CELL_02'}, 'rationale':'Synthetic independent guard observation.'})
            collector.seal(label, 'SYNTHETIC_WORKER', 'completed', None, 'Synthetic only')
        # Deliberately reproduce the historical overwrite before any collection.
        original = exp / 'worker-reports/CELL_00-failed.json'
        before = original.read_bytes()
        original.write_text('{"ok":true}')
        try:
            collector.collect()
        except ValueError as error:
            analyzer.require('Stale evidence' in str(error), 'unexpected collection refusal')
        else:
            raise AssertionError('collector accepted overwritten failed report')
        archived = collector.read(exp / 'attempts/CELL_00/failed.json')
        analyzer.require(json.loads(archived['raw_report'])['ok'] is False, 'failed report bytes lost')
        original.write_bytes(before)
        collector.collect()
        guards = collector.read(exp / 'report.json')
        analyzer.require(len(guards['trials']) == 24, 'collector lost planned trials')
        analyzer.require(sum(len(t['evidence']) >= 3 for t in guards['trials']) == 24, 'attempt evidence lost')
        try:
            collector.capture_attempt('CELL_00', 'late', original)
        except ValueError:
            pass
        else:
            raise AssertionError('attempt added after sealing')
        # Same production order, score forms, locking and post-lock merge serializers.
        orders = packets.make_orders([c for c in cells if c['split'] == 'held_out'])
        packets.verify_orders(orders, [c for c in cells if c['split'] == 'held_out'])
        identities = {'packets':[]}
        scores_by_judge = {}
        for judge in ('A','B'):
            packet_root = packets.SETUP / 'packets' / judge
            image_path = packet_root / 'synthetic.png'
            image_path.parent.mkdir(parents=True)
            # Original one-pixel PNG; no experiment artifact or semantic grade.
            import base64
            image_path.write_bytes(base64.b64decode('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jOAoAAAAASUVORK5CYII='))
            packet = {'schema_version':1, 'kind':'render_only_craft_packet', 'judge_id':packets.JUDGE_IDS[judge],
                      'human_acceptance':'not_collected', 'artifacts':[]}
            scores = {'schema_version':1, 'kind':'render_only_score_lock_input','locked':True,
                      'judge_id':packets.JUDGE_IDS[judge], 'human_acceptance':'not_collected', 'scores':{}}
            for row in [r for r in orders if r['judge'] == judge]:
                packet['artifacts'].append({'neutral_label':row['neutral_label'], 'presentation_order':row['presentation_order'],
                    'pair_position':row['pair_position'], 'views':[{'availability':'available','path':'synthetic.png'}]})
                cell = next(c for c in cells if c['label'] == row['cell_label'])
                score = 50 if cell['configuration'] == 'baseline' else (60 if judge == 'A' else 40)
                scores['scores'][row['neutral_label']] = {'presentation_order':row['presentation_order'],
                    'pair_position':row['pair_position'], 'render_only':True,'score_locked_before_guess':True,
                    'dimensions':{d:{'score':score,'evidence':['synthetic.png title region: synthetic observation one.',
                        'synthetic.png footer region: synthetic observation two.']} for d in packets.DIMENSIONS}}
            packets.save(packet_root / 'packet.json',packet)
            identities['packets'].append({'judge':judge,'files':[packets.binding(image_path), packets.binding(packet_root / 'packet.json')]})
            scores_by_judge[judge] = scores
        packets.save(packets.SETUP / 'render-identities.private.json',identities)
        judges = []
        for judge, scores in scores_by_judge.items():
            source = packets.SETUP / f'score-input-{judge}.json'
            packets.save(source,scores)
            packets.lock_scores(judge,source)
            try:
                packets.lock_scores(judge,source)
            except ValueError as error:
                analyzer.require('overwrite' in str(error), 'unexpected score-lock refusal')
            else:
                raise AssertionError('locked scores were replaceable')
            guesses = packets.load(packets.SETUP / f'guess-forms/{judge}.json')
            for guess in guesses['guesses'].values():
                guess['confidence'] = 0
            guess_path = packets.SETUP / f'guess-input-{judge}.json'
            packets.save(guess_path,guesses)
            packets.merge_guesses(judge,guess_path)
            judgment = packets.binding(packets.SETUP / f'judgments/{judge}.json')
            packets.verify(judgment)
            # Synthetic three-pair calibration uses the frozen analyzer's exact schema.
            ratings, pairs = [], []
            for index, (kind, values) in enumerate((('composition',(70,50)),('composition',(65,45)),('identical',(60,60)))):
                pair_id = f'PAIR_{index}'
                pair_ratings = []
                for position, score in enumerate(values):
                    label = f'CONTROL_{index}_{position}'
                    ratings.append({'pair_id':pair_id,'artifact_label':label,'dimensions':[
                        {'id':d,'score':score,'observations':[{'synthetic':1},{'synthetic':2}]} for d in analyzer.DIMENSIONS]})
                    pair_ratings.append({'label':label,'composite':score})
                pair = {'id':pair_id,'kind':kind,'family':f'FAMILY_{index}','ratings':pair_ratings,
                        'difference_first_minus_second':values[0]-values[1],'calibration_check_passed':True}
                pair['positive_minus_negative' if kind == 'composition' else 'absolute_difference'] = values[0]-values[1]
                pairs.append(pair)
            common = {'schema_version':1,'synthetic_controls':True,'judge_id':packets.JUDGE_IDS[judge],'packet_id':f'SYNTHETIC_{judge}'}
            calibration = put(f'calibration-{judge}.json',{**common,'kind':'synthetic_craft_calibration_judgment','model_executions':0,'ratings':ratings})
            validation = put(f'validation-{judge}.json',{**common,'kind':'synthetic_calibration_validation','control_model_executions':0,
                'judgment_sha256':calibration['sha256'],'calibrated':True,'pairs':pairs})
            judges.append({'id':judge,'judge_id':packets.JUDGE_IDS[judge],'judgment':judgment,
                           'calibration_attempts':[{'judgment':calibration,'validation':validation}]})
        rubric = put('rubric.json',{'schema_version':1,'scale':[0,100],'human_acceptance':'not_collected',
                                  'dimensions':[{'id':d,'weight':w} for d,w in analyzer.DIMENSIONS.items()]})
        manifest = {'schema_version':1,'kind':'design_quality_analysis_inputs','synthetic_controls':True,'model_executions':0,
                    'initialization':init,'craft_rubric':rubric,'guard_report':collector.binding(exp / 'report.json'),
                    'judges':judges,'label_map':orders,'source_bindings':[evidence],
                    'guard_consequences':[{'case_id':'SYNTHETIC_0','trial':1,'criterion':'guard','consequential':True,
                        'rationale':'Synthetic independent failed guard remains visible despite high judge A scores.','evidence':[evidence]}],
                    'integrity':{**{flag:True for flag in analyzer.INTEGRITY_FLAGS},'note':'Synthetic round trip only.','evidence':[evidence]}}
        bound = put('analysis-input.json',manifest)
        result = analyzer.analyze(collector.verify(bound))
        analyzer.require(len(result['raw_ratings']) == 32 and result['decision']['practical_win'] is False, 'ratings or contradictory outcomes lost')
        analyzer.require(len(result['guards']['all_24_guard_trials']) == 24, 'analyzer lost planned trials')
        broken = copy.deepcopy(manifest)
        broken['label_map'].pop()
        try:
            analyzer.analyze(collector.verify(put('missing-cell.json',broken)))
        except analyzer.InvalidInput:
            pass
        else:
            raise AssertionError('analyzer accepted a missing cell')
    return {'schema_version':1,'kind':'synthetic_packet_pipeline_preflight','ok':True,'model_executions':0,
            'planned_cells':24,'retained_attempts':48,'locked_ratings':32,'independent_briefs':4,
            'checks':['collector rejects overwritten failure','oversized report rejected before capture','captured failure bytes retained','sealed attempt set cannot grow',
                      'all planned cells survive production collector','production score lock and post-score merge','score-lock overwrite rejected',
                      'packet path and hash verification','frozen analyzer accepts round trip','missing cell rejected',
                      'contradictory judges and failed task guard prevent advancement'],
            'sources':{name:hashlib.sha256((BASE / name).read_bytes()).hexdigest() for name in
                       ('prepare.py','make-judge-packets.py','analyze.py','packet_preflight.py')},
            'limits':['Synthetic schema/identity regression only; no measured guidance benefit or human acceptance.']}


if __name__ == '__main__':
    print(json.dumps(run_preflight(), indent=2))
