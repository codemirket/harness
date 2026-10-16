#!/usr/bin/env python3
"""Versioned calibration compatibility for the unchanged design-quality analysis."""

import argparse
import copy
from fractions import Fraction
import hashlib
import importlib.util
from pathlib import Path
import sys

sys.dont_write_bytecode = True
CORE_PATH = Path(__file__).with_name('analyze.py')
CORE_SHA256 = 'a5e7d03fc92f602a14da4b79cf2b3091183e7601906d2d0b65b277cbe68dfad6'
if hashlib.sha256(CORE_PATH.read_bytes()).hexdigest() != CORE_SHA256:
    raise RuntimeError('Frozen original analysis bytes changed; compatibility method refused')
SPEC = importlib.util.spec_from_file_location('design_quality_analysis_frozen_v1', CORE_PATH)
core = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(core)
ORIGINAL_CALIBRATION = core.validate_calibration
ORIGINAL_BOUND_JSON = core.bound_json
DESCRIPTIVE_KINDS = {'content-order', 'polished-false-state'}
METHOD_VERSION = 'calibration-compatibility-v2'


def _binding_key(value):
    return value['path'], value['sha256']


def _reconcile_complete_five_pairs(judgment, validation):
    """Check all ten ratings before deriving the unchanged threshold-only view."""
    pairs, ratings = validation.get('pairs'), judgment.get('ratings')
    core.require(isinstance(pairs, list) and all(isinstance(p, dict) for p in pairs), 'calibration pair objects required')
    core.require(isinstance(ratings, list) and all(isinstance(r, dict) for r in ratings), 'calibration rating objects required')
    kinds = [p.get('kind') for p in pairs]
    allowed = {'composition', 'identical'} | DESCRIPTIVE_KINDS
    core.require(all(kind in allowed for kind in kinds), 'unknown calibration pair kind')
    if not (set(kinds) & DESCRIPTIVE_KINDS):
        return None
    core.require(len(pairs) == 5 and len(ratings) == 10 and kinds.count('composition') == 2
                 and kinds.count('identical') == 1 and all(kinds.count(k) == 1 for k in DESCRIPTIVE_KINDS),
                 'extended calibration requires the complete fixed five pairs/ten ratings')
    core.unique([p.get('id') for p in pairs], 'complete calibration pair IDs')
    source = {}
    for rating in ratings:
        identity = (rating.get('pair_id'), rating.get('artifact_label'))
        core.require(all(isinstance(s, str) and s.strip() for s in identity) and identity not in source,
                     'duplicate/absent complete calibration rating identity')
        values, composite = core.dimension_scores(rating.get('dimensions'), f'complete calibration/{identity}', observations=True)
        core.integer(rating.get('presentation_position'), 'calibration source presentation_position', 1, 2)
        source[identity] = (rating, values, composite)
    seen, descriptive = set(), []
    for pair in pairs:
        core.text_value(pair.get('id'), 'calibration pair ID')
        core.text_value(pair.get('family'), 'calibration pair family')
        pair_ratings = pair.get('ratings')
        core.require(isinstance(pair_ratings, list) and len(pair_ratings) == 2
                     and all(isinstance(r, dict) for r in pair_ratings), 'complete paired calibration ratings required')
        composites = []
        for position, rating in enumerate(pair_ratings, 1):
            identity = (pair['id'], rating.get('label'))
            core.require(identity in source and identity not in seen, 'complete calibration validator identity mismatch')
            original, values, composite = source[identity]
            core.require(rating.get('pair_id') == pair['id'], 'calibration rating pair_id mismatch')
            core.integer(rating.get('presentation_position'), 'calibration validation presentation_position', 1, 2)
            core.require(rating['presentation_position'] == position == original['presentation_position'],
                         'calibration source/validation presentation position mismatch')
            reported = rating.get('dimensions')
            core.require(isinstance(reported, list) and len(reported) == 5
                         and all(isinstance(d, dict) for d in reported), 'validation dimension scores required')
            core.unique([d.get('id') for d in reported], 'validation dimension IDs')
            core.require({d.get('id') for d in reported} == set(core.DIMENSIONS), 'validation dimension IDs differ')
            for dimension in reported:
                score = core.number(dimension.get('score'), 'validation dimension score')
                core.require(Fraction(str(score)) == Fraction(str(values[dimension['id']])),
                             'validation dimension differs from original rating')
            reported_composite = core.number(rating.get('composite'), 'complete calibration composite')
            core.require(core.close(reported_composite, composite), 'complete calibration composite differs from source')
            composites.append(Fraction(str(reported_composite))); seen.add(identity)
        difference = composites[0] - composites[1]
        core.require(core.close(core.number(pair.get('difference_first_minus_second'), 'complete calibration delta', -100, 100), difference),
                     'complete calibration pair delta does not reconcile')
        if pair['kind'] in DESCRIPTIVE_KINDS:
            core.require(set(pair) == {'id', 'kind', 'family', 'ratings', 'difference_first_minus_second'},
                         'descriptive calibration pair must have no invented flags or thresholds')
            descriptive.append({'id': pair['id'], 'kind': pair['kind'], 'rating_identities': [
                [pair['id'], rating['label']] for rating in pair_ratings],
                'use': 'Descriptive only; no calibration pass condition and no treatment-estimate contribution.'})
    core.require(seen == set(source), 'original calibration ratings missing from complete validation')
    checked_ids = {p['id'] for p in pairs if p['kind'] not in DESCRIPTIVE_KINDS}
    projected_judgment = copy.deepcopy(judgment)
    projected_judgment['ratings'] = [r for r in projected_judgment['ratings'] if r['pair_id'] in checked_ids]
    projected_validation = copy.deepcopy(validation)
    projected_validation['pairs'] = [p for p in projected_validation['pairs'] if p['id'] in checked_ids]
    return projected_judgment, projected_validation, descriptive


def validate_calibration(attempts, judge_id):
    """Reuse original fixed checks; retain complete bound originals in the result."""
    core.require(isinstance(attempts, list) and 1 <= len(attempts) <= 2, 'one calibration plus at most one retained retry')
    projections, complete = {}, []
    for attempt in attempts:
        core.require(isinstance(attempt, dict), 'calibration attempt object required')
        judgment = ORIGINAL_BOUND_JSON(attempt.get('judgment'))
        validation = ORIGINAL_BOUND_JSON(attempt.get('validation'))
        reconciled = _reconcile_complete_five_pairs(judgment, validation)
        if reconciled is not None:
            projected_judgment, projected_validation, descriptive = reconciled
            projections[_binding_key(attempt['judgment'])] = projected_judgment
            projections[_binding_key(attempt['validation'])] = projected_validation
        else:
            descriptive = []
        complete.append((validation, descriptive))

    def threshold_view(value):
        identity = _binding_key(value)
        return copy.deepcopy(projections[identity]) if identity in projections else ORIGINAL_BOUND_JSON(value)

    previous_loader = core.bound_json
    try:
        core.bound_json = threshold_view
        result = ORIGINAL_CALIBRATION(attempts, judge_id)
    finally:
        core.bound_json = previous_loader
    for attempt, (validation, descriptive) in zip(result['attempts'], complete):
        attempt['retained_validation'] = validation
        attempt['descriptive_pair_reconciliation'] = descriptive
        attempt['complete_source_rating_count'] = sum(len(p['ratings']) for p in validation['pairs'])
    return result


core.validate_calibration = validate_calibration


def analyze(manifest_path, amendment_path):
    amendment = core.read_json(amendment_path)
    core.require(amendment.get('schema_version') == 1
                 and amendment.get('kind') == 'pre_aggregate_analysis_method_amendment'
                 and amendment.get('method_version') == METHOD_VERSION,
                 'versioned pre-aggregate method amendment required')
    for name, path in (('original_analyzer', CORE_PATH), ('compatibility_analyzer', Path(__file__).absolute()),
                       ('input_manifest', manifest_path)):
        core.verify(amendment.get(name))
        core.require(amendment[name] == core.binding(path), f'amendment {name} binding mismatch')
    core.require(amendment.get('recorded_before_primary_aggregate') is True
                 and amendment.get('changes_decision_rule') is False, 'amendment scope must preserve the fixed decision rule')
    core.verify(amendment.get('focused_test_source')); core.verify(amendment.get('verification_report'))
    interpretation = ORIGINAL_BOUND_JSON(amendment.get('interruption_interpretation'))
    core.require(interpretation.get('kind') == 'outcome_blind_interruption_interpretation', 'original interruption interpretation required')
    result = core.analyze(manifest_path)
    result['analysis_method_version'] = METHOD_VERSION
    result['method_amendment'] = core.binding(amendment_path)
    result['outcome_blind_interruption_interpretation'] = {
        'binding': amendment['interruption_interpretation'], 'record': interpretation}
    result['first_wave_use'] = 'diagnostic_only'
    result['credible_quality_gain_claim'] = False
    result['decision']['mechanical_gates_only'] = True
    result['decision']['interpretation'] += ' The outcome-blind interruption decision makes this first wave diagnostic; an uninterrupted fresh confirmation is required regardless of the gate result.'
    result['limitations'].append('The explicit pause, uncertain original dispatch times and inconsistent self-counted budget categories prevent a credible first-wave gain claim. Numeric reconciliation does not replace inspection of located visual evidence.')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    analysis = commands.add_parser('analyze', help='Run unchanged fixed aggregation after the frozen compatibility amendment.')
    analysis.add_argument('--manifest', type=Path, required=True)
    analysis.add_argument('--method-amendment', type=Path, required=True)
    analysis.add_argument('--output', type=Path, required=True, help='New result path; existing results are refused.')
    commands.add_parser('self-test', help='Run the original 39 synthetic controls unchanged.')
    args = parser.parse_args()
    try:
        if args.command == 'self-test':
            result = core.self_test()
            print(core.json.dumps(result, indent=2, default=core.json_number, allow_nan=False))
        else:
            result = analyze(args.manifest.absolute(), args.method_amendment.absolute())
            core.write_new(args.output, result)
            print(core.json.dumps({'result': core.binding(args.output), 'method': METHOD_VERSION,
                                   'first_wave_use': result['first_wave_use'],
                                   'credible_quality_gain_claim': False}, allow_nan=False))
    except (core.InvalidInput, KeyError, TypeError, AttributeError, OSError) as exc:
        parser.exit(2, f'Analysis refused: {exc}\n')


if __name__ == '__main__':
    main()
