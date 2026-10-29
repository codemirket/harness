"""Synthetic tests for the Sharp-free Video recipient packet assembler."""

import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/assemble-video-recipient.py"
SPEC = importlib.util.spec_from_file_location("assemble_video_recipient", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class VideoRecipientAssemblyTests(unittest.TestCase):
    def test_layout_receipt_cannot_hide_a_changed_dependency(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            base, current, receipt = (root / name for name in ["base.json", "current.json", "receipt.json"])
            base.write_text(json.dumps({"packages": {"": {"name": "old"}, "node_modules/demo": {"version": "1"}}}))
            current.write_text(json.dumps({"packages": {"": {"name": "new"}, "node_modules/demo": {"version": "1"}}}))
            def update_receipt():
                receipt.write_text(json.dumps({"base_lock_sha256": MODULE.digest(base)[1], "current_lock_sha256": MODULE.digest(current)[1]}))
            update_receipt()
            MODULE.verify_pi_layout(base, current, receipt)
            current.write_text(json.dumps({"packages": {"node_modules/demo": {"version": "2"}}}))
            with self.assertRaisesRegex(MODULE.AssemblyError, "identity changed"):
                MODULE.verify_pi_layout(base, current, receipt)
            update_receipt()
            with self.assertRaisesRegex(MODULE.AssemblyError, "dependency instances"):
                MODULE.verify_pi_layout(base, current, receipt)

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name).resolve()
        self.addCleanup(self.temporary.cleanup)
        self.npm = self.root / "npm"
        self.notices = self.root / "notices"
        self.fixture = self.root / "ffmpeg-fixture"
        self.npm.mkdir()
        self.notices.mkdir()
        (self.fixture / "bin").mkdir(parents=True)
        (self.fixture / "notices").mkdir()

        archive = b"exact synthetic npm archive"
        (self.npm / "demo-1.0.0.tgz").write_bytes(archive)
        (self.notices / "demo-1.0.0").mkdir()
        (self.notices / "demo-1.0.0" / "LICENSE").write_text("demo notice\n", encoding="utf-8")

        ffmpeg = b"current ffmpeg"
        ffprobe = b"current ffprobe"
        (self.fixture / "bin/ffmpeg").write_bytes(ffmpeg)
        (self.fixture / "bin/ffprobe").write_bytes(ffprobe)
        (self.fixture / "notices/LICENSE").write_text("GPL notice\n", encoding="utf-8")
        fixture_files = [
            {"path": "bin/ffmpeg", "bytes": len(ffmpeg), "sha256": hashlib.sha256(ffmpeg).hexdigest()},
            {"path": "bin/ffprobe", "bytes": len(ffprobe), "sha256": hashlib.sha256(ffprobe).hexdigest()},
            {"path": "notices/LICENSE", "bytes": 11, "sha256": hashlib.sha256(b"GPL notice\n").hexdigest()},
        ]
        (self.fixture / "delivery-manifest.json").write_text(
            json.dumps({"schema": 1, "status": "private_local_successor_fixture_not_admitted", "release_admitted": False, "files": fixture_files}),
            encoding="utf-8",
        )

        package_json = b'{"name":"demo","version":"1.0.0"}\n'
        self.inventory = self.root / "inventory.json"
        self.lock = self.root / "package-lock.json"
        lock_value = {"lockfileVersion": 3, "packages": {"node_modules/demo": {"version": "1.0.0"}}}
        self.lock.write_text(json.dumps(lock_value), encoding="utf-8")
        self.inventory.write_text(
            json.dumps(
                {
                    "schema": 1,
                    "status": "sharp_absent_four_format_development_candidate_not_release_admitted",
                    "release_admitted": False,
                    "target": "aarch64-apple-darwin",
                    "package_lock_sha256": hashlib.sha256(self.lock.read_bytes()).hexdigest(),
                    "package_instances": 1,
                    "runtime_inventory": {
                        "regular_files": 1,
                        "regular_bytes": len(package_json),
                        "files": [{"path": "runtime/video/node_modules/demo/package.json", "bytes": len(package_json), "sha256": hashlib.sha256(package_json).hexdigest()}],
                    },
                    "media_files": [
                        {"path": "runtime/video/bin/ffmpeg", "bytes": len(ffmpeg), "sha256": hashlib.sha256(ffmpeg).hexdigest()},
                        {"path": "runtime/video/bin/ffprobe", "bytes": len(ffprobe), "sha256": hashlib.sha256(ffprobe).hexdigest()},
                    ],
                }
            ),
            encoding="utf-8",
        )
        self.source_index = self.root / "SOURCE-INDEX.json"
        self.source_index.write_text(
            json.dumps(
                {
                    "published_npm_archives": [
                        {
                            "name": "demo",
                            "version": "1.0.0",
                            "release_relative_path": "source/video/npm/demo-1.0.0.tgz",
                            "bytes": len(archive),
                            "sha256": hashlib.sha256(archive).hexdigest(),
                        }
                    ],
                    "complete_upstream_archives": [],
                }
            ),
            encoding="utf-8",
        )
        self.notice_index = self.root / "NOTICE-INDEX.json"
        self.notice_index.write_text(
            json.dumps(
                {
                    "packages": [
                        {
                            "name": "demo",
                            "version": "1.0.0",
                            "notice_directory": "notices/packages/demo-1.0.0",
                            "upstream_package_notice_members": [],
                        }
                    ]
                }
            ),
            encoding="utf-8",
        )
        self.release_manifest = self.root / "current-release-manifest.json"
        self.release_manifest.write_text(
            json.dumps(
                {
                    "files": [
                        {"relative_path": "runtime/video/dist/video-export.js", "byte_len": 10, "sha256": "1" * 64, "mode": 384},
                        {"relative_path": "runtime/video/dist/video-main.js", "byte_len": 11, "sha256": "2" * 64, "mode": 384},
                        {"relative_path": "runtime/video/dist/video-policy.js", "byte_len": 12, "sha256": "3" * 64, "mode": 384},
                    ]
                }
            ),
            encoding="utf-8",
        )
        self.supplemental = self.root / "supplemental-notices"
        for directory in MODULE.SUPPLEMENTAL_NOTICE_DIRS:
            (self.supplemental / directory).mkdir(parents=True)
            (self.supplemental / directory / "NOTICE.txt").write_text(f"{directory}\n", encoding="utf-8")

    def assemble(self, output: Path) -> dict:
        return MODULE.assemble(
            inventory_path=self.inventory,
            package_lock_path=self.lock,
            source_index_path=self.source_index,
            notice_index_path=self.notice_index,
            npm_archive_root=self.npm,
            package_notice_root=self.notices,
            ffmpeg_fixture=self.fixture,
            output=output,
            strict_current=False,
        )

    def test_assembles_exact_unadmitted_packet(self) -> None:
        output = self.root / "output"
        receipt = self.assemble(output)
        self.assertFalse(receipt["release_admitted"])
        self.assertEqual(receipt["package_instances"], 1)
        self.assertEqual(receipt["distinct_npm_archives"], 1)
        self.assertEqual((output / "source/video/npm/demo-1.0.0.tgz").read_bytes(), b"exact synthetic npm archive")
        self.assertEqual((output / "source/ffmpeg/bin/ffmpeg").read_bytes(), b"current ffmpeg")
        self.assertFalse(any("sharp" in path.as_posix().lower() for path in (output / "source/video").rglob("*")))

    def test_changed_archive_is_refused_before_output(self) -> None:
        (self.npm / "demo-1.0.0.tgz").write_bytes(b"changed")
        with self.assertRaisesRegex(MODULE.AssemblyError, "npm archive identity mismatch"):
            self.assemble(self.root / "output")
        self.assertFalse((self.root / "output").exists())

    def test_missing_archive_is_refused_before_output(self) -> None:
        (self.npm / "demo-1.0.0.tgz").unlink()
        with self.assertRaises((MODULE.AssemblyError, FileNotFoundError)):
            self.assemble(self.root / "output")
        self.assertFalse((self.root / "output").exists())

    def test_stale_ffmpeg_mapping_is_refused(self) -> None:
        manifest = json.loads((self.fixture / "delivery-manifest.json").read_text(encoding="utf-8"))
        manifest["files"][0]["sha256"] = "0" * 64
        (self.fixture / "delivery-manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
        with self.assertRaisesRegex(MODULE.AssemblyError, "FFmpeg fixture binary/source mapping is stale"):
            self.assemble(self.root / "output")
        self.assertFalse((self.root / "output").exists())

    def test_corrupt_ffmpeg_binary_is_refused_before_copy(self) -> None:
        (self.fixture / "bin/ffmpeg").write_bytes(b"corrupt ffmpeg")
        with self.assertRaisesRegex(MODULE.AssemblyError, "FFmpeg fixture binary/source mapping is stale"):
            self.assemble(self.root / "output")
        self.assertFalse((self.root / "output").exists())

    def test_changed_ffmpeg_source_is_refused_with_unchanged_binaries(self) -> None:
        source = self.fixture / "source/ffmpeg-source.tar.gz"
        source.parent.mkdir()
        original = b"exact corresponding source archive"
        source.write_bytes(original)
        manifest_path = self.fixture / "delivery-manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["files"].append({
            "path": "source/ffmpeg-source.tar.gz",
            "bytes": len(original),
            "sha256": hashlib.sha256(original).hexdigest(),
        })
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
        source.write_bytes(b"changed corresponding source archive")
        with self.assertRaisesRegex(MODULE.AssemblyError, "FFmpeg fixture file does not match"):
            self.assemble(self.root / "output")
        self.assertFalse((self.root / "output").exists())

    def test_unsafe_notice_path_is_refused(self) -> None:
        notice = json.loads(self.notice_index.read_text(encoding="utf-8"))
        notice["packages"][0]["notice_directory"] = "notices/packages/../escape"
        self.notice_index.write_text(json.dumps(notice), encoding="utf-8")
        with self.assertRaisesRegex(MODULE.AssemblyError, "unsafe package notice directory"):
            self.assemble(self.root / "output")
        self.assertFalse((self.root / "output").exists())

    def test_symlinked_archive_root_is_refused(self) -> None:
        linked = self.root / "linked-npm"
        linked.symlink_to(self.npm, target_is_directory=True)
        with self.assertRaisesRegex(MODULE.AssemblyError, "directory is missing, linked"):
            MODULE.assemble(
                inventory_path=self.inventory,
                package_lock_path=self.lock,
                source_index_path=self.source_index,
                notice_index_path=self.notice_index,
                npm_archive_root=linked,
                package_notice_root=self.notices,
                ffmpeg_fixture=self.fixture,
                output=self.root / "output",
                strict_current=False,
            )
        self.assertFalse((self.root / "output").exists())

    def test_release_manifest_must_match_real_selected_files(self) -> None:
        release_root = self.root / "release-source"
        (release_root / "runtime/video/dist").mkdir(parents=True)
        (release_root / "runtime/video/bin").mkdir(parents=True)
        main = b"current main"
        ffmpeg = b"current binary"
        (release_root / "runtime/video/dist/video-main.js").write_bytes(main)
        (release_root / "runtime/video/bin/ffmpeg").write_bytes(ffmpeg)
        manifest = release_root / "manifest.json"
        manifest.write_text(
            json.dumps(
                {
                    "files": [
                        {
                            "relative_path": "runtime/video/dist/video-main.js",
                            "byte_len": len(main),
                            "sha256": hashlib.sha256(main).hexdigest(),
                        },
                        {
                            "relative_path": "runtime/video/bin/ffmpeg",
                            "byte_len": len(ffmpeg),
                            "sha256": hashlib.sha256(ffmpeg).hexdigest(),
                        },
                    ]
                }
            ),
            encoding="utf-8",
        )
        inventory = {
            "runtime_inventory": {
                "files": [
                    {
                        "path": "runtime/video/dist/video-main.js",
                        "bytes": len(main),
                        "sha256": hashlib.sha256(main).hexdigest(),
                    }
                ]
            }
        }
        media = [{"path": "runtime/video/bin/ffmpeg", "bytes": len(ffmpeg), "sha256": hashlib.sha256(ffmpeg).hexdigest()}]
        MODULE.verify_release_source_files(manifest, release_root, inventory, media, True)
        (release_root / "runtime/video/bin/ffmpeg").write_bytes(b"drifted")
        with self.assertRaisesRegex(MODULE.AssemblyError, "does not match manifest"):
            MODULE.verify_release_source_files(manifest, release_root, inventory, media, True)

    def test_existing_output_is_preserved(self) -> None:
        output = self.root / "output"
        output.mkdir()
        sentinel = output / "sentinel"
        sentinel.write_text("preserve", encoding="utf-8")
        with self.assertRaisesRegex(MODULE.AssemblyError, "output already exists"):
            self.assemble(output)
        self.assertEqual(sentinel.read_text(encoding="utf-8"), "preserve")

    def test_first_party_rebind_is_recorded_separately_from_vendor_pins(self) -> None:
        output = self.root / "output"
        MODULE.assemble(
            inventory_path=self.inventory,
            package_lock_path=self.lock,
            source_index_path=self.source_index,
            notice_index_path=self.notice_index,
            npm_archive_root=self.npm,
            package_notice_root=self.notices,
            ffmpeg_fixture=self.fixture,
            release_source_manifest=self.release_manifest,
            output=output,
            strict_current=False,
        )
        index = json.loads((output / "SOURCE-INDEX.json").read_text(encoding="utf-8"))
        self.assertEqual(
            {entry["path"] for entry in index["first_party_video_rebind"]["files"]},
            {
                "runtime/video/dist/video-export.js",
                "runtime/video/dist/video-main.js",
                "runtime/video/dist/video-policy.js",
            },
        )
        self.assertEqual(index["current_release_manifest_sha256"], hashlib.sha256(self.release_manifest.read_bytes()).hexdigest())

    def test_supplemental_notices_and_metadata_are_retained(self) -> None:
        old_notice = json.loads(self.notice_index.read_text(encoding="utf-8"))
        old_notice.update(
            {
                "supplemental_components": ["embedded font license"],
                "embedded_font_faces": 1,
                "embedded_font_packages": 1,
                "embedded_fonts_license": "OFL-1.1",
                "unresolved_embedded_font_faces": [],
                "scope_limit": "synthetic",
            }
        )
        self.notice_index.write_text(json.dumps(old_notice), encoding="utf-8")
        output = self.root / "output"
        MODULE.assemble(
            inventory_path=self.inventory,
            package_lock_path=self.lock,
            source_index_path=self.source_index,
            notice_index_path=self.notice_index,
            npm_archive_root=self.npm,
            package_notice_root=self.notices,
            ffmpeg_fixture=self.fixture,
            release_source_manifest=self.release_manifest,
            supplemental_notice_root=self.supplemental,
            output=output,
            strict_current=False,
        )
        index = json.loads((output / "NOTICE-INDEX.json").read_text(encoding="utf-8"))
        self.assertEqual(index["embedded_fonts_license"], "OFL-1.1")
        self.assertEqual(len(index["supplemental_notices"]), 4)
        self.assertTrue((output / "notices/embedded-fonts/NOTICE.txt").is_file())


if __name__ == "__main__":
    unittest.main()
