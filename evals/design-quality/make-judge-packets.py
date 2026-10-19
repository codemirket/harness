#!/usr/bin/env python3
"""Freeze neutral judge orders and copy bound render-only evidence. No model calls."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import random
import re
import shutil
import struct

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / 'evals/design-quality'
OWNER = EXP = SETUP = None  # Set only from the caller's required experiment root.
DIMENSIONS = ['hierarchy_content', 'composition_typography', 'identity_specificity',
              'responsive_state_composure', 'economy_coherence']
JUDGE_IDS = {'A': '/root/quality_judge_a', 'B': '/root/quality_judge_b'}
SEEDS = {'A': 202610151, 'B': 202610152}
SCHEDULE_NAME = 'schedule.v3.private.json'
SCHEDULE_FILE = None
PNG_MAGIC = b'\x89PNG\r\n\x1a\n'


def require(value, message):
    if not value:
        raise ValueError(message)


def select_locations(experiment_root, schedule_path):
    global OWNER, EXP, SETUP, SCHEDULE_FILE
    def normal_path(value, option):
        require(value and not Path(value).is_absolute() and all(p not in ('', '.', '..') for p in value.split('/')),
                option + ' must use normal project-relative components')
        return owned(ROOT / value)
    EXP = normal_path(experiment_root, '--experiment-root')
    require(not EXP.exists() or EXP.is_dir(), '--experiment-root must name a directory')
    OWNER = EXP.parent
    SETUP = EXP / 'judge-packet-setup'
    SCHEDULE_FILE = SETUP / SCHEDULE_NAME
    if schedule_path is not None:
        SCHEDULE_FILE = normal_path(schedule_path, '--schedule')
        require(EXP in SCHEDULE_FILE.parents, '--schedule must be inside the selected experiment root')


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f'Duplicate JSON key: {key}')
        result[key] = value
    return result


def load(path):
    require(path.is_file(), f'Missing required caller-owned input: {relative(path)}; see evals/design-quality/reuse.md')
    return json.loads(path.read_text(), object_pairs_hook=unique_object)


def owned(path):
    path = path.absolute()
    require(path != ROOT and ROOT in path.parents, 'Expected project-owned path')
    for cursor in [path, *path.parents]:
        if cursor == ROOT:
            break
        require(not cursor.is_symlink(), 'Symlink component refused')
    return path


def relative(path):
    return str(owned(path).relative_to(ROOT))


def binding(path):
    path = owned(path)
    require(path.is_file(), 'Expected a regular file')
    return {'path': relative(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}


def verify(value):
    require(isinstance(value, dict) and set(value) >= {'path', 'sha256'}, 'File binding required')
    path = Path(value['path'])
    require(not path.is_absolute() and path.parts and all(x not in ('.', '..') for x in path.parts),
            'Expected normal project-relative binding')
    path = owned(ROOT / path)
    require(binding(path) == {'path': value['path'], 'sha256': value['sha256']}, 'Stale file identity')
    return path


def save(path, value):
    path = owned(path)
    require(not path.exists(), 'Refusing to overwrite a frozen artifact')
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def safe_name(value):
    return re.sub(r'[^A-Za-z0-9_.-]', '-', str(value)).lstrip('.')[:100] or 'capture'


def heldout_cells(initialization):
    cells = [c for c in initialization['cells'] if c['split'] == 'held_out']
    require(len(cells) == 16, 'Exactly sixteen planned heldout cells required')
    require(len({c['label'] for c in cells}) == 16, 'Unique heldout cell labels required')
    keys = {(c['case_id'], c['trial'], c['configuration']) for c in cells}
    case_ids = {c['case_id'] for c in cells}
    require(len(case_ids) == 4 and keys == {(c, t, a) for c in case_ids for t in (1, 2)
                                         for a in ('baseline', 'improved')}, 'Complete four-brief repeated design required')
    return cells


def assignment_identity(cells):
    fields = ['label', 'case_id', 'split', 'trial', 'configuration', 'raw_input', 'probe',
              'prompt_path', 'artifact_dir', 'observation_dir', 'receipt', 'grade']
    return [{key: cell[key] for key in fields} for cell in cells]


def make_orders(cells):
    groups = sorted({(c['case_id'], c['trial']) for c in cells})
    first_baseline_a = set(random.Random(202610150).sample(groups, 4))
    lookup = {(c['case_id'], c['trial'], c['configuration']): c for c in cells}
    rows = []
    for judge in ['A', 'B']:
        rng = random.Random(SEEDS[judge])
        permutation = list(groups); rng.shuffle(permutation)
        labels = rng.sample(range(10, 100), 16)
        for pair_index, group in enumerate(permutation):
            baseline_first = (group in first_baseline_a) == (judge == 'A')
            arms = ['baseline', 'improved'] if baseline_first else ['improved', 'baseline']
            for within, arm in enumerate(arms):
                position = 2 * pair_index + within + 1
                rows.append({'judge': judge, 'neutral_label': f'V{labels[position - 1]:02d}',
                             'cell_label': lookup[(*group, arm)]['label'],
                             'presentation_order': position, 'pair_position': 'first' if within == 0 else 'second'})
    return rows


def verify_orders(rows, cells):
    planned = {c['label']: c for c in cells}
    all_orders = []
    for judge in ['A', 'B']:
        entries = [x for x in rows if x['judge'] == judge]
        require(len(entries) == 16 and {x['cell_label'] for x in entries} == set(planned), 'Incomplete judge mapping')
        require(len({x['neutral_label'] for x in entries}) == 16, 'Duplicate neutral label')
        require({x['presentation_order'] for x in entries} == set(range(1, 17)), 'Incomplete presentation order')
        first = [x for x in entries if x['pair_position'] == 'first']
        require({arm: sum(planned[x['cell_label']]['configuration'] == arm for x in first)
                 for arm in ['baseline', 'improved']} == {'baseline': 4, 'improved': 4}, 'Unbalanced first-arm placement')
        for group in {(c['case_id'], c['trial']) for c in cells}:
            pair = sorted([x for x in entries if (planned[x['cell_label']]['case_id'], planned[x['cell_label']]['trial']) == group],
                          key=lambda x: x['presentation_order'])
            require([x['pair_position'] for x in pair] == ['first', 'second'], 'Invalid pair positions')
        all_orders.append([x['cell_label'] for x in sorted(entries, key=lambda x: x['presentation_order'])])
    require(all_orders[0] != all_orders[1], 'Judge orders must differ')
    for cell in cells:
        placements = [x['pair_position'] for x in rows if x['cell_label'] == cell['label']]
        require(set(placements) == {'first', 'second'}, 'A/B pair placements must reverse')


def view_policy():
    return {'schema_version': 1, 'initial_viewports': [{'width': 1280, 'height': 900},
             {'width': 1440, 'height': 900}, {'width': 390, 'height': 844}, {'width': 320, 'height': 800}],
            'image_roles': ['viewport', 'fullpage'],
            'states': 'Every explicit capture step and final capture in every frozen probe scenario, in declared order.',
            'missing_views': 'Retain each fixed slot as unknown; never substitute another frame or choose best output.',
            'excluded': ['DOM', 'HTML', 'decision', 'source', 'delivery logs', 'guard results', 'failure-only captures',
                         'source paths', 'condition labels', 'hashes', 'byte sizes', 'peer judgments'],
            'wide_1440': 'Fixed optional-view slot for all artifacts; absent collection is unknown, never replaced.'}


def schedule():
    initial_path = EXP / 'initialization.json'
    initialization = load(initial_path); cells = heldout_cells(initialization)
    rows = make_orders(cells); verify_orders(rows, cells)
    source_paths = [BASE / 'README.md', BASE / 'worker-prompt.md', BASE / 'craft-rubric.json',
                    BASE / 'reference.py', BASE / 'prepare.py', BASE / 'observe.cjs',
                    BASE / 'controls/observer-contract.md', Path(__file__),
                    OWNER / 'data-audit/score-schema.json', BASE / 'analyze.py']
    calibration = []
    for judge, actual in JUDGE_IDS.items():
        judgment = OWNER / f'judges/{judge}/calibration.json'
        validation = OWNER / f'judges/{judge}/calibration-validation.json'
        if judgment.is_file() and validation.is_file():
            # Bind real artifacts; never synthesize results or copy them into heldout scores.
            cal = load(judgment); val = load(validation)
            require(cal.get('judge_id') == actual == val.get('judge_id'), 'Actual calibration judge mismatch')
            require(val.get('judgment_sha256') == binding(judgment)['sha256'], 'Calibration validator identity mismatch')
            calibration.append({'id': judge, 'judge_id': actual,
                                'calibration_attempts': [{'judgment': binding(judgment), 'validation': binding(validation)}]})
    value = {'schema_version': 1, 'kind': 'fixed_neutral_craft_judge_schedule',
             'initialization': binding(initial_path), 'judge_ids': JUDGE_IDS, 'seeds': SEEDS,
             'label_map': rows, 'view_policy': view_policy(),
             'source_bindings': [binding(p) for p in source_paths], 'calibration_bindings': calibration,
             'timing': 'Scheduled before any heldout craft scoring; this is a coordinating attestation, not execution authentication.',
             'limits': ['Two same-host agent judges are not independent human audience participants.',
                        'Calibration duplicate consistency was identity-cued by visible hash metadata; do not claim proof of absent bias.',
                        'Main packets omit hashes, byte sizes and source filenames; visual arm inference remains possible.',
                        'The private label map and source-view hashes must never be supplied to judges before score lock.']}
    previous = SETUP / 'schedule.v2.private.json'
    if previous.is_file():
        old = load(previous)
        require(old['label_map'] == rows and old['view_policy'] == value['view_policy'], 'Frozen orders and views cannot change')
        updated_bindings = value['source_bindings']
        value = copy.deepcopy(old)
        value['source_bindings'] = updated_bindings
        value['portability_binding_update'] = {'schedule': binding(previous), 'reason': 'Optional experiment-root I/O portability before primary dispatch. Mapping, order, view policy, calibration and scoring rules are unchanged.'}
    save(SCHEDULE_FILE, value)
    print(json.dumps({'schedule': binding(SCHEDULE_FILE), 'mapping_rows': 32,
                      'judge_count': 2, 'heldout_cells_per_judge': 16, 'first_placement_per_arm_per_judge': 4}))


def fixed_slots(plan, policy):
    slots = []
    for viewport in policy['initial_viewports']:
        for role in policy['image_roles']:
            slots.append({'capture_id': f'default-{viewport["width"]}', 'viewport': viewport,
                          'role': role, 'state': 'initial'})
    state_index = 0
    for scenario in plan['scenarios']:
        stem = 'scenario-' + safe_name(scenario['id'])
        requested = [safe_name(s.get('name', s['id'])) for s in scenario['steps'] if s['action'] == 'capture'] + ['final']
        for name in requested:
            state_index += 1
            for role in policy['image_roles']:
                slots.append({'capture_id': stem + '-' + name,
                              'viewport': scenario.get('viewport', {'width': 1280, 'height': 900}),
                              'role': role, 'state': f'planned-state-{state_index:02d}'})
    return slots


def capture_index(observation):
    captures = list(observation.get('generic', {}).get('captures', []))
    for scenario in observation.get('scenarios', []):
        captures.extend(scenario.get('captures', []))
    index = {}
    for capture in captures:
        for file in capture.get('files', []):
            if file.get('role') in ('viewport', 'fullpage'):
                key = (capture['id'], file['role'])
                require(key not in index, 'Duplicate selected capture identity')
                index[key] = (capture, file)
    return index


def observation_file(root, value):
    relative_path = Path(value['path'])
    require(not relative_path.is_absolute() and relative_path.parts and all(x not in ('.', '..') for x in relative_path.parts),
            'Normal observer-relative image path required')
    path = owned(root / relative_path)
    require(root in path.parents and path.is_file(), 'Observer image escaped root or is absent')
    require(hashlib.sha256(path.read_bytes()).hexdigest() == value['sha256'], 'Changed observer image')
    data = path.read_bytes()
    require(data.startswith(PNG_MAGIC) and len(data) >= 24, 'PNG screenshot required')
    return path, {'width': struct.unpack('>I', data[16:20])[0], 'height': struct.unpack('>I', data[20:24])[0]}


def sealed_observations(freeze, replay, schedule_value):
    cells = heldout_cells(freeze)
    require(replay.get('schema_version') == 1 and replay.get('kind') == 'final_root_observer_replays', 'Final root replay manifest required')
    require(replay['experiment_freeze'] == binding(EXP / 'freeze.json'), 'Replay must bind the final improved freeze')
    observer = next(b for b in freeze['observer_bindings'] if b['path'] == 'evals/design-quality/observe.cjs')
    require(replay['observer'] == observer, 'Replay observer differs from frozen observer')
    verify(observer)
    entries = replay.get('entries', [])
    require(len(entries) == 16 and {e['cell_label'] for e in entries} == {c['label'] for c in cells}, 'All sixteen final root replays required')
    require(len({e['cell_label'] for e in entries}) == 16, 'Duplicate final root replay')
    observations = {}
    for cell in cells:
        receipt = load(ROOT / cell['receipt'])
        require(receipt.get('worker_id') and receipt.get('execution_status') in ('completed', 'infrastructure_failed'), 'All sixteen actual returns must be sealed')
        require((receipt['case_id'], receipt['configuration'], receipt['trial']) ==
                (cell['case_id'], cell['configuration'], cell['trial']), 'Receipt differs from planned cell')
        expected_files = {}
        artifact_root = owned(ROOT / cell['artifact_dir'])
        for b in receipt['output_bindings']:
            source = verify(b)
            require(artifact_root in source.parents, 'Receipt output escaped artifact root')
            expected_files[str(source.relative_to(artifact_root))] = b['sha256']
        entry = next(e for e in entries if e['cell_label'] == cell['label'])
        require(entry.get('status') in ('completed', 'infrastructure_failed'), 'Recorded replay status required')
        path = verify(entry['observation']); observation = load(path)
        source = observation.get('source_bindings', {})
        require(source.get('observer', {}).get('sha256') == observer['sha256'], 'Observation observer source changed')
        if source.get('artifact'):
            actual = {f['path']: f['sha256'] for f in source['artifact']['files']}
            require(actual == expected_files, 'Replay does not describe the sealed output')
        else:
            require(entry['status'] == 'infrastructure_failed', 'Missing artifact binding requires failed replay record')
        if observation.get('collection_completed'):
            require(entry['status'] == 'completed' and observation.get('artifact_unchanged') is True,
                    'Completed replay requires unchanged output')
        else:
            require(entry['status'] == 'infrastructure_failed' and entry.get('reason', '').strip(), 'Failed replay reason required')
        after = source.get('artifact_after')
        if after:
            require({f['path']: f['sha256'] for f in after['files']} == expected_files, 'Output changed during replay')
        observations[cell['label']] = {'path': path, 'value': observation, 'receipt': binding(ROOT / cell['receipt']),
                                     'replay_status': entry['status']}
    return cells, observations


def packet_instructions():
    return '''# Render-only craft review

Use only this packet's briefs, rubric and PNGs. Review artifacts in packet.json
presentation order. Do not read source, decisions, logs, condition identities,
private maps, hashes, guard results, another judge's packet or peer scores.
Judge the subject and audience, not a preferred style. Inspect every available
fixed view and record missing views as unknown; never invent an unavailable state.
Use the exact five continuous 0–100 dimensions. For each dimension give at least
two concrete observations naming a local PNG and its visible region/consequence.
Score all sixteen artifacts before making any arm guesses. Fill scores-template.json,
set locked=true, and save the result outside this read packet as your score-lock
input. The coordinator binds the complete score lock before supplying the
post-score guess form. Do not edit locked scores after guessing or source review.
Only then guess baseline/improved/unknown with confidence0–100. These guesses
assess imperfect blinding; they cannot change craft scores. Human acceptance was
not collected. Read/write boundaries are instructions, not a host sandbox.
'''


def build(replay_path):
    schedule_value = load(SCHEDULE_FILE)
    verify(schedule_value['initialization'])
    for b in schedule_value['source_bindings']:
        verify(b)
    freeze_path = EXP / 'freeze.json'; require(freeze_path.is_file(), 'Improved arm must be frozen before render packets')
    freeze = load(freeze_path)
    require(freeze.get('kind') == 'frozen_host_experiment' and freeze.get('improved_context'), 'Actual final experiment freeze required')
    require(assignment_identity(heldout_cells(freeze)) == assignment_identity(heldout_cells(load(verify(schedule_value['initialization'])))), 'Heldout assignment changed')
    for b in freeze['protocol_bindings'] + freeze['observer_bindings'] + freeze['source_manifests'] + [freeze['suite']]:
        verify(b)
    for m in freeze['source_manifests']:
        for b in load(verify(m))['files']:
            verify(b)
    replay_path = owned(replay_path); replay = load(replay_path)
    cells, observations = sealed_observations(freeze, replay, schedule_value)
    verify_orders(schedule_value['label_map'], cells)
    private = {'schema_version': 1, 'kind': 'private_render_packet_source_identities',
               'schedule': binding(SCHEDULE_FILE), 'experiment_freeze': binding(freeze_path),
               'replay_manifest': binding(replay_path), 'label_map': schedule_value['label_map'], 'views': [], 'packets': []}
    for judge in ['A', 'B']:
        packet_root = SETUP / f'packets/{judge}'
        require(not packet_root.exists(), 'Refusing to replace a judge packet')
        packet_root.mkdir(parents=True)
        entries = sorted([x for x in schedule_value['label_map'] if x['judge'] == judge], key=lambda x: x['presentation_order'])
        public = {'schema_version': 1, 'kind': 'render_only_craft_packet', 'judge_id': JUDGE_IDS[judge],
                  'human_acceptance': 'not_collected', 'artifacts': [],
                  'limits': ['Missing fixed views remain unknown.', 'Visual condition inference can remain possible.',
                             'PNG pixels and briefs only; no automatic craft or access conclusion.']}
        template = {'schema_version': 1, 'kind': 'render_only_score_lock_input', 'judge_id': JUDGE_IDS[judge],
                    'locked': False, 'human_acceptance': 'not_collected', 'scores': {}}
        briefs = {}
        for row in entries:
            cell = next(c for c in cells if c['label'] == row['cell_label'])
            if cell['case_id'] not in briefs:
                destination = packet_root / f'briefs/brief-{len(briefs) + 1:02d}.md'
                destination.parent.mkdir(parents=True, exist_ok=True)
                source = verify(cell['raw_input']); shutil.copyfile(source, destination)
                briefs[cell['case_id']] = str(destination.relative_to(packet_root))
                private['views'].append({'judge': judge, 'role': 'raw_brief', 'source': binding(source), 'packet_copy': binding(destination)})
            observation = observations[cell['label']]
            plan = load(verify(cell['probe']))
            slots = fixed_slots(plan, schedule_value['view_policy'])
            index = capture_index(observation['value'])
            item = {'neutral_label': row['neutral_label'], 'presentation_order': row['presentation_order'],
                    'pair_position': row['pair_position'], 'brief': briefs[cell['case_id']], 'views': []}
            for number, slot in enumerate(slots, 1):
                view = {'id': f'view-{number:02d}', 'viewport': slot['viewport'], 'role': slot['role'],
                        'state': slot['state'], 'availability': 'unknown', 'path': None}
                source_info = index.get((slot['capture_id'], slot['role']))
                identity = {'judge': judge, 'neutral_label': row['neutral_label'], 'cell_label': cell['label'],
                            'slot': slot, 'observation': binding(observation['path']), 'receipt': observation['receipt'],
                            'availability': 'unknown'}
                if source_info:
                    capture, file = source_info
                    require(capture['viewport'] == slot['viewport'], 'Selected view has unexpected viewport')
                    source, dimensions = observation_file(observation['path'].parent, file)
                    destination = packet_root / row['neutral_label'] / f'view-{number:02d}.png'
                    destination.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(source, destination)
                    view.update({'availability': 'available', 'path': str(destination.relative_to(packet_root))})
                    identity.update({'availability': 'available', 'source': binding(source), 'packet_copy': binding(destination),
                                     'pixel_dimensions': dimensions})
                item['views'].append(view); private['views'].append(identity)
            public['artifacts'].append(item)
            template['scores'][row['neutral_label']] = {
                'dimensions': {d: {'score': None, 'evidence': ['', '']} for d in DIMENSIONS},
                'presentation_order': row['presentation_order'], 'pair_position': row['pair_position'],
                'score_locked_before_guess': True, 'render_only': True}
        shutil.copyfile(BASE / 'craft-rubric.json', packet_root / 'craft-rubric.json')
        (packet_root / 'README.md').write_text(packet_instructions())
        save(packet_root / 'packet.json', public); save(packet_root / 'scores-template.json', template)
        private['packets'].append({'judge': judge, 'judge_id': JUDGE_IDS[judge],
                                  'files': [binding(p) for p in sorted(packet_root.rglob('*')) if p.is_file()]})
    save(SETUP / 'render-identities.private.json', private)
    print(json.dumps({'private_identity_manifest': binding(SETUP / 'render-identities.private.json'),
                      'judge_packet_dirs': [relative(SETUP / f'packets/{j}') for j in ['A', 'B']],
                      'artifacts_per_judge': 16, 'raw_briefs_per_judge': 4}))


def valid_number(value):
    return type(value) in (int, float) and math.isfinite(value) and 0 <= value <= 100


def validate_score_lock(value, packet):
    require(value.get('kind') == 'render_only_score_lock_input' and value.get('schema_version') == 1,
            'Render-only first-stage score-lock input required')
    require(value.get('judge_id') == packet['judge_id'] and value.get('locked') is True, 'Actual judge must lock complete first-stage scores')
    require(value.get('human_acceptance') == 'not_collected', 'Human acceptance was not collected')
    artifacts = {x['neutral_label']: x for x in packet['artifacts']}
    require(set(value.get('scores', {})) == set(artifacts), 'All sixteen neutral labels required')
    for label, rating in value['scores'].items():
        item = artifacts[label]
        require('arm_guess' not in rating, 'Arm guesses must follow the complete bound score lock')
        require(rating.get('presentation_order') == item['presentation_order'] and rating.get('pair_position') == item['pair_position'],
                'Score/schedule placement mismatch')
        require(rating.get('render_only') is True and rating.get('score_locked_before_guess') is True, 'Render-only pre-guess attestation required')
        require(set(rating.get('dimensions', {})) == set(DIMENSIONS), 'Exactly five fixed craft dimensions required')
        image_paths = [v['path'] for v in item['views'] if v['availability'] == 'available']
        for d, observation in rating['dimensions'].items():
            require(valid_number(observation.get('score')), 'Finite continuous 0–100 score required')
            evidence = observation.get('evidence')
            require(isinstance(evidence, list) and len(evidence) >= 2 and len(set(evidence)) >= 2,
                    'At least two distinct located observations per dimension required')
            for text in evidence:
                require(isinstance(text, str) and len(text.strip()) >= 20 and any(p in text for p in image_paths),
                        'Each observation must locate an available packet PNG and describe a visible region/consequence')


def lock_scores(judge, input_path):
    packet_path = SETUP / f'packets/{judge}/packet.json'
    packet = load(packet_path); value = load(owned(input_path))
    validate_score_lock(value, packet)
    identities = load(SETUP / 'render-identities.private.json')
    files = next(p['files'] for p in identities['packets'] if p['judge'] == judge)
    for b in files:
        verify(b)
    save(SETUP / f'locks/{judge}-render-scores.json', value)
    locked = binding(SETUP / f'locks/{judge}-render-scores.json')
    template = {'schema_version': 1, 'kind': 'post_score_arm_guess_input', 'judge_id': JUDGE_IDS[judge],
                'score_lock': locked, 'guesses': {label: {'arm': 'unknown', 'confidence': None} for label in value['scores']}}
    save(SETUP / f'guess-forms/{judge}.json', template)
    save(SETUP / f'locks/{judge}-receipt.private.json', {'schema_version': 1, 'kind': 'complete_pre_guess_score_lock_receipt',
         'judge': judge, 'judge_id': JUDGE_IDS[judge], 'input': binding(input_path), 'locked_scores': locked,
         'packet': binding(packet_path), 'render_identities': binding(SETUP / 'render-identities.private.json'),
         'limit': 'File identity and attestation do not authenticate when a judge privately formed a guess.'})
    print(json.dumps({'scores_locked': locked, 'guess_form': relative(SETUP / f'guess-forms/{judge}.json')}))


def merge_guesses(judge, input_path):
    lock_path = SETUP / f'locks/{judge}-render-scores.json'; value = load(lock_path)
    guesses = load(owned(input_path))
    require(guesses.get('kind') == 'post_score_arm_guess_input' and guesses.get('judge_id') == JUDGE_IDS[judge], 'Actual judge post-score guess file required')
    require(guesses.get('score_lock') == binding(lock_path), 'Guesses must bind the existing complete score lock')
    require(set(guesses.get('guesses', {})) == set(value['scores']), 'Exactly sixteen post-score guesses required')
    for label, guess in guesses['guesses'].items():
        require(set(guess) == {'arm', 'confidence'} and guess['arm'] in ('baseline', 'improved', 'unknown')
                and valid_number(guess['confidence']), 'Arm guess plus finite 0–100 confidence required')
        value['scores'][label]['arm_guess'] = guess
    value['kind'] = 'locked_craft_judgment'
    value['score_lock'] = binding(lock_path)
    value['post_score_guesses'] = binding(input_path)
    save(SETUP / f'judgments/{judge}.json', value)
    print(json.dumps({'judgment': binding(SETUP / f'judgments/{judge}.json'), 'judge_id': JUDGE_IDS[judge]}))


def self_test():
    cells = heldout_cells(load(EXP / 'initialization.json'))
    rows = make_orders(cells); verify_orders(rows, cells)
    changed_prompt = copy.deepcopy(cells); changed_prompt[0]['prompt_binding'] = {'path':'future-prompt.md','sha256':'0'*64}
    require(assignment_identity(changed_prompt) == assignment_identity(cells), 'Future prompt bindings must not change assignment')
    checks = ['future prompt binding is not an assignment change', '32-row complete coverage', 'distinct judge permutations', 'four first placements per arm per judge',
              'opposite matched pair positions across judges']
    probe = {'scenarios': [{'id': 'fixed', 'viewport': {'width':390,'height':844},
                            'steps': [{'id':'given','action':'capture'}, {'id':'ignored','action':'click'}]}]}
    slots = fixed_slots(probe, view_policy())
    require(len(slots) == 12 and slots[-1]['capture_id'] == 'scenario-fixed-final', 'Fixed-state selection regression')
    require('failure' not in ' '.join(s['capture_id'] for s in slots), 'Failure-only frames must not be selected')
    checks.append('fixed view slots retain optional wide and planned final states')
    # A synthetic structural packet exercises the score validator; it is never saved as an actual judgment.
    packet = {'judge_id':'SYNTHETIC-STRUCTURAL-CHECK', 'artifacts':[]}
    scores = {'schema_version':1,'kind':'render_only_score_lock_input','judge_id':packet['judge_id'],
              'locked':True,'human_acceptance':'not_collected','scores':{}}
    for n in range(16):
        name=f'S{n:02d}'; image=f'{name}/view-01.png'
        packet['artifacts'].append({'neutral_label':name,'presentation_order':n+1,'pair_position':'first' if n%2==0 else 'second',
                                    'views':[{'availability':'available','path':image}]})
        scores['scores'][name]={'dimensions':{d:{'score':52.5,'evidence':[f'{image}: title region defines the reading entry.',f'{image}: lower action area has a distinct relationship.']} for d in DIMENSIONS},
                               'presentation_order':n+1,'pair_position':'first' if n%2==0 else 'second','render_only':True,'score_locked_before_guess':True}
    validate_score_lock(scores,packet); checks.append('continuous intermediate scores and located evidence accepted')
    mutations=[('null score refused',lambda v:v['scores']['S00']['dimensions']['hierarchy_content'].update(score=None)),
               ('boolean score refused',lambda v:v['scores']['S00']['dimensions']['hierarchy_content'].update(score=True)),
               ('unlocated evidence refused',lambda v:v['scores']['S00']['dimensions']['hierarchy_content'].update(evidence=['looks good','looks clear'])),
               ('early arm guess refused',lambda v:v['scores']['S00'].update(arm_guess={'arm':'baseline','confidence':10})),
               ('missing label refused',lambda v:v['scores'].pop('S00'))]
    for name,change in mutations:
        broken=copy.deepcopy(scores);change(broken)
        try:validate_score_lock(broken,packet)
        except ValueError:checks.append(name)
        else:raise ValueError(name+' unexpectedly passed')
    save(SETUP / 'self-test.v2.json', {'schema_version':1,'kind':'synthetic_helper_structural_checks',
                                   'model_executions':0,'checks':checks,'passed':len(checks)})
    print(json.dumps({'synthetic_checks_passed':len(checks),'model_executions':0,
                      'evidence':binding(SETUP / 'self-test.v2.json')}))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--experiment-root', required=True, help='Normal project-relative experiment directory; parent owns data-audit and judges')
    parser.add_argument('--schedule', help='Normal project-relative schedule file inside the selected experiment root (default: judge-packet-setup/schedule.v3.private.json)')
    sub=parser.add_subparsers(dest='command',required=True)
    sub.add_parser('schedule'); sub.add_parser('self-test')
    p=sub.add_parser('build');p.add_argument('--replay-manifest',type=Path,required=True)
    for command in ['lock-scores','merge-guesses']:
        p=sub.add_parser(command);p.add_argument('--judge',choices=['A','B'],required=True);p.add_argument('--input',type=Path,required=True)
    args=parser.parse_args()
    try:
        select_locations(args.experiment_root, args.schedule)
    except ValueError as error:
        parser.error(str(error))
    if args.command=='schedule':schedule()
    elif args.command=='build':build(args.replay_manifest)
    elif args.command=='lock-scores':lock_scores(args.judge,args.input)
    elif args.command=='merge-guesses':merge_guesses(args.judge,args.input)
    else:self_test()

if __name__=='__main__':
    try:
        main()
    except ValueError as error:
        raise SystemExit('error: ' + str(error))
