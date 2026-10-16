#!/usr/bin/env python3
"""Add uniform native-pixel tall-view details to NEW, unscored render packets.

No models, ratings, screenshot selection, resizing, application or observer changes.
The existing packet/score helpers remain authoritative and unchanged.
"""
import argparse
import copy
from datetime import datetime, timezone
import hashlib
import io
import json
import os
from pathlib import Path
import re
import warnings

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
HEIGHT_THRESHOLD = 2048
TILE_HEIGHT = 1536
OVERLAP = 128
MAX_IMAGE_PIXELS = 64_000_000
MAX_TOTAL_PIXELS = 2_000_000_000
MAX_PNG_BYTES = 64 * 1024 * 1024
MAX_TOTAL_TILE_BYTES = 4 * 1024 * 1024 * 1024
MAX_TILES_PER_VIEW = 64
MAX_TOTAL_TILES = 8192
MAX_JSON_BYTES = 32 * 1024 * 1024
MAX_PACKET_VIEWS = 4096
MAX_INDEX_FILES = 16384
JUDGES = ('A', 'B')
INDEX = 'render-identities.private.json'
PRE_INDEX = 'render-identities.pre-tiles.private.json'
MANIFEST = 'tile-manifest.private.json'
TRANSACTION = 'tile-transaction.private.json'
BACKUPS = 'packets.pre-tiles.private'
STAGE = 'tile-stage.private'
DETAILS = 'detail-tiles'
DIMENSIONS = {'hierarchy_content', 'composition_typography', 'identity_specificity',
              'responsive_state_composure', 'economy_coherence'}
Image.MAX_IMAGE_PIXELS = MAX_IMAGE_PIXELS


def require(value, message):
    if not value:
        raise ValueError(message)


def normal_parts(value):
    require(isinstance(value, str) and value and '\\' not in value and '\x00' not in value,
            'Normal relative path required')
    parts = value.split('/')
    require(not Path(value).is_absolute() and all(p not in ('', '.', '..') for p in parts),
            'Normal relative path components required')
    return parts


def owned(path):
    path = Path(path).absolute()
    require(path != ROOT and ROOT in path.parents, 'Expected a project-owned path')
    for item in [path, *path.parents]:
        if item == ROOT:
            break
        require(not item.is_symlink(), 'Symlink component refused')
    return path


def project_path(value):
    return owned(ROOT.joinpath(*normal_parts(value)))


def child_path(parent, value):
    result = owned(parent.joinpath(*normal_parts(value)))
    require(parent in result.parents, 'Relative path escaped its owner')
    return result


def relative(path):
    return owned(path).relative_to(ROOT).as_posix()


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def binding(path, record_path=None):
    path = owned(path)
    require(path.is_file(), 'Expected a regular file')
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return {'path': relative(record_path or path), 'sha256': digest.hexdigest()}


def verify_binding(value):
    require(isinstance(value, dict) and set(value) >= {'path', 'sha256'}, 'File binding required')
    require(isinstance(value['sha256'], str) and re.fullmatch('[0-9a-f]{64}', value['sha256']),
            'SHA-256 file binding required')
    path = project_path(value['path'])
    require(binding(path) == {key: value[key] for key in ('path', 'sha256')}, 'Stale file binding')
    return path


def unique_object(pairs):
    value = {}
    for key, item in pairs:
        require(key not in value, 'Duplicate JSON key')
        value[key] = item
    return value


def load(path):
    path = owned(path)
    require(path.is_file() and 0 < path.stat().st_size <= MAX_JSON_BYTES, 'Bounded JSON file required')
    return json.loads(path.read_text(), object_pairs_hook=unique_object)


def encoded(value):
    return (json.dumps(value, indent=2, ensure_ascii=False, allow_nan=False) + '\n').encode()


def exclusive_write(path, data):
    path = owned(path)
    require(not path.exists(), 'Refusing existing output')
    require(path.parent.is_dir(), 'Output parent must exist')
    with path.open('xb') as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())


def replace_json(path, value):
    temporary = owned(path.with_name(path.name + '.writing'))
    exclusive_write(temporary, encoded(value))
    os.replace(temporary, path)


def tree_files(root):
    require(root.is_dir(), 'Packet directory required')
    files = []
    for path in sorted(root.rglob('*')):
        owned(path)
        require(path.is_file() or path.is_dir(), 'Non-regular packet entry refused')
        if path.is_file():
            files.append(path)
            require(len(files) <= MAX_INDEX_FILES, 'Too many packet files')
    return files


def tile_rectangles(width, height):
    require(type(width) is int and type(height) is int and width > 0 and height > 0,
            'Positive integer image dimensions required')
    if height <= HEIGHT_THRESHOLD:
        return []
    result = []
    y = 0
    while y < height:
        bottom = min(y + TILE_HEIGHT, height)
        result.append({'x': 0, 'y': y, 'width': width, 'height': bottom - y})
        require(len(result) <= MAX_TILES_PER_VIEW, 'Too many tiles for one view')
        if bottom == height:
            break
        y = bottom - OVERLAP
    return result


def open_png(path):
    path = owned(path)
    require(path.is_file() and 0 < path.stat().st_size <= MAX_PNG_BYTES, 'Bounded PNG file required')
    with warnings.catch_warnings():
        warnings.simplefilter('error', Image.DecompressionBombWarning)
        image = Image.open(path)
        try:
            require(image.format == 'PNG' and getattr(image, 'n_frames', 1) == 1, 'Static PNG required')
            require(image.width * image.height <= MAX_IMAGE_PIXELS, 'Decoded image exceeds pixel bound')
            require(image.mode in ('1', 'L', 'LA', 'P', 'RGB', 'RGBA', 'I;16'),
                    'Unsupported lossless PNG pixel mode')
            image.load()
        except BaseException:
            image.close()
            raise
    return image


def crop_pixels(image, rectangle):
    require(set(rectangle) == {'x', 'y', 'width', 'height'} and
            all(type(v) is int for v in rectangle.values()), 'Integer crop rectangle required')
    x, y, width, height = (rectangle[key] for key in ('x', 'y', 'width', 'height'))
    require(x == 0 and y >= 0 and width == image.width and 0 < height <= TILE_HEIGHT and
            y + height <= image.height, 'Crop rectangle escaped source or policy')
    crop = image.crop((x, y, x + width, y + height))
    # Palette transparency is decoded into explicit color/alpha, without resampling.
    if crop.mode == 'P':
        crop = crop.convert('RGBA')
    crop.info.clear()
    return crop


def verify_pixels(source_path, tile_path, rectangle):
    with open_png(source_path) as source, open_png(tile_path) as tile:
        with crop_pixels(source, rectangle) as expected:
            require(tile.size == expected.size and tile.mode == expected.mode and
                    tile.tobytes() == expected.tobytes(), 'Tile pixels differ from exact source crop')
            return {'equal': True, 'comparison_mode': expected.mode,
                    'method': 'decoded size,mode and every channel byte equal; no resize'}


def no_scores(setup):
    for path in setup.rglob('*'):
        owned(path)
        if path.is_dir():
            require(path.name not in ('locks', 'guess-forms', 'judgments'), 'Score/guess lock directory exists')
        elif path.is_file():
            if re.search(r'(score|guess|judgment|rating|lock)', path.name, re.IGNORECASE):
                require(path.name == 'scores-template.json' and
                        path.parent in [setup / 'packets' / judge for judge in JUDGES],
                        'Score/guess artifact exists')
        else:
            raise ValueError('Non-regular setup entry refused')


def blank_template(path, packet):
    value = load(path)
    require(value.get('schema_version') == 1 and value.get('kind') == 'render_only_score_lock_input' and
            value.get('judge_id') == packet['judge_id'] and value.get('locked') is False and
            value.get('human_acceptance') == 'not_collected', 'New blank score template required')
    artifacts = {item['neutral_label']: item for item in packet['artifacts']}
    require(set(value.get('scores', {})) == set(artifacts), 'Blank template label coverage mismatch')
    for label, score in value['scores'].items():
        require(set(score) == {'dimensions', 'presentation_order', 'pair_position',
                               'score_locked_before_guess', 'render_only'}, 'Unexpected score/template state')
        require(score['presentation_order'] == artifacts[label]['presentation_order'] and
                score['pair_position'] == artifacts[label]['pair_position'] and
                score['score_locked_before_guess'] is True and score['render_only'] is True,
                'Blank template placement mismatch')
        require(set(score['dimensions']) == DIMENSIONS, 'Fixed dimension template required')
        for observation in score['dimensions'].values():
            require(set(observation) == {'score', 'evidence'} and observation['score'] is None and
                    isinstance(observation['evidence'], list) and
                    all(isinstance(x, str) and not x.strip() for x in observation['evidence']),
                    'Packet has nonblank score/evidence; refusing update')


def validate_packet(packet, packet_root):
    require(set(packet) == {'schema_version', 'kind', 'judge_id', 'human_acceptance', 'artifacts', 'limits'} and
            packet['schema_version'] == 1 and packet['kind'] == 'render_only_craft_packet' and
            packet['human_acceptance'] == 'not_collected', 'Existing render-only packet schema required')
    require(isinstance(packet['judge_id'], str) and packet['judge_id'], 'Judge ID required')
    require(isinstance(packet['artifacts'], list) and 0 < len(packet['artifacts']) <= 128, 'Bounded artifacts required')
    labels, orders, count = set(), set(), 0
    for item in packet['artifacts']:
        require(set(item) == {'neutral_label', 'presentation_order', 'pair_position', 'brief', 'views'},
                'Existing neutral artifact schema required')
        require(isinstance(item['neutral_label'], str) and re.fullmatch(r'V[0-9]{2}', item['neutral_label']) and
                item['neutral_label'] not in labels, 'Unique neutral artifact label required')
        labels.add(item['neutral_label'])
        require(type(item['presentation_order']) is int and item['presentation_order'] > 0 and
                item['presentation_order'] not in orders and item['pair_position'] in ('first', 'second'),
                'Unique presentation order and pair placement required')
        orders.add(item['presentation_order'])
        require(child_path(packet_root, item['brief']).is_file(), 'Packet brief is absent')
        require(isinstance(item['views'], list), 'View list required')
        ids = set()
        for view in item['views']:
            count += 1
            require(count <= MAX_PACKET_VIEWS, 'Too many packet views')
            require(set(view) in ({'id', 'viewport', 'role', 'state', 'availability', 'path'},
                                 {'id', 'viewport', 'role', 'state', 'availability', 'path', 'tile'}),
                    'Neutral view schema required; metadata/source identity refused')
            require(isinstance(view['id'], str) and re.fullmatch(r'(?:view|detail)-[0-9]{2,}', view['id']) and
                    view['id'] not in ids, 'Unique neutral view ID required')
            ids.add(view['id'])
            require(view['role'] in ('viewport', 'fullpage', 'detail'), 'Unknown view role')
            require(isinstance(view['state'], str) and re.fullmatch(r'(?:initial|planned-state-[0-9]{2,})', view['state']),
                    'Neutral state required')
            viewport = view['viewport']
            require(isinstance(viewport, dict) and set(viewport) == {'width', 'height'} and
                    all(type(x) is int and 0 < x <= 16384 for x in viewport.values()), 'Integer viewport required')
            require(view['availability'] in ('available', 'unknown'), 'Known availability required')
            if view['availability'] == 'available':
                path = child_path(packet_root, view['path'])
                require(path.suffix == '.png' and path.is_file(), 'Available packet PNG is absent')
                require(not re.search(r'baseline|improved', view['path'], re.IGNORECASE), 'Non-neutral image path refused')
            else:
                require(view['path'] is None and view['role'] != 'detail', 'Unknown view must retain null path')
            require((view['role'] == 'detail') == ('tile' in view), 'Detail view coordinates required')
    require(orders == set(range(1, len(orders) + 1)), 'Complete presentation order required')


def index_packets(index, setup):
    require(index.get('schema_version') == 1 and index.get('kind') == 'private_render_packet_source_identities',
            'Existing private render identity index required')
    entries = index.get('packets')
    require(isinstance(entries, list) and len(entries) == 2 and {x.get('judge') for x in entries} == set(JUDGES),
            'Both packet identity entries required')
    result = {}
    for row in entries:
        packet_root = setup / 'packets' / row['judge']
        files = row.get('files')
        require(isinstance(files, list) and len(files) <= MAX_INDEX_FILES, 'Bounded packet identity list required')
        paths = []
        for item in files:
            path = verify_binding(item)
            require(packet_root in path.parents, 'Indexed packet file escaped its packet')
            paths.append(path)
        require(len(set(paths)) == len(paths) and set(paths) == set(tree_files(packet_root)),
                'Packet identity index must cover every current file exactly once')
        result[row['judge']] = row
    return result


def all_bindings(value):
    if isinstance(value, dict):
        if set(value) >= {'path', 'sha256'}:
            yield value
        for child in value.values():
            yield from all_bindings(child)
    elif isinstance(value, list):
        for child in value:
            yield from all_bindings(child)


def recheck_inputs(inputs):
    for item in inputs:
        verify_binding(item)


def verify_setup(setup, allow_in_progress=False):
    setup = owned(setup)
    manifest = load(setup / MANIFEST)
    require(manifest.get('kind') == 'uniform_native_pixel_tile_manifest' and manifest.get('schema_version') == 1,
            'Tile manifest required')
    require(manifest.get('policy') == policy(), 'Uniform tile policy differs')
    require(manifest['preserved_input_index']['sha256'] == manifest['input_index']['sha256'],
            'Preserved index differs from original input')
    before_index = load(verify_binding(manifest['preserved_input_index']))
    verify_binding(manifest['output_index'])
    index = load(setup / INDEX)
    rows = index_packets(index, setup)
    expected_index = copy.deepcopy(before_index)
    expected_rows = {row['judge']: row for row in expected_index['packets']}
    recheck_inputs(manifest['unchanged_inputs'])
    count = 0
    require([p['judge'] for p in manifest['packets']] == list(JUDGES), 'Complete ordered packet lineage required')
    for entry in manifest['packets']:
        judge = entry['judge']
        require(entry['preserved_input_packet']['sha256'] == entry['input_packet']['sha256'],
                'Preserved packet differs from original input')
        verify_binding(entry['preserved_input_packet'])
        packet_path = verify_binding(entry['output_packet'])
        packet_root = packet_path.parent
        packet = load(packet_path)
        validate_packet(packet, packet_root)
        require(rows[judge]['judge_id'] == packet['judge_id'], 'Packet/index judge mismatch')
        before = load(verify_binding(entry['preserved_input_packet']))
        expected = copy.deepcopy(before)
        by_label = {x['neutral_label']: x for x in expected['artifacts']}
        tile_entries = entry['tiles']
        actual_paths = []
        for tile in tile_entries:
            source = verify_binding(tile['original_png'])
            destination = verify_binding(tile['tile_png'])
            verify_pixels(source, destination, tile['rectangle'])
            require(tile['pixel_verification'].get('equal') is True, 'Pixel verification attestation absent')
            require(tile['public_view']['tile'] == tile['rectangle'] and tile['public_view']['role'] == 'detail' and
                    tile['public_view']['availability'] == 'available' and
                    child_path(packet_root, tile['public_view']['path']) == destination,
                    'Public tile coordinates/path differ from provenance')
            by_label[tile['neutral_label']]['views'].append(tile['public_view'])
            actual_paths.append(destination)
            count += 1
            require(count <= MAX_TOTAL_TILES, 'Tile count exceeds policy bound')
        require(packet == expected, 'Packet transformation differs from retained append-only lineage')
        expected_tiles = []
        for item in before['artifacts']:
            for view in item['views']:
                if view['availability'] == 'available' and view['role'] == 'fullpage':
                    source = child_path(packet_root, view['path'])
                    with open_png(source) as image:
                        expected_tiles.extend((item['neutral_label'], view['id'], r, binding(source),
                                               view['viewport'], view['state'])
                                              for r in tile_rectangles(image.width, image.height))
        require([(t['neutral_label'], t['source_view_id'], t['rectangle'], t['original_png'],
                  t['public_view']['viewport'], t['public_view']['state']) for t in tile_entries] == expected_tiles,
                'Tiles do not cover every required fullpage view in fixed order')
        require(set(actual_paths) == set(tree_files(packet_root / DETAILS)), 'Untracked or missing tile output')
        for old in expected_rows[judge]['files']:
            if old['path'] == entry['output_packet']['path']:
                old['sha256'] = entry['output_packet']['sha256']
        for tile in tile_entries:
            expected_rows[judge]['files'].append(tile['tile_png'])
            expected_index['views'].append({'judge': judge, 'neutral_label': tile['neutral_label'], 'role': 'detail',
                'source': tile['original_png'], 'packet_copy': tile['tile_png'],
                'source_view_id': tile['source_view_id'], 'pixel_rectangle': tile['rectangle'],
                'pixel_verification': tile['pixel_verification']})
    expected_index['pixel_tile_supplement'] = {'policy': policy(), 'pre_tiles_index': manifest['preserved_input_index'],
                                             'manifest_path': relative(setup / MANIFEST)}
    require(index == expected_index, 'Private index lost original bindings or differs from append-only lineage')
    if not allow_in_progress:
        transaction = load(setup / TRANSACTION)
        require(transaction.get('status') == 'complete' and transaction.get('manifest') == binding(setup / MANIFEST),
                'Incomplete transaction; reconcile retained evidence before reuse')
    return {'status': 'verified', 'packets': len(JUDGES), 'tiles': count, 'model_executions': 0,
            'manifest': relative(setup / MANIFEST)}


def policy():
    return {'source_role': 'fullpage', 'source_availability': 'available',
            'height_strictly_greater_than': HEIGHT_THRESHOLD, 'max_tile_height': TILE_HEIGHT,
            'vertical_overlap': OVERLAP, 'horizontal_crop': 'none', 'resize': False,
            'ordering': 'original artifact/view order,then top-to-bottom row order',
            'overview': 'retain every original view and original PNG byte unchanged',
            'selection': 'every qualifying view in both packets; no state,width,arm or content exceptions'}


def apply_tiles(setup, update_new_unscored=False):
    require(update_new_unscored is True, 'Explicit --update-new-unscored flag required')
    setup = owned(setup)
    require(setup.is_dir(), 'Existing setup directory required')
    for name in (PRE_INDEX, MANIFEST, TRANSACTION, BACKUPS, STAGE):
        target = owned(setup / name)
        require(not target.exists(), 'Existing tiling/transaction evidence; reconcile instead of replaying')
    no_scores(setup)
    for judge in JUDGES:
        require(not owned(setup / 'packets' / judge / DETAILS).exists(), 'Existing tile output refused')
    index_path = owned(setup / INDEX)
    index_bytes = index_path.read_bytes()
    index = load(index_path)
    require('pixel_tile_supplement' not in index, 'Index already tiled')
    rows = index_packets(index, setup)
    packets = {}
    inputs_by_path = {}
    for item in all_bindings(index):
        verify_binding(item)
        inputs_by_path[item['path']] = {key: item[key] for key in ('path', 'sha256')}
    index_input = binding(index_path)
    total_pixels = total_tiles = 0
    planned = []
    for judge in JUDGES:
        packet_root = setup / 'packets' / judge
        packet_path = packet_root / 'packet.json'
        packet = load(packet_path)
        validate_packet(packet, packet_root)
        blank_template(packet_root / 'scores-template.json', packet)
        require(rows[judge]['judge_id'] == packet['judge_id'], 'Packet/index judge mismatch')
        require(not any(v['role'] == 'detail' for a in packet['artifacts'] for v in a['views']),
                'Previously tiled packet refused')
        packets[judge] = {'root': packet_root, 'path': packet_path, 'input': binding(packet_path),
                          'input_bytes': packet_path.read_bytes(), 'before': packet, 'after': copy.deepcopy(packet), 'tiles': []}
        for artifact_number, item in enumerate(packet['artifacts'], 1):
            for view_number, view in enumerate(item['views'], 1):
                if view['availability'] != 'available' or view['role'] != 'fullpage':
                    continue
                source = child_path(packet_root, view['path'])
                with open_png(source) as image:
                    total_pixels += image.width * image.height
                    require(total_pixels <= MAX_TOTAL_PIXELS, 'Cumulative decoded-pixel bound exceeded')
                    rectangles = tile_rectangles(image.width, image.height)
                for number, rectangle in enumerate(rectangles, 1):
                    total_tiles += 1
                    require(total_tiles <= MAX_TOTAL_TILES, 'Total tile count exceeds policy bound')
                    destination = packet_root / DETAILS / f'item-{artifact_number:03d}' / f'view-{view_number:03d}' / f'tile-{number:03d}.png'
                    planned.append({'judge': judge, 'artifact_number': artifact_number, 'source_view': view,
                                    'label': item['neutral_label'], 'source': source, 'destination': destination,
                                    'rectangle': rectangle})
    # Only packet JSON and index are intentionally replaced. Every other source and
    # original packet file remains bound and unchanged through the transaction.
    changed_paths = {relative(p['path']) for p in packets.values()}
    unchanged = [value for key, value in inputs_by_path.items() if key not in changed_paths]
    journal = {'schema_version': 1, 'kind': 'native_pixel_tile_transaction', 'status': 'in_progress',
               'started_at': datetime.now(timezone.utc).isoformat(), 'phase': 'staging',
               'explicit_new_unscored_authorization': True, 'input_index': index_input,
               'input_packets': [packets[j]['input'] for j in JUDGES], 'model_executions': 0}
    exclusive_write(setup / TRANSACTION, encoded(journal))
    try:
        stage = owned(setup / STAGE)
        stage.mkdir()
        backups = owned(setup / BACKUPS)
        backups.mkdir()
        exclusive_write(setup / PRE_INDEX, index_bytes)
        for judge in JUDGES:
            exclusive_write(backups / f'{judge}.packet.json', packets[judge]['input_bytes'])
            (stage / judge).mkdir()
        total_tile_bytes = 0
        for plan in planned:
            packet = packets[plan['judge']]
            tile_number = len(packet['tiles']) + 1
            target = stage / plan['judge'] / plan['destination'].relative_to(packet['root'])
            target.parent.mkdir(parents=True, exist_ok=True)
            with open_png(plan['source']) as image, crop_pixels(image, plan['rectangle']) as crop:
                stream = io.BytesIO()
                crop.save(stream, format='PNG')
                data = stream.getvalue()
            total_tile_bytes += len(data)
            require(total_tile_bytes <= MAX_TOTAL_TILE_BYTES, 'Total tile bytes exceed policy bound')
            exclusive_write(target, data)
            pixel_check = verify_pixels(plan['source'], target, plan['rectangle'])
            public = {'id': f'detail-{tile_number:04d}', 'role': 'detail', 'availability': 'available',
                      'path': plan['destination'].relative_to(packet['root']).as_posix(),
                      'viewport': copy.deepcopy(plan['source_view']['viewport']), 'state': plan['source_view']['state'],
                      'tile': plan['rectangle']}
            packet['after']['artifacts'][plan['artifact_number'] - 1]['views'].append(public)
            packet['tiles'].append({'neutral_label': plan['label'], 'source_view_id': plan['source_view']['id'],
                                    'original_png': binding(plan['source']),
                                    'rectangle': plan['rectangle'], 'tile_png': binding(target, plan['destination']),
                                    'pixel_verification': pixel_check, 'public_view': public})
        final_index = copy.deepcopy(index)
        final_rows = {row['judge']: row for row in final_index['packets']}
        require(isinstance(final_index.get('views'), list), 'Existing source view lineage required')
        packet_lineage = []
        for judge in JUDGES:
            packet = packets[judge]
            # A no-tall-view packet still gets an empty, owned detail directory.
            (stage / judge / DETAILS).mkdir(exist_ok=True)
            new_bytes = encoded(packet['after'])
            exclusive_write(stage / f'{judge}.packet.json', new_bytes)
            final_packet_binding = {'path': relative(packet['path']), 'sha256': sha_bytes(new_bytes)}
            for old_binding in final_rows[judge]['files']:
                if old_binding['path'] == relative(packet['path']):
                    old_binding['sha256'] = final_packet_binding['sha256']
            for tile in packet['tiles']:
                final_rows[judge]['files'].append(tile['tile_png'])
                final_index['views'].append({'judge': judge, 'neutral_label': tile['neutral_label'], 'role': 'detail',
                                             'source': tile['original_png'], 'packet_copy': tile['tile_png'],
                                             'source_view_id': tile['source_view_id'], 'pixel_rectangle': tile['rectangle'],
                                             'pixel_verification': tile['pixel_verification']})
            packet_lineage.append({'judge': judge, 'input_packet': packet['input'],
                                   'preserved_input_packet': binding(backups / f'{judge}.packet.json'),
                                   'output_packet': final_packet_binding, 'tiles': packet['tiles']})
        final_index['pixel_tile_supplement'] = {'policy': policy(), 'pre_tiles_index': binding(setup / PRE_INDEX),
                                                'manifest_path': relative(setup / MANIFEST)}
        final_index_bytes = encoded(final_index)
        exclusive_write(stage / INDEX, final_index_bytes)
        manifest = {'schema_version': 1, 'kind': 'uniform_native_pixel_tile_manifest',
                    'created_at': datetime.now(timezone.utc).isoformat(), 'policy': policy(),
                    'tool': binding(Path(__file__)), 'policy_document': binding(Path(__file__).with_suffix('.md')),
                    'runtime': {'pillow': Image.__version__}, 'input_index': index_input,
                    'preserved_input_index': binding(setup / PRE_INDEX),
                    'output_index': {'path': relative(index_path), 'sha256': sha_bytes(final_index_bytes)},
                    'unchanged_inputs': unchanged, 'packets': packet_lineage,
                    'model_executions': 0, 'limits': ['Native pixel inspection supplement only; no ratings or human/AT acceptance.',
                        'All available qualifying fullpages and their originally fixed states are treated uniformly.',
                        'Unseen horizontal or dynamic offviewport behavior is not established by vertical crops.',
                        'File checks and explicit authorization do not authenticate private prior scoring or create a sandbox.']}
        exclusive_write(stage / MANIFEST, encoded(manifest))
        # Do not replay after any partial write. Journal/backups/staging remain for
        # reconciliation; all mutations below preserve original PNG/brief bytes.
        recheck_inputs(list(inputs_by_path.values()) + [index_input])
        no_scores(setup)
        journal['phase'] = 'committing_packets'
        replace_json(setup / TRANSACTION, journal)
        for judge in JUDGES:
            packet = packets[judge]
            owned(packet['root'] / DETAILS)
            require(not (packet['root'] / DETAILS).exists(), 'Tile destination appeared during staging')
            os.replace(stage / judge / DETAILS, packet['root'] / DETAILS)
            os.replace(stage / f'{judge}.packet.json', packet['path'])
        journal['phase'] = 'committing_index'
        replace_json(setup / TRANSACTION, journal)
        os.replace(stage / INDEX, index_path)
        os.replace(stage / MANIFEST, setup / MANIFEST)
        result = verify_setup(setup, allow_in_progress=True)
        journal.update(status='complete', phase='verified', finished_at=datetime.now(timezone.utc).isoformat(),
                       manifest=binding(setup / MANIFEST), output_index=binding(index_path))
        replace_json(setup / TRANSACTION, journal)
        for judge in JUDGES:
            (stage / judge).rmdir()
        stage.rmdir()
        return result
    except BaseException as error:
        journal.update(status='failed', failed_at=datetime.now(timezone.utc).isoformat(),
                       error={'type': type(error).__name__, 'message': str(error)},
                       next_action='Stop. Inspect preserved inputs,staged/committed outputs and journal; do not replay.')
        try:
            replace_json(setup / TRANSACTION, journal)
        except BaseException:
            pass  # Keep the original in-progress journal if failure recording also fails.
        raise


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    apply = sub.add_parser('apply')
    apply.add_argument('--setup', required=True, help='Normal project-relative NEW judge-packet-setup directory')
    apply.add_argument('--update-new-unscored', action='store_true',
                       help='Explicitly authorize replacement of both NEW blank packets and their private identity index')
    verify = sub.add_parser('verify')
    verify.add_argument('--setup', required=True)
    args = parser.parse_args()
    try:
        setup = project_path(args.setup)
        result = apply_tiles(setup, args.update_new_unscored) if args.command == 'apply' else verify_setup(setup)
    except (ValueError, OSError, json.JSONDecodeError, Image.DecompressionBombError,
            Image.DecompressionBombWarning) as error:
        parser.exit(1, 'error: ' + str(error) + '\n')
    print(json.dumps(result))


if __name__ == '__main__':
    main()
