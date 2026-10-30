"""External-host acquisition refuses unsafe inputs and publishes only verified bytes."""
import base64
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('acquire_pi_host', Path(__file__).resolve().parents[1] / 'scripts/acquire-pi-host.py')
ACQUIRE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ACQUIRE)


def archive(entries):
    result = io.BytesIO()
    with tarfile.open(fileobj=result, mode='w:gz') as output:
        for name, content, kind in entries:
            member = tarfile.TarInfo(name)
            member.type = kind
            member.size = len(content) if kind == tarfile.REGTYPE else 0
            member.mode = 0o644
            if kind in (tarfile.SYMTYPE, tarfile.LNKTYPE):
                member.linkname = '../../outside'
            output.addfile(member, io.BytesIO(content) if member.isfile() else None)
    return result.getvalue()


def integrity(data):
    return 'sha512-' + base64.b64encode(hashlib.sha512(data).digest()).decode()


class HostAcquisitionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='mirket-host-acquire-test-', dir='/private/tmp')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.package = self.root / 'package'
        self.cache = self.root / 'archives'
        self.output = self.root / 'host'
        self.cache.mkdir()
        self.verifier = self.package / 'native/bin/mirket'
        self.verifier.parent.mkdir(parents=True)
        self.verifier.write_bytes(b'bound verifier')
        self.content = b'unchanged official file'
        self.url = 'https://registry.npmjs.org/fixture/-/fixture-1.0.0.tgz'
        self.data = archive([('package/index.js', self.content, tarfile.REGTYPE)])
        self.cache_file = self.cache / (ACQUIRE.sha(self.url.encode()) + '.tgz')
        self.cache_file.write_bytes(self.data)
        self.lock = self.root / 'host-lock.json'
        self.lock.write_text(json.dumps({'packages': {'': {}, 'node_modules/fixture': {
            'resolved': self.url, 'integrity': integrity(self.data)}}}))
        self.inventory = {'schema': 1, 'hostVersion': '1.4.0', 'entry': 'fixture/index.js',
                          'lockSha256': ACQUIRE.sha(self.lock.read_bytes()), 'files': [
                              {'path': 'fixture/index.js', 'bytes': len(self.content), 'sha256': ACQUIRE.sha(self.content)}]}
        self.bind()

    def bind(self):
        inventory = json.dumps(self.inventory).encode()
        (self.package / 'host-inventory.json').write_bytes(inventory)
        (self.package / 'native-binding.json').write_text(json.dumps({
            'hostInventorySha256': ACQUIRE.sha(inventory), 'coreSha256': ACQUIRE.sha(self.verifier.read_bytes())}))

    def verified(self, command, **kwargs):
        self.assertEqual(command[1], 'verify-pi-host')
        host = Path(command[command.index('--root') + 1])
        self.assertEqual((host / 'fixture/index.js').read_bytes(), self.content)
        self.assertEqual(kwargs['env']['HOME'], str(host.parent))
        self.assertFalse(self.output.exists())
        return json.dumps({'root': str(host), 'entry': str(host / 'fixture/index.js'),
                           'lockSha256': self.inventory['lockSha256'], 'verifiedFiles': 1}).encode()

    def run_acquire(self):
        return ACQUIRE.acquire(self.package, self.lock, self.output, self.cache)

    def no_staging(self):
        self.assertEqual(list(self.root.glob('.host-acquire-*')), [])

    def test_offline_acquisition_checks_native_response_before_private_atomic_publish(self):
        with patch.object(ACQUIRE.subprocess, 'check_output', side_effect=self.verified) as verify, \
                patch.object(ACQUIRE.urllib.request, 'build_opener', side_effect=AssertionError('offline means no network')):
            report = self.run_acquire()
        self.assertEqual(report['verifiedFiles'], 1)
        self.assertEqual(report['entry'], str(self.output / 'node_modules/fixture/index.js'))
        self.assertEqual(self.output.stat().st_mode & 0o777, 0o700)
        self.assertEqual(self.cache_file.read_bytes(), self.data)
        self.assertEqual(verify.call_count, 1)
        self.no_staging()

    def test_binding_lock_and_native_byte_changes_fail_before_archive_access(self):
        changes = [self.package / 'host-inventory.json', self.lock, self.verifier]
        for path in changes:
            with self.subTest(path=path):
                original = path.read_bytes()
                path.write_bytes(original + b' ')
                with patch.object(ACQUIRE, 'archive_bytes', side_effect=AssertionError('must refuse first')):
                    with self.assertRaisesRegex(ValueError, 'differs'):
                        self.run_acquire()
                path.write_bytes(original)
                self.assertFalse(self.output.exists())
                self.no_staging()

    def test_archive_integrity_failure_preserves_cache_and_publishes_nothing(self):
        self.cache_file.write_bytes(self.data + b'tampered')
        with self.assertRaisesRegex(ValueError, 'integrity differs'):
            self.run_acquire()
        self.assertFalse(self.output.exists())
        self.assertTrue(self.cache_file.read_bytes().endswith(b'tampered'))
        self.no_staging()

    def test_native_verifier_failure_or_wrong_response_never_publishes(self):
        for outcome in [subprocess.CalledProcessError(1, ['verifier']), b'{"root":"wrong"}']:
            with self.subTest(outcome=outcome):
                with patch.object(ACQUIRE.subprocess, 'check_output', side_effect=outcome if isinstance(outcome, Exception) else None,
                                  return_value=outcome):
                    with self.assertRaises((subprocess.CalledProcessError, ValueError)):
                        self.run_acquire()
                self.assertFalse(self.output.exists())
                self.no_staging()

    def test_existing_or_aliased_output_is_preserved(self):
        self.output.mkdir()
        sentinel = self.output / 'keep'
        sentinel.write_bytes(b'existing user state')
        with self.assertRaisesRegex(ValueError, 'Output must be absent'):
            self.run_acquire()
        self.assertEqual(sentinel.read_bytes(), b'existing user state')
        alias = self.root / 'alias'
        alias.symlink_to(self.output, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, 'canonical'):
            ACQUIRE.acquire(self.package, self.lock, alias / 'new', self.cache)

    def test_atomic_publish_refuses_a_concurrently_created_output(self):
        staged = self.root / 'staged'
        staged.mkdir()
        self.output.mkdir()
        sentinel = self.output / 'keep'
        sentinel.write_bytes(b'preserve')
        with self.assertRaises(OSError):
            ACQUIRE.publish(staged, self.output)
        self.assertEqual(sentinel.read_bytes(), b'preserve')
        self.assertTrue(staged.is_dir())

    def test_unsafe_archives_never_escape_private_tree(self):
        for name, kind in [('../outside', tarfile.REGTYPE), ('package/../../outside', tarfile.REGTYPE),
                           ('/absolute', tarfile.REGTYPE), ('package/../index.js', tarfile.REGTYPE),
                           ('package/link', tarfile.SYMTYPE), ('package/link', tarfile.LNKTYPE),
                           ('package/pipe', tarfile.FIFOTYPE)]:
            with self.subTest(name=name, kind=kind):
                data = archive([(name, self.content, kind)])
                with self.assertRaises(ValueError):
                    ACQUIRE.extract_archive(data, integrity(data), 'node_modules/fixture', self.root / 'extract',
                                            {'fixture/index.js': self.inventory['files'][0]}, set())
        self.assertFalse((self.root / 'outside').exists())

    def test_integrity_valid_malicious_archive_cannot_publish_or_escape_staging(self):
        malicious = archive([('package/../../escaped', self.content, tarfile.REGTYPE)])
        self.cache_file.write_bytes(malicious)
        lock = json.loads(self.lock.read_bytes())
        lock['packages']['node_modules/fixture']['integrity'] = integrity(malicious)
        self.lock.write_text(json.dumps(lock))
        self.inventory['lockSha256'] = ACQUIRE.sha(self.lock.read_bytes())
        self.bind()
        with patch.object(ACQUIRE.subprocess, 'check_output', side_effect=AssertionError('unsafe tree')):
            with self.assertRaisesRegex(ValueError, 'Unsafe archive path'):
                self.run_acquire()
        self.assertFalse(self.output.exists())
        self.assertEqual(list(self.root.rglob('escaped')), [])
        self.no_staging()

    def test_publisher_dot_components_and_identical_duplicates_preserve_exact_bytes(self):
        data = archive([('package/./index.js', self.content, tarfile.REGTYPE),
                        ('package/index.js', self.content, tarfile.REGTYPE)])
        observed = set()
        target = self.root / 'extract'
        ACQUIRE.extract_archive(data, integrity(data), 'node_modules/fixture', target,
                                {'fixture/index.js': self.inventory['files'][0]}, observed)
        self.assertEqual(observed, {'fixture/index.js'})
        self.assertEqual((target / 'fixture/index.js').read_bytes(), self.content)

    def test_extra_changed_missing_and_conflicting_archive_members_are_refused(self):
        cases = [
            [('package/extra.js', self.content, tarfile.REGTYPE)],
            [('package/index.js', b'changed', tarfile.REGTYPE)],
            [('package/index.js', self.content, tarfile.REGTYPE), ('package/index.js', b'conflict', tarfile.REGTYPE)],
            [],
        ]
        for entries in cases:
            with self.subTest(entries=entries):
                data = archive(entries)
                with self.assertRaises(ValueError):
                    ACQUIRE.extract_archive(data, integrity(data), 'node_modules/fixture', self.root / 'extract',
                                            {'fixture/index.js': self.inventory['files'][0]}, set())

    def test_missing_inventory_members_refuses_before_native_execution(self):
        self.inventory['files'].append({'path': 'fixture/missing.js', 'bytes': 1, 'sha256': ACQUIRE.sha(b'x')})
        self.bind()
        with patch.object(ACQUIRE.subprocess, 'check_output', side_effect=AssertionError('incomplete tree')):
            with self.assertRaisesRegex(ValueError, 'complete host inventory'):
                self.run_acquire()
        self.assertFalse(self.output.exists())
        self.no_staging()

    def test_nonregistry_sources_and_redirects_are_refused(self):
        for url in ['http://registry.npmjs.org/x.tgz', 'https://example.org/x.tgz',
                    'https://registry.npmjs.org.evil/x.tgz', 'https://user@registry.npmjs.org/x.tgz',
                    'https://registry.npmjs.org/x.tgz?redirect=1']:
            with self.subTest(url=url), self.assertRaisesRegex(ValueError, 'HTTPS npm registry'):
                ACQUIRE.registry_url(url)
        with self.assertRaisesRegex(ValueError, 'redirects'):
            ACQUIRE.NoRedirect().redirect_request(None, None, 302, 'Found', {}, 'https://example.org/x.tgz')


if __name__ == '__main__':
    unittest.main()
