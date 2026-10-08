"""Archive traversal budgets cover unselected data before skill publication."""
import gzip
import io
import tarfile
from unittest import mock
import unittest

from test_catalog import CatalogFixture, archive_bytes, catalog


class ArchiveBudgetTests(CatalogFixture):
    def oversized_archive(self):
        return archive_bytes(self.members + [
            ('repo/unrelated.bin', b'\0' * (64 * 1024), tarfile.REGTYPE)])

    def assert_rejected_before_install(self, blob, pattern):
        with mock.patch.object(catalog, 'download', return_value=blob):
            with self.assertRaisesRegex(ValueError, pattern):
                catalog.install_many(self.data, [self.entry], self.project, 'both')
        self.assert_project_empty()

    def raw_members(self):
        raw = gzip.decompress(archive_bytes(self.members))
        # All fixture members are regular files; omit only writer-added padding.
        length = sum(512 + ((len(content) + 511) // 512) * 512
                     for _, content, _ in self.members)
        return raw[:length]

    def test_unselected_compressible_data_is_bounded_before_install(self):
        blob = self.oversized_archive()
        self.assertLess(len(blob), 4096)
        with mock.patch.object(catalog, 'MAX_EXPANDED_ARCHIVE', 16 * 1024, create=True):
            with mock.patch.object(catalog, 'download', return_value=blob):
                with self.assertRaisesRegex(ValueError, 'expanded size limit'):
                    catalog.install_many(self.data, [self.entry], self.project, 'both')
        self.assert_project_empty()

    def test_unselected_empty_members_are_bounded(self):
        members = self.members + [
            ('repo/empty-' + str(n), b'', tarfile.REGTYPE) for n in range(20)]
        with mock.patch.object(catalog, 'MAX_ARCHIVE_MEMBERS', 8, create=True):
            with self.assertRaisesRegex(ValueError, 'member count limit'):
                catalog.read_archive(archive_bytes(members), self.entry)

    def test_metadata_headers_count_before_they_are_resolved(self):
        raw = gzip.decompress(archive_bytes(self.members))
        for kind in (tarfile.XGLTYPE, tarfile.XHDTYPE, tarfile.GNUTYPE_LONGLINK):
            metadata = tarfile.TarInfo('repo/metadata')
            metadata.type = kind
            blob = gzip.compress(metadata.tobuf(format=tarfile.GNU_FORMAT) * 20 + raw)
            with self.subTest(kind=kind):
                with mock.patch.object(catalog, 'MAX_ARCHIVE_MEMBERS', len(self.members)):
                    self.assert_rejected_before_install(blob, 'member count limit')
                with mock.patch.object(catalog, 'MAX_ARCHIVE_MEMBERS', len(self.members) + 20):
                    self.assertEqual(catalog.read_archive(blob, self.entry), self.files)

    def test_metadata_nesting_has_an_independent_inclusive_limit(self):
        raw = gzip.decompress(archive_bytes(self.members))
        metadata = tarfile.TarInfo('repo/metadata')
        metadata.type = tarfile.XGLTYPE
        header = metadata.tobuf(format=tarfile.GNU_FORMAT)
        with mock.patch.object(catalog, 'MAX_ARCHIVE_METADATA_DEPTH', 4):
            self.assertEqual(catalog.read_archive(gzip.compress(header * 3 + raw), self.entry), self.files)
            self.assert_rejected_before_install(gzip.compress(header * 4 + raw), 'metadata nesting limit')

    def test_pax_metadata_expansion_is_bounded(self):
        buffer = io.BytesIO()
        with tarfile.open(fileobj=buffer, mode='w:gz', format=tarfile.PAX_FORMAT) as archive:
            member = tarfile.TarInfo('repo/unrelated')
            member.pax_headers = {'comment': 'x' * (64 * 1024)}
            archive.addfile(member)
        with mock.patch.object(catalog, 'MAX_EXPANDED_ARCHIVE', 16 * 1024, create=True):
            with self.assertRaisesRegex(ValueError, 'expanded size limit'):
                catalog.read_archive(buffer.getvalue(), self.entry)

    def test_trailing_gzip_expansion_is_bounded_after_tar_end(self):
        blob = archive_bytes(self.members) + gzip.compress(b'\0' * (64 * 1024))
        with mock.patch.object(catalog, 'MAX_EXPANDED_ARCHIVE', 16 * 1024, create=True):
            with self.assertRaisesRegex(ValueError, 'expanded size limit'):
                catalog.read_archive(blob, self.entry)

    def test_exact_expanded_and_member_limits_accept_valid_payload(self):
        blob = archive_bytes(self.members)
        with mock.patch.object(catalog, 'MAX_EXPANDED_ARCHIVE', len(gzip.decompress(blob)), create=True):
            with mock.patch.object(catalog, 'MAX_ARCHIVE_MEMBERS', len(self.members), create=True):
                self.assertEqual(catalog.read_archive(blob, self.entry), self.files)

    def test_truncated_or_corrupt_gzip_is_not_a_successful_payload(self):
        blob = archive_bytes(self.members)
        for invalid in (blob[:-8], blob[:-8] + bytes([blob[-8] ^ 1]) + blob[-7:]):
            with self.subTest(tail=invalid[-8:]), mock.patch.object(catalog, 'download', return_value=invalid):
                with self.assertRaises((ValueError, tarfile.TarError)):
                    catalog.install_many(self.data, [self.entry], self.project, 'both')
                self.assert_project_empty()

    def test_missing_partial_or_invalid_tar_headers_do_not_publish(self):
        for suffix in (b'', b'partial header', b'X' * 512):
            with self.subTest(suffix_length=len(suffix)):
                self.assert_rejected_before_install(gzip.compress(self.raw_members() + suffix),
                                                    'Invalid or truncated tar')

    def test_two_complete_zero_end_blocks_are_required(self):
        for suffix in (b'\0' * 512, b'\0' * 1023, b'\0' * 512 + b'X' * 512):
            with self.subTest(suffix_length=len(suffix)):
                self.assert_rejected_before_install(gzip.compress(self.raw_members() + suffix),
                                                    'two zero end blocks')
        self.assertEqual(catalog.read_archive(gzip.compress(self.raw_members() + b'\0' * 1024),
                                              self.entry), self.files)

    def test_nonzero_tail_is_rejected_in_read_ahead_or_later_gzip_member(self):
        blob = archive_bytes(self.members)
        variants = (gzip.compress(self.raw_members() + b'\0' * 1024 + b'garbage'),
                    blob + gzip.compress(b'garbage'))
        for invalid in variants:
            with self.subTest(compressed_length=len(invalid)):
                self.assert_rejected_before_install(invalid, 'nonzero trailing data')
        self.assertEqual(catalog.read_archive(blob + gzip.compress(b'\0' * 512), self.entry), self.files)

    def test_old_gnu_sparse_is_rejected_before_truncated_extension_parser(self):
        member = tarfile.TarInfo('repo/unrelated-sparse')
        member.type = tarfile.GNUTYPE_SPARSE
        header = bytearray(member.tobuf(format=tarfile.GNU_FORMAT))
        header[482] = 1  # Extended sparse map follows, deliberately absent.
        header[148:156] = b' ' * 8
        header[148:156] = ('%06o\0 ' % sum(header)).encode()
        self.assert_rejected_before_install(gzip.compress(self.raw_members() + header), 'Sparse.*unsupported')

    def test_pax_sparse_dialects_are_rejected_before_map_parsing(self):
        for fields in ({'GNU.sparse.size': '100'}, {'GNU.sparse.map': ''},
                       {'GNU.sparse.major': '1', 'GNU.sparse.minor': '0'},
                       {'GNU.sparse.name': 'repo/unrelated-sparse'}):
            buffer = io.BytesIO()
            with tarfile.open(fileobj=buffer, mode='w', format=tarfile.PAX_FORMAT) as archive:
                member = tarfile.TarInfo('repo/unrelated-sparse')
                member.pax_headers = fields
                archive.addfile(member)
            with self.subTest(fields=fields):
                self.assert_rejected_before_install(gzip.compress(self.raw_members() + buffer.getvalue()),
                                                    'Sparse.*unsupported')

    def test_negative_member_and_invalid_pax_sizes_do_not_publish(self):
        member = tarfile.TarInfo('repo/unrelated-negative')
        member.size = -1
        self.assert_rejected_before_install(
            gzip.compress(self.raw_members() + member.tobuf(format=tarfile.GNU_FORMAT) + b'\0' * 1024),
            'Invalid.*size')
        for value in ('garbage', '-1', '+1', '1.5'):
            buffer = io.BytesIO()
            with tarfile.open(fileobj=buffer, mode='w', format=tarfile.PAX_FORMAT) as archive:
                member = tarfile.TarInfo('repo/unrelated')
                member.pax_headers = {'size': value}
                archive.addfile(member)
            with self.subTest(size=value):
                self.assert_rejected_before_install(
                    gzip.compress(self.raw_members() + buffer.getvalue()), 'Invalid.*size')

    def test_ordinary_pax_unicode_and_gnu_long_names_keep_payload(self):
        name = 'repo/skills/fixture/references/' + 'long-' * 30 + 'ışık.md'
        for archive_format in (tarfile.PAX_FORMAT, tarfile.GNU_FORMAT):
            buffer = io.BytesIO()
            with tarfile.open(fileobj=buffer, mode='w:gz', format=archive_format) as archive:
                for path, content, kind in self.members + [(name, b'guide', tarfile.REGTYPE)]:
                    member = tarfile.TarInfo(path)
                    member.size = len(content)
                    archive.addfile(member, io.BytesIO(content))
            with self.subTest(format=archive_format):
                result = catalog.read_archive(buffer.getvalue(), self.entry)
                self.assertEqual(result, dict(self.files, **{name.split('fixture/')[1]: b'guide'}))


if __name__ == '__main__':
    unittest.main()
