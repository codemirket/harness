#!/usr/bin/env python3
"""Analyze bound design-quality evidence. Never collect grades or launch models."""

import argparse
import copy
from fractions import Fraction
import hashlib
import json
import math
from pathlib import Path
import re
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[2]
AUDIT = ROOT / '.mirket/implementation/design-quality-2026-10-15/data-audit'
DIMENSIONS = {
    'hierarchy_content': .25,
    'composition_typography': .25,
    'identity_specificity': .20,
    'responsive_state_composure': .20,
    'economy_coherence': .10,
}
ARMS = ('baseline', 'improved')
INTEGRITY_FLAGS = ('all_planned_outputs_retained', 'identity_consistent',
                   'no_material_leak', 'no_treatment_crossover')


class InvalidInput(ValueError):
    """An input cannot support the prespecified analysis."""


def require(test, message):
    if not test:
        raise InvalidInput(message)


def text_value(value, name):
    require(isinstance(value, str) and bool(value.strip()), f'{name}: nonempty string required')
    return value


def number(value, name, low=0, high=100):
    require(type(value) in (int, float) and math.isfinite(value), f'{name}: finite number required')
    require(low <= value <= high, f'{name}: must be in [{low}, {high}]')
    return value


def integer(value, name, low, high):
    require(type(value) is int and low <= value <= high, f'{name}: integer in [{low}, {high}] required')
    return value


def boolean(value, name):
    require(type(value) is bool, f'{name}: explicit boolean required')
    return value


def unique(values, name):
    require(len(values) == len(set(values)), f'{name}: duplicate identity')


def object_pairs(pairs):
    obj = {}
    for key, value in pairs:
        require(key not in obj, f'duplicate JSON key: {key}')
        obj[key] = value
    return obj


def finite_tree(value):
    if type(value) is float:
        require(math.isfinite(value), 'nonfinite JSON value')
    elif isinstance(value, dict):
        for child in value.values():
            finite_tree(child)
    elif isinstance(value, list):
        for child in value:
            finite_tree(child)


def read_json(path):
    try:
        value = json.loads(path.read_text(encoding='utf-8'), object_pairs_hook=object_pairs,
                           parse_constant=lambda v: (_ for _ in ()).throw(InvalidInput(f'nonfinite JSON: {v}')))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise InvalidInput(f'Cannot read JSON {path}: {exc}') from exc
    finite_tree(value)
    require(isinstance(value, dict), f'{path}: JSON object required')
    return value


def project_path(relative):
    text_value(relative, 'binding.path')
    require('\\' not in relative and all(p not in ('', '.', '..') for p in relative.split('/')),
            'binding.path must be normal project-relative path')
    require(not Path(relative).is_absolute(), 'absolute binding path refused')
    path = ROOT / relative
    for component in (path, *path.parents):
        if component == ROOT:
            break
        require(not component.is_symlink(), f'symlink component refused: {relative}')
    return path


def binding(path):
    path = path.absolute()
    try:
        relative = path.relative_to(ROOT).as_posix()
    except ValueError as exc:
        raise InvalidInput('file must be project-owned') from exc
    project_path(relative)
    require(path.is_file(), f'bound file absent: {relative}')
    return {'path': relative, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}


def verify(value):
    require(isinstance(value, dict) and set(value) == {'path', 'sha256'}, 'exact path/sha256 binding required')
    require(isinstance(value['sha256'], str) and re.fullmatch('[0-9a-f]{64}', value['sha256']),
            'binding.sha256 must be 64 lowercase hex characters')
    path = project_path(value['path'])
    require(binding(path) == value, f'stale binding: {value["path"]}')
    return path


def bound_json(value):
    return read_json(verify(value))


def verify_bindings(values, name, nonempty=True):
    require(isinstance(values, list) and (bool(values) or not nonempty), f'{name}: binding list required')
    unique([v.get('path') if isinstance(v, dict) else None for v in values], name)
    for value in values:
        verify(value)


def mean(values):
    values = list(values)
    require(bool(values), 'cannot average absent observations')
    return sum((Fraction(int(v)) if type(v) is bool else Fraction(str(v)) for v in values), Fraction()) / len(values)


def close(a, b):
    return math.isclose(a, b, rel_tol=0, abs_tol=1e-9)


def validate_rubric(value):
    require(value.get('schema_version') == 1 and value.get('scale') == [0, 100], 'fixed craft rubric required')
    dimensions = value.get('dimensions')
    require(isinstance(dimensions, list) and len(dimensions) == 5, 'five rubric dimensions required')
    unique([d.get('id') for d in dimensions], 'rubric dimensions')
    require({d.get('id') for d in dimensions} == set(DIMENSIONS), 'fixed dimension IDs required')
    for dimension in dimensions:
        require(close(number(dimension.get('weight'), 'dimension weight', 0, 1), DIMENSIONS[dimension['id']]),
                'prespecified dimension weight changed')
    require(value.get('human_acceptance') == 'not_collected', 'human acceptance must remain not_collected')


def validate_initialization(value):
    require(value.get('schema_version') == 1 and value.get('expected_cells') == 24
            and value.get('held_out_cells') == 16 and value.get('development_cells') == 8,
            'fixed first-wave 24-cell initialization required')
    cells, cases = value.get('cells'), value.get('cases')
    require(isinstance(cells, list) and len(cells) == 24, 'all 24 planned cells required')
    require(isinstance(cases, list) and len(cases) == 6, 'six fixed cases required')
    require(all(isinstance(c, dict) for c in cases + cells), 'planned case/cell objects required')
    unique([c.get('id') for c in cases], 'case IDs')
    case_by_id = {}
    for case in cases:
        identifier = text_value(case.get('id'), 'case id')
        require(case.get('split') in ('development', 'held_out'), 'case split required')
        criteria = case.get('criteria')
        require(isinstance(criteria, list) and bool(criteria), 'fixed nonempty criteria required')
        for criterion in criteria:
            text_value(criterion, 'criterion')
        unique(criteria, 'fixed criterion IDs')
        verify(case.get('input')); verify(case.get('rubric'))
        case_by_id[identifier] = case
    unique([c.get('label') for c in cells], 'cell labels')
    keys = []
    for cell in cells:
        text_value(cell.get('label'), 'cell label')
        case = case_by_id.get(cell.get('case_id'))
        require(case is not None and cell.get('split') == case['split'], 'cell/case split identity mismatch')
        require(cell.get('configuration') in ARMS, 'known arm required')
        integer(cell.get('trial'), 'trial', 1, 2)
        keys.append((cell['case_id'], cell['configuration'], cell['trial']))
        if 'raw_input' in cell:
            require(cell['raw_input'] == case['input'], 'cell raw input differs from fixed case')
            verify(cell['raw_input'])
    unique(keys, 'planned trial keys')
    require({c['split'] for c in cases} == {'development', 'held_out'}, 'both splits required')
    held_ids = sorted(c['id'] for c in cases if c['split'] == 'held_out')
    require(len(held_ids) == 4 and len(cases) - len(held_ids) == 2, 'four heldout/two development briefs required')
    require(set(keys) == {(c['id'], arm, repeat) for c in cases for arm in ARMS for repeat in (1, 2)},
            'planned trial matrix incomplete')
    for name in ('protocol_bindings', 'sealed_before_dispatch', 'observer_bindings', 'source_manifests'):
        if name in value:
            verify_bindings(value[name], name)
    if 'checker' in value:
        verify(value['checker'])
    if 'suite' in value:
        verify(value['suite'])
    return cells, case_by_id, held_ids


def validate_guards(value, initialization, cells, cases):
    require(value.get('schema_version') == 1, 'authoritative role-eval version 1 report required')
    trials = value.get('trials')
    require(isinstance(trials, list) and len(trials) == 24, 'all 24 actual guard trials required')
    require(all(isinstance(t, dict) for t in trials), 'guard trial objects required')
    if 'checker' in initialization:
        require(value.get('checker_sha256') == initialization['checker']['sha256'], 'guard checker identity mismatch')
    if 'suite' in initialization:
        require(value.get('suite_sha256') == initialization['suite']['sha256'], 'guard suite identity mismatch')
        suite = bound_json(initialization['suite'])
        require(suite.get('schema_version') == 1 and suite.get('trials_per_case') == 2, 'fixed guard suite required')
        configurations = suite.get('configurations')
        require(isinstance(configurations, list) and len(configurations) == 2
                and all(isinstance(c, dict) for c in configurations)
                and {c.get('id') for c in configurations} == set(ARMS), 'fixed guard configurations required')
        configuration_hashes = {}
        for configuration in configurations:
            verify(configuration.get('file'))
            configuration_hashes[configuration['id']] = configuration['file']['sha256']
    else:
        configuration_hashes = None
    planned = {(c['case_id'], c['configuration'], c['trial']) for c in cells}
    rows = {}
    for trial in trials:
        integer(trial.get('trial'), 'guard trial', 1, 2)
        key = (trial.get('case_id'), trial.get('configuration'), trial['trial'])
        require(key in planned and key not in rows, 'duplicate/unplanned guard trial identity')
        case = cases[key[0]]
        require(trial.get('input_sha256') == case['input']['sha256']
                and trial.get('rubric_sha256') == case['rubric']['sha256'], 'guard input/rubric identity mismatch')
        require(isinstance(trial.get('configuration_sha256'), str)
                and re.fullmatch('[0-9a-f]{64}', trial['configuration_sha256']), 'guard configuration binding required')
        if configuration_hashes is not None:
            require(trial['configuration_sha256'] == configuration_hashes[key[1]], 'guard configuration identity mismatch')
        verdicts = trial.get('verdicts')
        require(isinstance(verdicts, dict), 'guard verdict object required')
        if trial.get('outcome') == 'infrastructure_failed':
            require(verdicts == {} and trial.get('score') is None, 'infrastructure failures must remain unscored')
        else:
            require(set(verdicts) == set(case['criteria']), 'guard criteria missing or changed')
            require(all(type(v) is bool for v in verdicts.values()), 'guard verdicts must be actual booleans')
            expected = sum(verdicts.values()) / len(verdicts)
            require(close(number(trial.get('score'), 'guard score', 0, 1), expected), 'guard score does not reconcile')
            require(trial.get('outcome') == ('passed' if expected == 1 else 'task_failed'), 'guard outcome does not reconcile')
        text_value(trial.get('rationale'), 'guard rationale')
        verify_bindings(trial.get('evidence'), 'guard evidence')
        rows[key] = trial
    require(set(rows) == planned, 'missing planned guard trial')
    for arm in ARMS:
        unique_hashes = {r['configuration_sha256'] for k, r in rows.items() if k[1] == arm}
        require(len(unique_hashes) == 1, f'{arm}: inconsistent configuration identity')
    return rows


def dimension_scores(dimensions, name, observations=False):
    if observations:
        require(isinstance(dimensions, list) and len(dimensions) == 5, f'{name}: five dimensions required')
        require(all(isinstance(d, dict) for d in dimensions), f'{name}: dimension objects required')
        unique([d.get('id') for d in dimensions], f'{name} dimensions')
        dimensions = {d['id']: d for d in dimensions}
    require(isinstance(dimensions, dict) and set(dimensions) == set(DIMENSIONS), f'{name}: exact dimensions required')
    values = {}
    for identifier, weight in DIMENSIONS.items():
        dimension = dimensions[identifier]
        require(isinstance(dimension, dict), f'{name}/{identifier}: dimension object required')
        values[identifier] = number(dimension.get('score'), f'{name}/{identifier}')
        evidence = dimension.get('observations' if observations else 'evidence')
        require(isinstance(evidence, list) and len(evidence) >= 2, f'{name}/{identifier}: two observations required')
        if not observations:
            for location in evidence:
                text_value(location, f'{name}/{identifier} evidence')
            unique(evidence, f'{name}/{identifier} evidence')
    return values, sum((Fraction(str(DIMENSIONS[d])) * Fraction(str(values[d])) for d in DIMENSIONS), Fraction())


def validate_calibration(attempts, judge_id):
    require(isinstance(attempts, list) and 1 <= len(attempts) <= 2, 'one calibration, plus at most one retained retry')
    results, paths, packets = [], [], []
    for attempt in attempts:
        require(isinstance(attempt, dict), 'calibration attempt object required')
        judgment, validation = bound_json(attempt.get('judgment')), bound_json(attempt.get('validation'))
        paths.append(attempt['judgment']['path'])
        require(judgment.get('kind') == 'synthetic_craft_calibration_judgment'
                and judgment.get('synthetic_controls') is True, 'actual separately collected calibration judgment required')
        require(validation.get('kind') == 'synthetic_calibration_validation'
                and validation.get('synthetic_controls') is True and validation.get('control_model_executions') == 0,
                'saved calibration validator result required')
        require(judgment.get('judge_id') == judge_id == validation.get('judge_id'), 'calibration/heldout judge identity mismatch')
        require(validation.get('judgment_sha256') == attempt['judgment']['sha256'], 'calibration validation source mismatch')
        require(judgment.get('packet_id') == validation.get('packet_id'), 'calibration packet mismatch')
        packets.append(text_value(validation.get('packet_id'), 'calibration packet_id'))
        ratings = judgment.get('ratings')
        require(isinstance(ratings, list) and bool(ratings), 'original calibration ratings required')
        require(all(isinstance(r, dict) for r in ratings), 'calibration rating objects required')
        raw = {}
        for rating in ratings:
            key = (rating.get('pair_id'), rating.get('artifact_label'))
            require(all(isinstance(k, str) and k.strip() for k in key) and key not in raw,
                    'duplicate/absent calibration rating identity')
            _, raw[key] = dimension_scores(rating.get('dimensions'), f'calibration/{key}', observations=True)
        pairs = validation.get('pairs')
        require(isinstance(pairs, list) and bool(pairs), 'calibration contrast evidence required')
        require(all(isinstance(p, dict) for p in pairs), 'calibration pair objects required')
        unique([p.get('id') for p in pairs], 'calibration pairs')
        checks, families, seen = [], set(), set()
        for pair in pairs:
            scores = pair.get('ratings')
            require(isinstance(scores, list) and len(scores) == 2, 'paired calibration ratings required')
            require(all(isinstance(s, dict) for s in scores), 'paired calibration rating objects required')
            composites = []
            for score in scores:
                key = (pair.get('id'), score.get('label'))
                require(key in raw and key not in seen, 'calibration validator rating identity mismatch')
                composite = number(score.get('composite'), 'calibration composite')
                require(close(composite, raw[key]), 'calibration composite differs from source rating')
                seen.add(key); composites.append(composite)
            difference = composites[0] - composites[1]
            require(close(number(pair.get('difference_first_minus_second'), 'calibration difference', -100, 100), difference),
                    'calibration pair difference does not reconcile')
            if pair.get('kind') == 'composition':
                families.add(text_value(pair.get('family'), 'calibration family'))
                contrast = number(pair.get('positive_minus_negative'), 'positive/negative contrast', -100, 100)
                require(close(abs(contrast), abs(difference)), 'calibration contrast does not reconcile')
                passed = contrast >= 10
            elif pair.get('kind') == 'identical':
                require(close(number(pair.get('absolute_difference'), 'identical contrast'), abs(difference)),
                        'identical calibration difference does not reconcile')
                passed = abs(difference) <= 5
            else:
                raise InvalidInput('unknown calibration pair kind')
            require(boolean(pair.get('calibration_check_passed'), 'calibration pair flag') == passed,
                    'calibration pair flag does not match fixed threshold')
            checks.append(passed)
        require(seen == set(raw), 'calibration source ratings missing from validation')
        require(len(families) >= 2 and any(p.get('kind') == 'identical' for p in pairs),
                'both control families and identical pair required')
        calibrated = all(checks)
        require(boolean(validation.get('calibrated'), 'calibrated') == calibrated, 'calibrated flag does not reconcile')
        results.append({'judgment': attempt['judgment'], 'validation': attempt['validation'],
                        'calibrated': calibrated, 'retained_validation': validation})
    unique(paths, 'calibration attempt source paths')
    if len(results) == 2:
        require(not results[0]['calibrated'], 'a second calibration after success is outside the fixed protocol')
        unique(packets, 'retry calibration packets')
    return {'calibrated': results[-1]['calibrated'], 'attempts': results}


def validate_judges(manifest, cells, held_ids):
    judges = manifest.get('judges')
    require(isinstance(judges, list) and len(judges) == 2 and all(isinstance(j, dict) for j in judges)
            and {j.get('id') for j in judges} == {'A', 'B'},
            'exactly two planned judges A/B required')
    unique([j.get('judge_id') for j in judges], 'actual judge IDs')
    unique([j.get('judgment', {}).get('path') for j in judges], 'heldout judgment source paths')
    planned = {c['label']: c for c in cells if c['split'] == 'held_out'}
    mapping = manifest.get('label_map')
    require(isinstance(mapping, list) and len(mapping) == 32, 'complete 32-row neutral label mapping required')
    require(all(isinstance(m, dict) for m in mapping), 'neutral label map objects required')
    maps, rows, calibrations, orders = {}, {}, {}, {}
    for judge in sorted(judges, key=lambda j: j['id']):
        identifier = judge['id']
        text_value(judge.get('judge_id'), 'actual judge_id')
        entries = [m for m in mapping if m.get('judge') == identifier]
        require(len(entries) == 16 and {e.get('cell_label') for e in entries} == set(planned),
                'each judge must map every heldout cell exactly once')
        unique([e.get('cell_label') for e in entries], 'judge/cell map')
        unique([e.get('neutral_label') for e in entries], 'judge neutral labels')
        for entry in entries:
            text_value(entry.get('neutral_label'), 'neutral label')
            integer(entry.get('presentation_order'), 'presentation_order', 1, 16)
            require(entry.get('pair_position') in ('first', 'second'), 'first/second placement required')
        unique([e['presentation_order'] for e in entries], 'judge presentation positions')
        label_map = {e['neutral_label']: e for e in entries}
        maps[identifier] = label_map
        orders[identifier] = [e['cell_label'] for e in sorted(entries, key=lambda e: e['presentation_order'])]
        for case in held_ids:
            for repeat in (1, 2):
                pair = [e for e in entries if planned[e['cell_label']]['case_id'] == case
                        and planned[e['cell_label']]['trial'] == repeat]
                require(len(pair) == 2 and {e['pair_position'] for e in pair} == {'first', 'second'},
                        'matched judge pair placement incomplete')
                first, second = sorted(pair, key=lambda e: e['presentation_order'])
                require(first['pair_position'] == 'first' and second['pair_position'] == 'second',
                        'pair position contradicts presentation order')
        first_arm_counts = {arm: sum(e['pair_position'] == 'first' and planned[e['cell_label']]['configuration'] == arm
                                     for e in entries) for arm in ARMS}
        require(first_arm_counts == {'baseline': 4, 'improved': 4}, 'judge placement must counterbalance arms')
        judgment = bound_json(judge.get('judgment'))
        require(judgment.get('schema_version') == 1 and judgment.get('kind') == 'locked_craft_judgment'
                and judgment.get('locked') is True, 'locked render-only craft judgment required')
        require(judgment.get('judge_id') == judge['judge_id'], 'heldout judge binding identity mismatch')
        require(judgment.get('human_acceptance', 'not_collected') == 'not_collected', 'human acceptance was not collected')
        scores = judgment.get('scores')
        require(isinstance(scores, dict) and set(scores) == set(label_map), 'missing/unplanned neutral-label scores')
        for label, score in scores.items():
            require(isinstance(score, dict), 'score record object required')
            entry = label_map[label]; cell = planned[entry['cell_label']]
            integer(score.get('presentation_order'), 'rating presentation_order', 1, 16)
            require(score.get('presentation_order') == entry['presentation_order']
                    and score.get('pair_position') == entry['pair_position'], 'score/schedule placement mismatch')
            require(score.get('render_only') is True and score.get('score_locked_before_guess') is True,
                    'render-only and pre-guess score lock required')
            values, composite = dimension_scores(score.get('dimensions'), f'{identifier}/{label}')
            guess = score.get('arm_guess')
            require(isinstance(guess, dict) and guess.get('arm') in ('baseline', 'improved', 'unknown'), 'post-score arm guess required')
            number(guess.get('confidence'), 'arm guess confidence')
            rows[(identifier, cell['label'])] = {'judge': identifier, 'judge_id': judge['judge_id'],
                'neutral_label': label, 'cell_label': cell['label'], 'case_id': cell['case_id'],
                'configuration': cell['configuration'], 'trial': cell['trial'], 'composite': composite,
                'dimension_scores': values, 'raw_rating': score}
        calibrations[identifier] = validate_calibration(judge.get('calibration_attempts'), judge['judge_id'])
    require(all(m.get('judge') in ('A', 'B') for m in mapping), 'unplanned judge in label mapping')
    require(orders['A'] != orders['B'], 'independently permuted judge orders required')
    return rows, calibrations


def guard_summary(manifest, guards, held_ids):
    expected = {}
    unmatched = []
    for case in held_ids:
        for repeat in (1, 2):
            baseline, improved = (guards[(case, arm, repeat)] for arm in ARMS)
            if 'infrastructure_failed' in (baseline['outcome'], improved['outcome']):
                unmatched.append({'case_id': case, 'trial': repeat,
                                  'baseline_outcome': baseline['outcome'], 'improved_outcome': improved['outcome']})
                continue
            for criterion, verdict in improved['verdicts'].items():
                if baseline['verdicts'][criterion] and not verdict:
                    expected[(case, repeat, criterion)] = True
    assessments = manifest.get('guard_consequences')
    require(isinstance(assessments, list), 'explicit consequential guard assessments required')
    found = {}
    for assessment in assessments:
        require(isinstance(assessment, dict), 'consequence assessment object required')
        key = (assessment.get('case_id'), assessment.get('trial'), assessment.get('criterion'))
        require(key in expected and key not in found, 'duplicate/unmatched consequential assessment')
        boolean(assessment.get('consequential'), 'consequential')
        text_value(assessment.get('rationale'), 'consequential assessment rationale')
        verify_bindings(assessment.get('evidence'), 'consequential guard evidence')
        found[key] = assessment
    require(set(found) == set(expected), 'every new matched guard failure needs independent consequence assessment')
    counts = {arm: sum(guards[(case, arm, repeat)]['outcome'] == 'passed'
                        for case in held_ids for repeat in (1, 2)) for arm in ARMS}
    return {'heldout_full_pass_counts': counts, 'matched_new_failures': list(found.values()),
            'new_consequential_failure_count': sum(a['consequential'] for a in found.values()),
            'unavailable_matched_comparisons': unmatched,
            'all_24_guard_trials': [guards[k] for k in sorted(guards)],
            'guard_scale': 'Version-1 guard pass fraction; separate from continuous craft quality.'}


def analyze(manifest_path):
    manifest = read_json(manifest_path)
    require(manifest.get('schema_version') == 1 and manifest.get('kind') == 'design_quality_analysis_inputs',
            'design_quality_analysis_inputs manifest required')
    verify_bindings(manifest.get('source_bindings'), 'frozen source/view/schedule bindings')
    initialization = bound_json(manifest.get('initialization'))
    validate_rubric(bound_json(manifest.get('craft_rubric')))
    cells, cases, held_ids = validate_initialization(initialization)
    guard_report = bound_json(manifest.get('guard_report'))
    if not manifest.get('synthetic_controls', False):
        require('suite' in initialization, 'real analysis must bind frozen initialization with the authoritative suite')
        require(guard_report.get('provenance') == 'host_observed', 'actual host-observed guard report required')
    guards = validate_guards(guard_report, initialization, cells, cases)
    rows, calibrations = validate_judges(manifest, cells, held_ids)
    integrity = manifest.get('integrity')
    require(isinstance(integrity, dict), 'integrity attestations required')
    for flag in INTEGRITY_FLAGS:
        boolean(integrity.get(flag), flag)
    text_value(integrity.get('note'), 'integrity note')
    verify_bindings(integrity.get('evidence'), 'integrity evidence')
    guard = guard_summary(manifest, guards, held_ids)
    held = [c for c in cells if c['split'] == 'held_out']
    trial_scores = []
    for cell in held:
        judge_rows = [rows[(j, cell['label'])] for j in ('A', 'B')]
        trial_scores.append({'cell_label': cell['label'], 'case_id': cell['case_id'],
            'configuration': cell['configuration'], 'trial': cell['trial'],
            'composite': mean(r['composite'] for r in judge_rows),
            'dimensions': {d: mean(r['dimension_scores'][d] for r in judge_rows) for d in DIMENSIONS},
            'judge_composites': {r['judge']: r['composite'] for r in judge_rows}})
    briefs = []
    for case in held_ids:
        arms = {}
        for arm in ARMS:
            repeats = sorted([r for r in trial_scores if r['case_id'] == case and r['configuration'] == arm],
                             key=lambda r: r['trial'])
            arms[arm] = {'mean': mean(r['composite'] for r in repeats), 'repeats': repeats,
                'repeat_range': max(r['composite'] for r in repeats) - min(r['composite'] for r in repeats),
                'repeat_2_minus_1': repeats[1]['composite'] - repeats[0]['composite'],
                'dimension_means': {d: mean(r['dimensions'][d] for r in repeats) for d in DIMENSIONS}}
        judge_gains = {}
        for judge in ('A', 'B'):
            judge_gains[judge] = mean(rows[(judge, c['label'])]['composite'] for c in held
                if c['case_id'] == case and c['configuration'] == 'improved') - mean(
                rows[(judge, c['label'])]['composite'] for c in held
                if c['case_id'] == case and c['configuration'] == 'baseline')
        briefs.append({'case_id': case, 'arms': arms, 'gain': arms['improved']['mean'] - arms['baseline']['mean'],
            'judge_gains': judge_gains,
            'dimension_gains': {d: arms['improved']['dimension_means'][d] - arms['baseline']['dimension_means'][d]
                                for d in DIMENSIONS}})
    gains = [b['gain'] for b in briefs]
    primary = mean(gains)
    judge_gains = {j: mean(b['judge_gains'][j] for b in briefs) for j in ('A', 'B')}
    agreement = []
    for trial in trial_scores:
        difference = trial['judge_composites']['A'] - trial['judge_composites']['B']
        agreement.append({'cell_label': trial['cell_label'], 'case_id': trial['case_id'],
            'configuration': trial['configuration'], 'trial': trial['trial'],
            'A_minus_B': difference, 'absolute_difference': abs(difference), 'requires_frozen_score_inspection': abs(difference) > 20})
    guesses, order = {}, {}
    for judge in ('A', 'B'):
        scored = [r for (j, _), r in rows.items() if j == judge]
        known = [r for r in scored if r['raw_rating']['arm_guess']['arm'] != 'unknown']
        guesses[judge] = {'ratings': len(scored), 'known_guesses': len(known), 'unknown_guesses': len(scored) - len(known),
            'correct_known_guesses': sum(r['raw_rating']['arm_guess']['arm'] == r['configuration'] for r in known),
            'accuracy_among_known_guesses': mean(r['raw_rating']['arm_guess']['arm'] == r['configuration'] for r in known) if known else None,
            'mean_confidence_all_guesses': mean(r['raw_rating']['arm_guess']['confidence'] for r in scored),
            'raw_guesses': [{'cell_label': r['cell_label'], **r['raw_rating']['arm_guess']} for r in scored]}
        placements = {}
        for arm in (*ARMS, 'all'):
            values = {p: [r['composite'] for r in scored if r['raw_rating']['pair_position'] == p
                          and (arm == 'all' or r['configuration'] == arm)] for p in ('first', 'second')}
            placements[arm] = {'first_count': len(values['first']), 'second_count': len(values['second']),
                'first_mean': mean(values['first']), 'second_mean': mean(values['second']),
                'second_minus_first': mean(values['second']) - mean(values['first'])}
        ordered = sorted(scored, key=lambda r: r['raw_rating']['presentation_order'])
        order[judge] = {'within_pair_placement': placements,
            'first_half_mean': mean(r['composite'] for r in ordered[:8]),
            'second_half_mean': mean(r['composite'] for r in ordered[8:]),
            'presentation_order': [{'cell_label': r['cell_label'], 'position': r['raw_rating']['presentation_order'],
                                    'pair_position': r['raw_rating']['pair_position'], 'composite': r['composite']} for r in ordered],
            'interpretation': 'Descriptive only: artifact, brief, arm and sequence can confound placement; no causal order effect is identified.'}
    gates = {
        'mean_gain_at_least_7': primary >= 7,
        'at_least_3_brief_gains_at_least_5': sum(g >= 5 for g in gains) >= 3,
        'each_judge_overall_gain_at_least_3': all(g >= 3 for g in judge_gains.values()),
        'no_brief_mean_loss_worse_than_5': min(gains) >= -5,
        'no_new_consequential_matched_guard_failure': guard['new_consequential_failure_count'] == 0 and not guard['unavailable_matched_comparisons'],
        'no_guard_full_pass_count_loss': guard['heldout_full_pass_counts']['improved'] >= guard['heldout_full_pass_counts']['baseline'],
        'both_judges_calibrated': all(c['calibrated'] for c in calibrations.values()),
        **{flag: integrity[flag] for flag in INTEGRITY_FLAGS},
    }
    synthetic = manifest.get('synthetic_controls', False)
    require(type(synthetic) is bool, 'synthetic_controls must be boolean when present')
    if synthetic:
        require(manifest.get('model_executions') == 0, 'synthetic analysis must declare zero model executions')
    return {'schema_version': 1, 'kind': 'design_quality_continuous_analysis',
        'synthetic_controls': synthetic, 'model_executions': 0 if synthetic else None,
        'input_manifest': binding(manifest_path), 'input_bindings': manifest,
        'human_acceptance': 'not_collected', 'independent_comparison_count': 4,
        'aggregation': 'Fixed weighted dimension composite; average judges within trial, repeats within arm/brief, then unweighted mean of four improved-minus-baseline brief differences.',
        'primary_gain_points': primary, 'brief_gain_range': [min(gains), max(gains)],
        'leave_one_brief_out_means': [{'omitted_case_id': b['case_id'], 'mean_gain': mean(x['gain'] for x in briefs if x is not b)} for b in briefs],
        'judge_overall_gains': judge_gains,
        'dimension_overall_gains': {d: mean(b['dimension_gains'][d] for b in briefs) for d in DIMENSIONS},
        'per_brief': briefs, 'raw_ratings': [rows[k] for k in sorted(rows)],
        'judge_agreement': {'mean_absolute_difference': mean(a['absolute_difference'] for a in agreement),
                            'maximum_absolute_difference': max(a['absolute_difference'] for a in agreement),
                            'greater_than_20_point_disagreements': [a for a in agreement if a['requires_frozen_score_inspection']],
                            'all_trial_differences': agreement},
        'post_score_arm_guesses': guesses, 'descriptive_order_comparisons': order,
        'calibration': calibrations, 'guards': guard, 'integrity_attestations': integrity,
        'decision': {'practical_win': all(gates.values()), 'gates': gates,
                     'failed_gates': [g for g, passed in gates.items() if not passed],
                     'interpretation': 'Fixed bounded exploratory decision; agent ratings do not establish human acceptance or a general quality gain.'},
        'limitations': ['Four hand-authored brief clusters under one host and budget; repeats and judges are not independent sampled briefs.',
            'Thresholds are prespecified practical rules, not validated perceptual differences. No p-value, confidence interval or population-wide claim is produced.',
            'Same-host agents are not independent human experts; arm guesses and order comparisons are descriptive.',
            'Hashes bind saved bytes and identities; they cannot prove execution authenticity, attention, evidence truth, blinding or human approval.',
            'Exact backend identity/settings, tokens, cost and latency are not inferred; analysis does not invent telemetry.',
            'Original ratings and consequential disputes are retained; no ratings are overwritten after unblinding.']}


def write_new(path, value):
    path = path.absolute()
    try:
        relative = path.relative_to(ROOT).as_posix()
    except ValueError as exc:
        raise InvalidInput('analysis output must be project-owned') from exc
    project_path(relative)
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        with path.open('x', encoding='utf-8') as stream:
            json.dump(value, stream, indent=2, ensure_ascii=False, allow_nan=False, default=json_number)
            stream.write('\n')
    except FileExistsError as exc:
        raise InvalidInput(f'refusing overwrite: {relative}') from exc


def json_number(value):
    if isinstance(value, Fraction):
        return float(value)
    raise TypeError(f'Not JSON serializable: {type(value).__name__}')


def self_test():
    """Exercise synthetic arithmetic and refusal controls; zero model executions."""
    AUDIT.mkdir(parents=True, exist_ok=True)
    results = []
    with tempfile.TemporaryDirectory(prefix='synthetic-analysis-', dir=AUDIT) as directory:
        base = Path(directory)

        def put(name, value):
            path = base / name
            path.write_text(json.dumps(value, allow_nan=False), encoding='utf-8')
            return binding(path)

        evidence = put('synthetic-evidence.json', {'synthetic_controls': True, 'model_executions': 0})
        rubric = put('rubric.json', {'schema_version': 1, 'scale': [0, 100], 'human_acceptance': 'not_collected',
                                    'dimensions': [{'id': d, 'weight': w} for d, w in DIMENSIONS.items()]})
        cases, cells, guard_rows = [], [], []
        for n in range(6):
            case_id = f'SYNTHETIC_{n + 1}'
            split = 'held_out' if n < 4 else 'development'
            cases.append({'id': case_id, 'split': split, 'input': evidence, 'rubric': evidence, 'criteria': ['synthetic_guard']})
            for arm in ARMS:
                for repeat in (1, 2):
                    label = f'SYNTHETIC_CELL_{len(cells) + 1:02}'
                    cells.append({'label': label, 'case_id': case_id, 'configuration': arm, 'trial': repeat, 'split': split})
                    guard_rows.append({'case_id': case_id, 'configuration': arm, 'trial': repeat,
                        'input_sha256': evidence['sha256'], 'rubric_sha256': evidence['sha256'],
                        'configuration_sha256': hashlib.sha256(arm.encode()).hexdigest(),
                        'outcome': 'passed', 'score': 1, 'verdicts': {'synthetic_guard': True},
                        'rationale': 'Synthetic control only; zero model executions.', 'evidence': [evidence]})
        initialization = put('initialization.json', {'schema_version': 1, 'expected_cells': 24,
            'held_out_cells': 16, 'development_cells': 8, 'cases': cases, 'cells': cells})
        guard_report = put('guard-report.json', {'schema_version': 1, 'trials': guard_rows})
        manifest = {'schema_version': 1, 'kind': 'design_quality_analysis_inputs', 'synthetic_controls': True,
            'model_executions': 0, 'initialization': initialization, 'craft_rubric': rubric, 'guard_report': guard_report,
            'judges': [], 'label_map': [], 'guard_consequences': [], 'source_bindings': [evidence],
            'integrity': {**{flag: True for flag in INTEGRITY_FLAGS}, 'note': 'Synthetic structural controls, not observations.', 'evidence': [evidence]}}

        def dims(score, calibration=False):
            if calibration:
                return [{'id': d, 'score': score, 'observations': [{'synthetic': True}, {'synthetic': True}]} for d in DIMENSIONS]
            return {d: {'score': score, 'evidence': ['Synthetic located observation one', 'Synthetic located observation two']} for d in DIMENSIONS}

        held = [c for c in cells if c['split'] == 'held_out']
        for j, judge in enumerate(('A', 'B')):
            actual = f'SYNTHETIC_JUDGE_{judge}'
            packet = f'SYNTHETIC_PACKET_{judge}'
            cratings, pairs = [], []
            for k, (kind, scores) in enumerate((('composition', (70, 50)), ('composition', (65, 45)), ('identical', (60, 60)))):
                pair_id = f'SYNTHETIC_PAIR_{k}'
                ratings = []
                for m, score in enumerate(scores):
                    label = f'SYNTHETIC_CONTROL_{k}_{m}'
                    cratings.append({'pair_id': pair_id, 'artifact_label': label, 'dimensions': dims(score, True)})
                    ratings.append({'label': label, 'composite': score})
                pair = {'id': pair_id, 'kind': kind, 'family': f'SYNTHETIC_FAMILY_{k}', 'ratings': ratings,
                        'difference_first_minus_second': scores[0] - scores[1], 'calibration_check_passed': True}
                pair['positive_minus_negative' if kind == 'composition' else 'absolute_difference'] = scores[0] - scores[1]
                pairs.append(pair)
            calibration = put(f'calibration-{judge}.json', {'schema_version': 1, 'kind': 'synthetic_craft_calibration_judgment',
                'synthetic_controls': True, 'model_executions': 0, 'judge_id': actual, 'packet_id': packet, 'ratings': cratings})
            validation = put(f'calibration-validation-{judge}.json', {'schema_version': 1, 'kind': 'synthetic_calibration_validation',
                'synthetic_controls': True, 'control_model_executions': 0, 'judge_id': actual, 'packet_id': packet,
                'judgment_sha256': calibration['sha256'], 'calibrated': True, 'pairs': pairs})
            arranged = []
            for case_number in (range(4) if j == 0 else reversed(range(4))):
                for repeat in (1, 2):
                    pair = [c for c in held if c['case_id'] == f'SYNTHETIC_{case_number + 1}' and c['trial'] == repeat]
                    first_arm = ARMS[(case_number + repeat + j) % 2]
                    arranged.extend(sorted(pair, key=lambda c: c['configuration'] != first_arm))
            scores = {}
            for position, cell in enumerate(arranged, 1):
                label = f'SYNTHETIC_LABEL_{judge}_{position:02}'
                placement = 'first' if position % 2 else 'second'
                manifest['label_map'].append({'judge': judge, 'neutral_label': label, 'cell_label': cell['label'],
                                             'presentation_order': position, 'pair_position': placement})
                scores[label] = {'dimensions': dims(50 if cell['configuration'] == 'baseline' else 57),
                    'presentation_order': position, 'pair_position': placement,
                    'arm_guess': {'arm': 'unknown', 'confidence': 0}, 'score_locked_before_guess': True, 'render_only': True}
            judgment = put(f'judgment-{judge}.json', {'schema_version': 1, 'kind': 'locked_craft_judgment',
                'synthetic_controls': True, 'model_executions': 0, 'judge_id': actual, 'locked': True, 'scores': scores})
            manifest['judges'].append({'id': judge, 'judge_id': actual, 'judgment': judgment,
                'calibration_attempts': [{'judgment': calibration, 'validation': validation}]})
        path = base / 'manifest.json'

        def run(candidate):
            path.write_text(json.dumps(candidate, allow_nan=False), encoding='utf-8')
            return analyze(path)

        def check(name, assertion):
            require(assertion, f'self-test failed: {name}')
            results.append({'name': name, 'passed': True})

        def refuses(name, callback):
            try:
                callback()
            except (InvalidInput, KeyError, TypeError):
                results.append({'name': name, 'passed': True})
            else:
                raise InvalidInput(f'self-test failed to refuse: {name}')

        report = run(manifest)
        check('exact 7-point boundary passes with four brief clusters', close(report['primary_gain_points'], 7)
              and report['decision']['practical_win'] and report['independent_comparison_count'] == 4)
        check('all raw judge ratings retained', len(report['raw_ratings']) == 32)
        check('leave-one-brief-out uses three remaining brief differences',
              len(report['leave_one_brief_out_means']) == 4 and all(close(r['mean_gain'], 7) for r in report['leave_one_brief_out_means']))

        def changed_scores(gains_a, gains_b):
            candidate = copy.deepcopy(manifest)
            by_cell = {c['label']: c for c in held}
            for item in candidate['judges']:
                judgment = bound_json(item['judgment'])
                for label, rating in judgment['scores'].items():
                    entry = next(e for e in candidate['label_map'] if e['judge'] == item['id'] and e['neutral_label'] == label)
                    cell = by_cell[entry['cell_label']]
                    index = int(cell['case_id'].rsplit('_', 1)[1]) - 1
                    gain = (gains_a if item['id'] == 'A' else gains_b)[index]
                    rating['dimensions'] = dims(50 + gain if cell['configuration'] == 'improved' else 50)
                item['judgment'] = put(f'changed-{item["id"]}-{len(results)}.json', judgment)
            return candidate

        concentrated = run(changed_scores([28, 0, 0, 0], [28, 0, 0, 0]))
        check('one-brief gain cannot manufacture practical win', close(concentrated['primary_gain_points'], 7)
              and not concentrated['decision']['gates']['at_least_3_brief_gains_at_least_5'])
        divergent = run(changed_scores([2, 2, 2, 2], [12, 12, 12, 12]))
        check('mean 7 with one judge below 3 is blocked', close(divergent['primary_gain_points'], 7)
              and not divergent['decision']['gates']['each_judge_overall_gain_at_least_3'])
        loss = run(changed_scores([12, 12, 12, -6], [12, 12, 12, -6]))
        check('one brief worse than minus 5 is blocked despite aggregate gain', loss['primary_gain_points'] >= 7
              and not loss['decision']['gates']['no_brief_mean_loss_worse_than_5'])
        below = run(changed_scores([6.99] * 4, [6.99] * 4))
        check('below mean threshold is blocked without rounding up', not below['decision']['gates']['mean_gain_at_least_7'])
        for name, a, b in (
                ('exact 3-point judge threshold is included', [3] * 4, [11] * 4),
                ('exact 5-point brief threshold is included', [5, 5, 5, 13], [5, 5, 5, 13]),
                ('exact minus 5-point brief loss boundary is included', [11, 11, 11, -5], [11, 11, 11, -5])):
            check(name, run(changed_scores(a, b))['decision']['practical_win'])
        decimals = copy.deepcopy(manifest)
        for item in decimals['judges']:
            judgment = bound_json(item['judgment'])
            for label, rating in judgment['scores'].items():
                entry = next(e for e in decimals['label_map'] if e['judge'] == item['id'] and e['neutral_label'] == label)
                cell = next(c for c in held if c['label'] == entry['cell_label'])
                if cell['configuration'] == 'improved':
                    for dimension, score in zip(DIMENSIONS, (50.1, 50.3, 70, 60, 59)):
                        rating['dimensions'][dimension]['score'] = score
            item['judgment'] = put(f'decimal-{item["id"]}.json', judgment)
        exact = run(decimals)
        check('decimal-weighted threshold is exact before JSON display rounding',
              exact['primary_gain_points'] == Fraction(7) and exact['decision']['practical_win'])
        disputed = run(changed_scores([25] * 4, [3] * 4))
        check('greater-than-20 trial disagreement retained without regrading',
              len(disputed['judge_agreement']['greater_than_20_point_disagreements']) == 8)
        for flag in INTEGRITY_FLAGS:
            candidate = copy.deepcopy(manifest); candidate['integrity'][flag] = False
            check(f'{flag} false blocks claim', not run(candidate)['decision']['practical_win'])
        regression = copy.deepcopy(manifest)
        guard_data = bound_json(guard_report)
        target = next(r for r in guard_data['trials'] if r['case_id'] == 'SYNTHETIC_1' and r['configuration'] == 'improved' and r['trial'] == 1)
        target.update({'outcome': 'task_failed', 'score': 0, 'verdicts': {'synthetic_guard': False}})
        regression['guard_report'] = put('guard-regression.json', guard_data)
        refuses('unassessed new matched guard failure refused', lambda: run(regression))
        regression['guard_consequences'] = [{'case_id': 'SYNTHETIC_1', 'trial': 1, 'criterion': 'synthetic_guard',
            'consequential': True, 'rationale': 'Synthetic matched consequential regression.', 'evidence': [evidence]}]
        guard_loss = run(regression)
        check('consequential guard regression and full-pass-count loss both block win',
              not guard_loss['decision']['gates']['no_new_consequential_matched_guard_failure']
              and not guard_loss['decision']['gates']['no_guard_full_pass_count_loss'])
        calibration_fail = copy.deepcopy(manifest)
        item = calibration_fail['judges'][0]; cal_attempt = item['calibration_attempts'][0]
        source = bound_json(cal_attempt['judgment']); val = bound_json(cal_attempt['validation'])
        for rating in source['ratings']:
            if rating['pair_id'] == 'SYNTHETIC_PAIR_0' and rating['artifact_label'].endswith('_1'):
                rating['dimensions'] = dims(65, True)
        val['pairs'][0]['ratings'][1]['composite'] = 65
        val['pairs'][0].update({'difference_first_minus_second': 5, 'positive_minus_negative': 5, 'calibration_check_passed': False})
        val['calibrated'] = False
        cal_attempt['judgment'] = put('failed-calibration-source.json', source)
        val['judgment_sha256'] = cal_attempt['judgment']['sha256']
        cal_attempt['validation'] = put('failed-calibration-validation.json', val)
        check('retained calibration failure blocks win', not run(calibration_fail)['decision']['gates']['both_judges_calibrated'])
        for name, mutate in (
                ('missing rating', lambda v: v['scores'].pop(next(iter(v['scores'])))),
                ('out-of-range rating', lambda v: next(iter(v['scores'].values()))['dimensions']['hierarchy_content'].update(score=101)),
                ('boolean rating', lambda v: next(iter(v['scores'].values()))['dimensions']['hierarchy_content'].update(score=True)),
                ('boolean presentation order', lambda v: next(iter(v['scores'].values())).update(presentation_order=True)),
                ('absent dimension', lambda v: next(iter(v['scores'].values()))['dimensions'].pop('economy_coherence')),
                ('insufficient evidence', lambda v: next(iter(v['scores'].values()))['dimensions']['hierarchy_content'].update(evidence=['one'])),
                ('post-source scoring', lambda v: next(iter(v['scores'].values())).update(render_only=False))):
            candidate = copy.deepcopy(manifest); judgment = bound_json(candidate['judges'][0]['judgment']); mutate(judgment)
            candidate['judges'][0]['judgment'] = put(f'invalid-{len(results)}.json', judgment)
            refuses(name, lambda candidate=candidate: run(candidate))
        candidate = copy.deepcopy(manifest); candidate['label_map'][1] = copy.deepcopy(candidate['label_map'][0])
        refuses('duplicate cell mapping', lambda: run(candidate))
        candidate = copy.deepcopy(manifest); candidate['judges'][1]['judge_id'] = candidate['judges'][0]['judge_id']
        refuses('folded duplicate judge identities', lambda: run(candidate))
        candidate = copy.deepcopy(manifest)
        candidate['judges'][0]['calibration_attempts'][0]['judgment'] = candidate['judges'][1]['calibration_attempts'][0]['judgment']
        refuses('wrong calibration judge identity', lambda: run(candidate))
        candidate = copy.deepcopy(manifest)
        invalid_guard = bound_json(guard_report); invalid_guard['trials'][0]['score'] = 0
        candidate['guard_report'] = put('invalid-guard-score.json', invalid_guard)
        refuses('guard score inconsistent with actual boolean verdicts', lambda: run(candidate))
        candidate = copy.deepcopy(manifest)
        duplicate_guard = bound_json(guard_report); duplicate_guard['trials'][1] = copy.deepcopy(duplicate_guard['trials'][0])
        candidate['guard_report'] = put('duplicate-guard-identity.json', duplicate_guard)
        refuses('duplicate guard trial identity', lambda: run(candidate))
        candidate = copy.deepcopy(manifest)
        missing_guard = bound_json(guard_report); missing_guard['trials'].pop()
        candidate['guard_report'] = put('missing-guard-trial.json', missing_guard)
        refuses('missing planned guard trial', lambda: run(candidate))
        candidate = copy.deepcopy(manifest); candidate['source_bindings'][0]['sha256'] = '0' * 64
        refuses('stale source binding', lambda: run(candidate))
        candidate = copy.deepcopy(manifest); candidate['source_bindings'][0]['path'] = '../escape.json'
        refuses('escaped binding path', lambda: run(candidate))
        for name, content in (('duplicate JSON object keys', '{"a":1,"a":2}'),
                              ('nonfinite literal', '{"a":NaN}'), ('overflowed numeric literal', '{"a":1e400}')):
            bad = base / 'bad-json.json'; bad.write_text(content, encoding='utf-8')
            refuses(name, lambda: read_json(bad))
        destination = base / 'saved-result.json'; write_new(destination, {'synthetic_controls': True, 'model_executions': 0})
        refuses('existing results cannot be erased', lambda: write_new(destination, {}))
        symlink = base / 'symlink.json'; symlink.symlink_to(base / 'synthetic-evidence.json')
        refuses('symlink input bindings', lambda: verify({'path': symlink.relative_to(ROOT).as_posix(), 'sha256': evidence['sha256']}))
    return {'schema_version': 1, 'kind': 'design_quality_analysis_self_test', 'synthetic_controls': True,
            'model_executions': 0, 'human_acceptance': 'not_collected', 'passed': True,
            'checks': results, 'check_count': len(results),
            'interpretation': 'Synthetic arithmetic/schema controls only; no experiment outputs, grades, model executions or human acceptance were collected.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    analysis = commands.add_parser('analyze', help='Analyze a saved explicitly bound input manifest.')
    analysis.add_argument('--manifest', type=Path, required=True)
    analysis.add_argument('--output', type=Path, help='New project-owned JSON result path; existing files are refused.')
    control = commands.add_parser('self-test', help='Run labeled synthetic controls with zero model executions.')
    control.add_argument('--output', type=Path, help='Optional new project-owned self-test report.')
    args = parser.parse_args()
    try:
        result = self_test() if args.command == 'self-test' else analyze(args.manifest.absolute())
        if args.output:
            write_new(args.output, result)
            print(json.dumps({'result': binding(args.output), 'kind': result['kind'],
                              'synthetic_controls': result['synthetic_controls'],
                              'practical_win': result.get('decision', {}).get('practical_win'),
                              'self_test_passed': result.get('passed')}, allow_nan=False))
        else:
            print(json.dumps(result, indent=2, ensure_ascii=False, allow_nan=False, default=json_number))
    except (InvalidInput, KeyError, TypeError, AttributeError, OSError) as exc:
        parser.exit(2, f'Analysis refused: {exc}\n')


if __name__ == '__main__':
    main()
