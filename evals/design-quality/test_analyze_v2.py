#!/usr/bin/env python3
"""Synthetic compatibility probes; no experiment scores or model executions."""

import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile

sys.dont_write_bytecode = True
import analyze_v2 as compatibility

core = compatibility.core


def self_test():
    checks = []
    core.AUDIT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='synthetic-five-pair-', dir=core.AUDIT) as directory:
        folder = Path(directory)
        serial = [0]

        def put(name, value):
            serial[0] += 1
            path = folder / f'{serial[0]}-{name}.json'
            path.write_text(json.dumps(value, allow_nan=False), encoding='utf-8')
            return core.binding(path)

        def dimensions(score):
            return [{'id': d, 'score': score, 'observations': [
                {'image': 'SYNTHETIC/image.png', 'region': 'Synthetic title region', 'claim': 'Synthetic observation one, never model evidence.'},
                {'image': 'SYNTHETIC/image.png', 'region': 'Synthetic detail region', 'claim': 'Synthetic observation two, never model evidence.'}]} for d in core.DIMENSIONS]

        source = {'schema_version': 1, 'kind': 'synthetic_craft_calibration_judgment',
                  'synthetic_controls': True, 'model_executions': 0, 'human_acceptance': 'not_collected',
                  'judge_id': 'SYNTHETIC_COMPAT_JUDGE', 'packet_id': 'SYNTHETIC_FIVE_PAIR_PACKET', 'ratings': []}
        validation = {'schema_version': 1, 'kind': 'synthetic_calibration_validation',
                      'synthetic_controls': True, 'control_model_executions': 0,
                      'judge_id': source['judge_id'], 'packet_id': source['packet_id'],
                      'calibrated': True, 'pairs': [], 'label_order_consistency': [
                          {'synthetic_controls': True, 'interpretation': 'Retain complete metadata, including descriptive relationships.'}],
                      'limits': ['Synthetic structural control; no human acceptance.']}
        for index, (kind, family, scores) in enumerate((
                ('composition', 'SYNTHETIC_FAMILY_1', (76, 53)),
                ('content-order', 'SYNTHETIC_FAMILY_1', (54, 82)),
                ('composition', 'SYNTHETIC_FAMILY_2', (81, 62)),
                ('polished-false-state', 'SYNTHETIC_FAMILY_1', (91, 57)),
                ('identical', 'SYNTHETIC_FAMILY_1', (74, 78)))):
            pair = {'id': f'SYNTHETIC_P{index}', 'kind': kind, 'family': family,
                    'ratings': [], 'difference_first_minus_second': scores[0] - scores[1]}
            for position, score in enumerate(scores, 1):
                label = f'SYNTHETIC_L{index}_{position}'
                source['ratings'].append({'pair_id': pair['id'], 'artifact_label': label,
                                          'presentation_position': position, 'dimensions': dimensions(score)})
                pair['ratings'].append({'pair_id': pair['id'], 'label': label, 'presentation_position': position,
                                        'composite': score, 'dimensions': [{'id': d, 'score': score} for d in core.DIMENSIONS]})
            if kind == 'composition':
                pair.update(positive_minus_negative=scores[0] - scores[1], calibration_check_passed=True)
            elif kind == 'identical':
                pair.update(absolute_difference=abs(scores[0] - scores[1]), calibration_check_passed=True)
            validation['pairs'].append(pair)

        def attempt(src, val):
            source_binding = put('source', src)
            val = copy.deepcopy(val); val['judgment_sha256'] = source_binding['sha256']
            return {'judgment': source_binding, 'validation': put('validation', val)}

        def run(src=None, val=None, actual=None):
            return compatibility.validate_calibration([attempt(src or source, val or validation)], actual or source['judge_id'])

        def check(name, truth):
            core.require(truth, f'compatibility self-test failed: {name}')
            checks.append({'name': name, 'passed': True})

        def refuses(name, callback):
            try:
                callback()
            except core.InvalidInput:
                checks.append({'name': name, 'passed': True})
            else:
                raise core.InvalidInput(f'compatibility self-test failed to refuse: {name}')

        original_attempt = attempt(source, validation)
        refuses('frozen original refusal reproduced on realistic five-pair synthetic schema',
                lambda: compatibility.ORIGINAL_CALIBRATION([original_attempt], source['judge_id']))
        complete = compatibility.validate_calibration([original_attempt], source['judge_id'])
        retained = complete['attempts'][0]
        check('all five pairs/ten original ratings retained', complete['calibrated']
              and len(retained['retained_validation']['pairs']) == 5 and retained['complete_source_rating_count'] == 10)
        check('entire original validator result retained without filtering',
              retained['retained_validation'] == core.bound_json(original_attempt['validation']))
        check('both descriptive controls explicitly reconciled without a pass criterion',
              {p['kind'] for p in retained['descriptive_pair_reconciliation']} == compatibility.DESCRIPTIVE_KINDS)
        check('scoped bound JSON loader restored after validation', core.bound_json is compatibility.ORIGINAL_BOUND_JSON)

        def changed(kind, transform_source=None, transform_validation=None):
            src, val = copy.deepcopy(source), copy.deepcopy(validation)
            pair = next(p for p in val['pairs'] if p['kind'] == kind)
            if transform_source:
                transform_source(src, pair)
            if transform_validation:
                transform_validation(val, pair)
            return src, val

        for kind in sorted(compatibility.DESCRIPTIVE_KINDS):
            for flag in (True, False):
                src, val = changed(kind, transform_validation=lambda v, p, flag=flag: p.update(calibration_check_passed=flag))
                refuses(f'{kind} invented pass flag {flag} refused', lambda src=src, val=val: run(src, val))
        for name, transform in (
                ('unknown descriptive kind', lambda v, p: p.update(kind='future-unapproved-kind')),
                ('mismatched pair delta', lambda v, p: p.update(difference_first_minus_second=0)),
                ('mismatched composite', lambda v, p: p['ratings'][0].update(composite=55)),
                ('invented threshold on descriptive control', lambda v, p: p.update(positive_minus_negative=99)),
                ('wrong pair identity', lambda v, p: p['ratings'][0].update(pair_id='SYNTHETIC_OTHER')),
                ('wrong artifact identity', lambda v, p: p['ratings'][0].update(label='SYNTHETIC_ABSENT')),
                ('wrong presentation position', lambda v, p: p['ratings'][0].update(presentation_position=2)),
                ('mismatched dimension score', lambda v, p: p['ratings'][0]['dimensions'][0].update(score=1))):
            src, val = changed('content-order', transform_validation=transform)
            refuses(name, lambda src=src, val=val: run(src, val))
        src, val = copy.deepcopy(source), copy.deepcopy(validation)
        src['ratings'][1] = copy.deepcopy(src['ratings'][0])
        refuses('duplicated original rating identity', lambda: run(src, val))
        src, val = copy.deepcopy(source), copy.deepcopy(validation)
        src['ratings'].pop()
        refuses('missing one of ten original ratings', lambda: run(src, val))
        src, val = copy.deepcopy(source), copy.deepcopy(validation)
        removed = next(p for p in val['pairs'] if p['kind'] == 'content-order')
        val['pairs'].remove(removed); src['ratings'] = [r for r in src['ratings'] if r['pair_id'] != removed['id']]
        refuses('dropping a descriptive pair and its ratings', lambda: run(src, val))
        refuses('wrong actual judge identity', lambda: run(actual='SYNTHETIC_WRONG_JUDGE'))
        src, val = copy.deepcopy(source), copy.deepcopy(validation)
        val['calibrated'] = False
        refuses('invented overall calibration failure flag', lambda: run(src, val))
        src, val = copy.deepcopy(source), copy.deepcopy(validation)
        pair = next(p for p in val['pairs'] if p['kind'] == 'composition')
        pair.update(positive_minus_negative=5, difference_first_minus_second=5, calibration_check_passed=False)
        pair['ratings'][1].update(composite=71, dimensions=[{'id': d, 'score': 71} for d in core.DIMENSIONS])
        for rating in src['ratings']:
            if rating['pair_id'] == pair['id'] and rating['presentation_position'] == 2:
                rating['dimensions'] = dimensions(71)
        val['calibrated'] = False
        failure = run(src, val)
        check('real checked contrast failure retained while descriptive controls remain ungraded',
              not failure['calibrated'] and len(failure['attempts'][0]['retained_validation']['pairs']) == 5)
        check('frozen original analysis source unchanged', hashlib.sha256(compatibility.CORE_PATH.read_bytes()).hexdigest()
              == compatibility.CORE_SHA256)
    original = core.self_test()
    core.require(original['passed'] and original['check_count'] == 39, 'unchanged original 39 controls must pass through compatibility')
    return {'schema_version': 1, 'kind': 'design_quality_calibration_compatibility_self_test',
            'synthetic_controls': True, 'model_executions': 0, 'human_acceptance': 'not_collected',
            'passed': True, 'focused_check_count': len(checks), 'focused_checks': checks,
            'original_controls_through_compatibility': original,
            'interpretation': 'Realistic five-pair synthetic schema controls only; no real heldout scores, renders or model executions were used.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        result = self_test()
        core.write_new(args.output, result)
        print(json.dumps({'result': core.binding(args.output), 'passed': result['passed'],
                          'focused_check_count': result['focused_check_count'], 'original_check_count': 39,
                          'synthetic_controls': True, 'model_executions': 0}))
    except (core.InvalidInput, KeyError, TypeError, AttributeError, OSError) as exc:
        parser.exit(2, f'Compatibility self-test refused: {exc}\n')


if __name__ == '__main__':
    main()
