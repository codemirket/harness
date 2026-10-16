#!/usr/bin/env python3
"""Original non-model synthetic checks; never reads studies, cases or ratings."""
import contextlib
import copy
import importlib.util
import io
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

from PIL import Image

sys.dont_write_bytecode = True
BASE = Path(__file__).resolve().parent


def module_at(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def pixel(x, y):
    # Both coordinates and alpha matter, including RGB under zero alpha.
    return (y % 251, (y // 251 + x) % 256, (x * 13 + y * 7) % 256, (x + y) % 256)


def synthetic_png(path, width, height):
    image = Image.new('RGBA', (width, height))
    image.putdata([pixel(x, y) for y in range(height) for x in range(width)])
    image.save(path)
    image.close()


class Fixture:
    def __init__(self, root):
        # macOS /var is a system symlink; use one canonical disposable root.
        root = root.resolve()
        self.root = root
        source = root / 'evals/design-quality'
        source.mkdir(parents=True)
        for name in ('tile-judge-views.py', 'tile-judge-views.md', 'make-judge-packets.py'):
            shutil.copyfile(BASE / name, source / name)
        self.helper_path = source / 'tile-judge-views.py'
        self.tool = module_at('synthetic_tile_helper', self.helper_path)
        self.setup = root / 'new-run/judge-packet-setup'
        self.setup.mkdir(parents=True)
        self.sources = root / 'original-non-model-fixtures'
        self.sources.mkdir()
        synthetic_png(self.sources / 'tall.png', 23, 4101)
        synthetic_png(self.sources / 'small.png', 23, 2048)
        synthetic_png(self.sources / 'threshold.png', 31, 2049)
        (self.sources / 'brief.md').write_text('# Original synthetic fixture\nNo model or prior study.\n')
        self.index = {'schema_version': 1, 'kind': 'private_render_packet_source_identities', 'views': [], 'packets': []}
        self.original_bytes = {}
        for judge, label in (('A', 'V10'), ('B', 'V20')):
            packet_root = self.setup / 'packets' / judge
            (packet_root / label).mkdir(parents=True)
            (packet_root / 'briefs').mkdir()
            shutil.copyfile(self.sources / 'brief.md', packet_root / 'briefs/brief-01.md')
            views = []
            for number, filename, role, state in ((1, 'small.png', 'fullpage', 'initial'),
                                                  (2, 'tall.png', 'fullpage', 'planned-state-01'),
                                                  (3, 'tall.png', 'viewport', 'planned-state-01'),
                                                  (5, 'threshold.png', 'fullpage', 'planned-state-02')):
                destination = packet_root / label / f'view-{number:02d}.png'
                shutil.copyfile(self.sources / filename, destination)
                views.append({'id': f'view-{number:02d}', 'viewport': {'width': 390, 'height': 844},
                              'role': role, 'state': state, 'availability': 'available',
                              'path': destination.relative_to(packet_root).as_posix()})
                self.index['views'].append({'judge': judge, 'neutral_label': label, 'role': role,
                                           'source': self.tool.binding(self.sources / filename),
                                           'packet_copy': self.tool.binding(destination)})
            views.insert(3, {'id': 'view-04', 'viewport': {'width': 1440, 'height': 900}, 'role': 'fullpage',
                             'state': 'initial', 'availability': 'unknown', 'path': None})
            packet = {'schema_version': 1, 'kind': 'render_only_craft_packet', 'judge_id': f'/synthetic/judge-{judge}',
                      'human_acceptance': 'not_collected', 'limits': ['Original synthetic structural fixture only.'],
                      'artifacts': [{'neutral_label': label, 'presentation_order': 1, 'pair_position': 'first',
                                     'brief': 'briefs/brief-01.md', 'views': views}]}
            self.write(packet_root / 'packet.json', packet)
            template = {'schema_version': 1, 'kind': 'render_only_score_lock_input', 'judge_id': packet['judge_id'],
                        'locked': False, 'human_acceptance': 'not_collected', 'scores': {
                            label: {'presentation_order': 1, 'pair_position': 'first',
                                    'score_locked_before_guess': True, 'render_only': True,
                                    'dimensions': {d: {'score': None, 'evidence': ['', '']} for d in sorted(self.tool.DIMENSIONS)}}}}
            self.write(packet_root / 'scores-template.json', template)
            (packet_root / 'README.md').write_text('Original synthetic packet, no ratings.\n')
            (packet_root / 'craft-rubric.json').write_text('{}\n')
            self.index['packets'].append({'judge': judge, 'judge_id': packet['judge_id'],
                                         'files': [self.tool.binding(p) for p in self.tool.tree_files(packet_root)]})
            for path in packet_root.rglob('*'):
                if path.is_file():
                    self.original_bytes[path] = path.read_bytes()
        self.write(self.setup / self.tool.INDEX, self.index)
        self.index_bytes = (self.setup / self.tool.INDEX).read_bytes()

    def write(self, path, value):
        path.write_text(json.dumps(value, indent=2) + '\n')

    def refresh_index(self):
        for row in self.index['packets']:
            row['files'] = [self.tool.binding(p) for p in self.tool.tree_files(self.setup / 'packets' / row['judge'])]
        self.write(self.setup / self.tool.INDEX, self.index)

    def apply(self):
        return self.tool.apply_tiles(self.setup, True)


class TilingChecks(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='original-tiling-check-')
        self.addCleanup(self.temp.cleanup)
        self.fixture = Fixture(Path(self.temp.name))
        self.tool = self.fixture.tool

    def assert_no_transaction(self):
        self.assertFalse((self.fixture.setup / self.tool.TRANSACTION).exists())

    def test_uniform_pixels_overviews_order_coverage_and_private_lineage(self):
        result = self.fixture.apply()
        self.assertEqual((result['packets'], result['tiles'], result['model_executions']), (2, 10, 0))
        self.assertEqual((self.fixture.setup / self.tool.PRE_INDEX).read_bytes(), self.fixture.index_bytes)
        manifest = self.tool.load(self.fixture.setup / self.tool.MANIFEST)
        for judge in ('A', 'B'):
            packet_root = self.fixture.setup / 'packets' / judge
            before = json.loads(self.fixture.original_bytes[packet_root / 'packet.json'])
            after = self.tool.load(packet_root / 'packet.json')
            original_views = before['artifacts'][0]['views']
            self.assertEqual(after['artifacts'][0]['views'][:5], original_views)
            details = after['artifacts'][0]['views'][5:]
            self.assertEqual([v['tile'] for v in details], [
                {'x': 0, 'y': 0, 'width': 23, 'height': 1536},
                {'x': 0, 'y': 1408, 'width': 23, 'height': 1536},
                {'x': 0, 'y': 2816, 'width': 23, 'height': 1285},
                {'x': 0, 'y': 0, 'width': 31, 'height': 1536},
                {'x': 0, 'y': 1408, 'width': 31, 'height': 641}])
            self.assertEqual(len({v['id'] for v in after['artifacts'][0]['views']}), 10)
            for view in details:
                self.assertEqual(set(view), {'id', 'viewport', 'role', 'state', 'availability', 'path', 'tile'})
                self.assertEqual((view['role'], view['availability']), ('detail', 'available'))
                self.assertNotRegex(view['path'], r'baseline|improved|[0-9a-f]{64}')
                with Image.open(packet_root / view['path']) as tile:
                    self.assertEqual(tile.size, (view['tile']['width'], view['tile']['height']))
                    # Independent literal pixel function checks every saved sample,
                    # including overlapping/boundary rows and transparent channels.
                    self.assertEqual(list(tile.getdata()), [pixel(x, y + view['tile']['y'])
                        for y in range(tile.height) for x in range(tile.width)])
            for height, group in ((4101, details[:3]), (2049, details[3:])):
                covered = set()
                for v in group:
                    covered.update(range(v['tile']['y'], v['tile']['y'] + v['tile']['height']))
                self.assertEqual(covered, set(range(height)))
                self.assertEqual(group[0]['tile']['y'], 0)
                self.assertEqual(group[-1]['tile']['y'] + group[-1]['tile']['height'], height)
            self.assertEqual([v['state'] for v in details], ['planned-state-01'] * 3 + ['planned-state-02'] * 2)
            self.assertEqual((self.fixture.setup / self.tool.BACKUPS / f'{judge}.packet.json').read_bytes(),
                             self.fixture.original_bytes[packet_root / 'packet.json'])
        for path, data in self.fixture.original_bytes.items():
            if path.name != 'packet.json':
                self.assertEqual(path.read_bytes(), data)
        index = self.tool.load(self.fixture.setup / self.tool.INDEX)
        self.assertEqual(index['views'][:len(self.fixture.index['views'])], self.fixture.index['views'])
        self.assertEqual(len(index['views']) - len(self.fixture.index['views']), 10)
        self.assertTrue(all(t['pixel_verification']['equal'] for p in manifest['packets'] for t in p['tiles']))
        self.assertEqual(self.tool.verify_setup(self.fixture.setup)['status'], 'verified')
        self.assertFalse((self.fixture.setup / self.tool.STAGE).exists())

    def test_explicit_new_unscored_flag_required(self):
        with self.assertRaisesRegex(ValueError, 'Explicit'):
            self.tool.apply_tiles(self.fixture.setup)
        self.assert_no_transaction()

    def test_initial_and_all_fixed_width_metadata_follow_identical_rule(self):
        for judge in ('A', 'B'):
            path = self.fixture.setup / 'packets' / judge / 'packet.json'
            packet = self.tool.load(path)
            views = packet['artifacts'][0]['views']
            if judge == 'B':
                views[1]['viewport'] = {'width': 320, 'height': 800}
            for number, width in ((6, 1280), (7, 1440)):
                view = copy.deepcopy(views[1])
                view.update(id=f'view-{number:02d}', viewport={'width':width,'height':900}, state='initial')
                views.append(view)
            self.fixture.write(path, packet)
        self.fixture.refresh_index()
        result = self.fixture.apply()
        self.assertEqual(result['tiles'], 22)
        manifest = self.tool.load(self.fixture.setup / self.tool.MANIFEST)
        for packet in manifest['packets']:
            grouped = {}
            for tile in packet['tiles']:
                grouped.setdefault(tile['source_view_id'], []).append(tile)
            for view_id in ('view-02', 'view-06', 'view-07'):
                self.assertEqual([t['rectangle']['y'] for t in grouped[view_id]], [0,1408,2816])
            self.assertTrue(all(t['public_view']['state']=='initial' for t in grouped['view-06']+grouped['view-07']))
            self.assertEqual({t['public_view']['viewport']['width'] for t in packet['tiles']},
                             {320,390,1280,1440} if packet['judge']=='B' else {390,1280,1440})

    def test_no_qualifying_views_retains_complete_packet(self):
        for judge in ('A','B'):
            path=self.fixture.setup/'packets'/judge/'packet.json'
            packet=self.tool.load(path)
            for view in packet['artifacts'][0]['views']:
                view['role']='viewport'
            self.fixture.write(path,packet)
        self.fixture.refresh_index()
        self.assertEqual(self.fixture.apply()['tiles'],0)
        self.assertEqual(self.tool.verify_setup(self.fixture.setup)['tiles'],0)

    def test_replay_refuses_completed_outputs(self):
        self.fixture.apply()
        packet = self.fixture.setup / 'packets/A/packet.json'
        before = packet.read_bytes()
        with self.assertRaisesRegex(ValueError, 'Existing tiling'):
            self.fixture.apply()
        self.assertEqual(packet.read_bytes(), before)

    def test_existing_private_manifest_refused(self):
        (self.fixture.setup / self.tool.MANIFEST).write_text('{}')
        with self.assertRaisesRegex(ValueError, 'Existing tiling'):
            self.fixture.apply()
        self.assert_no_transaction()

    def test_existing_tile_directory_refused(self):
        (self.fixture.setup / 'packets/A' / self.tool.DETAILS).mkdir()
        with self.assertRaisesRegex(ValueError, 'Existing tile output'):
            self.fixture.apply()
        self.assert_no_transaction()

    def test_existing_score_lock_refused(self):
        (self.fixture.setup / 'locks').mkdir()
        (self.fixture.setup / 'locks/A-render-scores.json').write_text('{}')
        with self.assertRaisesRegex(ValueError, 'Score/guess'):
            self.fixture.apply()
        self.assert_no_transaction()

    def test_nonblank_score_or_evidence_refused(self):
        path = self.fixture.setup / 'packets/B/scores-template.json'
        value = self.tool.load(path)
        next(iter(value['scores']['V20']['dimensions'].values()))['evidence'] = ['already inspected']
        self.fixture.write(path, value)
        self.fixture.refresh_index()
        with self.assertRaisesRegex(ValueError, 'nonblank'):
            self.fixture.apply()
        self.assert_no_transaction()

    def test_stale_original_binding_refused_before_write(self):
        path = self.fixture.setup / 'packets/A/V10/view-02.png'
        path.write_bytes(path.read_bytes() + b'changed')
        with self.assertRaisesRegex(ValueError, 'Stale'):
            self.fixture.apply()
        self.assert_no_transaction()

    def test_paths_refuse_escape_non_normal_and_symlinks(self):
        for value in ('../outside', '/absolute', 'a//b', './a', 'a/../b', 'a\\b', 'a\x00b'):
            with self.subTest(value=value), self.assertRaises(ValueError):
                self.tool.project_path(value)
        link = self.fixture.root / 'linked'
        link.symlink_to(self.fixture.setup, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, 'Symlink'):
            self.tool.project_path('linked/packets/A/packet.json')
        self.assert_no_transaction()

    def test_symlink_png_rejected_even_if_target_bytes_match(self):
        path = self.fixture.setup / 'packets/A/V10/view-02.png'
        path.unlink()
        path.symlink_to(self.fixture.sources / 'tall.png')
        with self.assertRaisesRegex(ValueError, 'Symlink'):
            self.fixture.apply()
        self.assert_no_transaction()

    def test_public_source_metadata_refused(self):
        path = self.fixture.setup / 'packets/A/packet.json'
        value = self.tool.load(path)
        value['artifacts'][0]['views'][0]['sha256'] = '0' * 64
        self.fixture.write(path, value)
        self.fixture.refresh_index()
        with self.assertRaisesRegex(ValueError, 'metadata/source identity'):
            self.fixture.apply()
        self.assert_no_transaction()

    def test_pixel_mismatch_detected_even_under_zero_alpha(self):
        original = self.fixture.sources / 'tall.png'
        tile = self.fixture.root / 'wrong.png'
        with Image.open(original) as image:
            crop = image.crop((0, 0, 23, 1536))
            self.assertEqual(crop.getpixel((0, 0))[3], 0)
            crop.putpixel((0, 0), (99, 88, 77, 0))
            crop.save(tile)
        with self.assertRaisesRegex(ValueError, 'pixels differ'):
            self.tool.verify_pixels(original, tile, {'x': 0, 'y': 0, 'width': 23, 'height': 1536})

    def test_saved_tile_tampering_refused_by_read_only_verification(self):
        self.fixture.apply()
        packet = self.tool.load(self.fixture.setup / 'packets/A/packet.json')
        tile_path = self.fixture.setup / 'packets/A' / packet['artifacts'][0]['views'][5]['path']
        with Image.open(tile_path) as image:
            changed = image.copy()
        changed.putpixel((0, 0), (1, 2, 3, 4))
        changed.save(tile_path)
        with self.assertRaisesRegex(ValueError, 'Stale'):
            self.tool.verify_setup(self.fixture.setup)

    def test_decoded_pixel_and_tile_count_bounds(self):
        with mock.patch.object(self.tool, 'MAX_IMAGE_PIXELS', 1):
            with self.assertRaisesRegex(ValueError, 'pixel bound'):
                self.fixture.apply()
        self.assert_no_transaction()
        with mock.patch.object(self.tool, 'MAX_TOTAL_TILES', 1):
            with self.assertRaisesRegex(ValueError, 'tile count'):
                self.fixture.apply()
        self.assert_no_transaction()

    def test_palette_transparency_and_16bit_samples_preserved(self):
        palette = self.fixture.root / 'palette.png'
        image = Image.new('P', (3, 2050))
        image.putpalette([n for value in range(256) for n in (value, 255-value, value//2)])
        image.putdata([y % 256 for y in range(2050) for x in range(3)])
        image.info['transparency'] = 0
        image.save(palette)
        sixteen = self.fixture.root / 'sixteen.png'
        image16 = Image.new('I;16', (3, 2050))
        image16.putdata([(y * 31 + x) % 65536 for y in range(2050) for x in range(3)])
        image16.save(sixteen)
        for source in (palette, sixteen):
            tile = source.with_name(source.stem + '-tile.png')
            with self.tool.open_png(source) as loaded, self.tool.crop_pixels(loaded, {'x':0,'y':0,'width':3,'height':1536}) as crop:
                crop.save(tile)
            self.assertTrue(self.tool.verify_pixels(source, tile, {'x':0,'y':0,'width':3,'height':1536})['equal'])

    def test_partial_commit_stops_and_preserves_reconciliation_evidence(self):
        destination = self.fixture.setup / 'packets/B/packet.json'
        real_replace = self.tool.os.replace
        def fail_second_packet(source, target):
            if Path(target) == destination:
                raise OSError('original synthetic second-packet write failure')
            return real_replace(source, target)
        with mock.patch.object(self.tool.os, 'replace', side_effect=fail_second_packet):
            with self.assertRaisesRegex(OSError, 'synthetic'):
                self.fixture.apply()
        transaction = self.tool.load(self.fixture.setup / self.tool.TRANSACTION)
        self.assertEqual(transaction['status'], 'failed')
        self.assertTrue((self.fixture.setup / self.tool.PRE_INDEX).exists())
        self.assertTrue((self.fixture.setup / self.tool.STAGE).is_dir())
        self.assertTrue((self.fixture.setup / self.tool.BACKUPS / 'A.packet.json').is_file())
        self.assertEqual(destination.read_bytes(), self.fixture.original_bytes[destination])
        with self.assertRaisesRegex(ValueError, 'reconcile instead of replaying'):
            self.fixture.apply()
        for path, data in self.fixture.original_bytes.items():
            if path.suffix == '.png':
                self.assertEqual(path.read_bytes(), data)

    def test_unchanged_score_lock_helper_accepts_detail_evidence_and_checks_tiles(self):
        self.fixture.apply()
        old = module_at('synthetic_existing_score_helper', self.fixture.root / 'evals/design-quality/make-judge-packets.py')
        old.SETUP = self.fixture.setup
        old.JUDGE_IDS = {'A':'/synthetic/judge-A','B':'/synthetic/judge-B'}
        packet_root = self.fixture.setup / 'packets/A'
        packet = self.tool.load(packet_root / 'packet.json')
        scores = self.tool.load(packet_root / 'scores-template.json')
        scores['locked'] = True
        tile_path = packet['artifacts'][0]['views'][5]['path']
        for observation in scores['scores']['V10']['dimensions'].values():
            observation.update(score=50, evidence=[
                f'{tile_path}: synthetic upper pixel band is visible in the detail.',
                f'{tile_path}: synthetic lower pixel band retains native samples.'])
        input_path = self.fixture.root / 'synthetic-score-input.json'
        self.fixture.write(input_path, scores)
        with contextlib.redirect_stdout(io.StringIO()):
            old.lock_scores('A', input_path)
        self.assertTrue((self.fixture.setup / 'locks/A-render-scores.json').is_file())
        index = self.tool.load(self.fixture.setup / self.tool.INDEX)
        bindings = next(row['files'] for row in index['packets'] if row['judge'] == 'A')
        located = next(item for item in bindings if item['path'].endswith(tile_path))
        self.assertEqual(old.verify(located), packet_root / tile_path)

    def test_cli_apply_verify_and_absent_update_flag(self):
        command = [sys.executable, str(self.fixture.helper_path)]
        argument = self.fixture.setup.relative_to(self.fixture.root).as_posix()
        denied = subprocess.run(command + ['apply','--setup',argument], capture_output=True, text=True)
        self.assertNotEqual(denied.returncode, 0)
        self.assertIn('Explicit', denied.stderr)
        self.assert_no_transaction()
        applied = subprocess.run(command + ['apply','--setup',argument,'--update-new-unscored'], capture_output=True, text=True)
        self.assertEqual(applied.returncode, 0, applied.stderr)
        checked = subprocess.run(command + ['verify','--setup',argument], capture_output=True, text=True)
        self.assertEqual(checked.returncode, 0, checked.stderr)
        self.assertEqual(json.loads(checked.stdout)['tiles'], 10)


if __name__ == '__main__':
    unittest.main(verbosity=2)
