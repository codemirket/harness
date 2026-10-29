"""Focused tests for the repository-owned Chrome onboarding pin generator."""

import importlib.util
from pathlib import Path
import tempfile
import unittest
import zipfile


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/generate-chrome-pin.py"
SPEC = importlib.util.spec_from_file_location("generate_chrome_pin", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)
VERSION = "156.0.8078.73"
URL = "https://storage.googleapis.com/chrome-for-testing-public/156.0.8078.73/mac-arm64/chrome-mac-arm64.zip"


class ChromePinGeneratorTests(unittest.TestCase):
    def make_archive(self, root: Path, members: list[tuple[str, bytes, int]]) -> Path:
        archive = root / "chrome-mac-arm64.zip"
        with zipfile.ZipFile(archive, "w") as output:
            for name, data, mode in members:
                info = zipfile.ZipInfo(name)
                info.create_system = 3
                info.external_attr = mode << 16
                output.writestr(info, data)
        return archive

    def test_maps_regular_files_and_framework_links(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = self.make_archive(
                Path(directory),
                [
                    (
                        "chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
                        b"executable",
                        0o100755,
                    ),
                    ("chrome-mac-arm64/ABOUT", b"about", 0o100644),
                    (
                        "chrome-mac-arm64/Google Chrome for Testing.app/Contents/Frameworks/Google Chrome for Testing Framework.framework/Resources",
                        b"Versions/Current/Resources",
                        0o120777,
                    ),
                ],
            )
            result = MODULE.generate(archive, VERSION, URL)

        self.assertEqual(result["file_count"], 2)
        self.assertEqual(result["link_count"], 1)
        self.assertEqual(result["files"][0]["mode"], 0o700)
        self.assertEqual(result["files"][1]["mode"], 0o600)
        self.assertEqual(result["links"][0]["relative_path"].split("/", 3)[-1], "Google Chrome for Testing.app/Contents/Frameworks/Google Chrome for Testing Framework.framework/Resources")
        self.assertEqual(result["links"][0]["target"], "Versions/Current/Resources")

    def test_rejects_unsafe_archive_member_and_target(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = self.make_archive(
                root,
                [("chrome-mac-arm64/../escape", b"bad", 0o100644)],
            )
            with self.assertRaisesRegex(ValueError, "unsafe archive member"):
                MODULE.generate(archive, VERSION, URL)

            for target in ("../escape", "alias//target", "alias/./target", "alias\\target"):
                archive = self.make_archive(
                    root,
                    [
                        (
                            "chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
                            b"executable",
                            0o100755,
                        ),
                        ("chrome-mac-arm64/Resources", target.encode(), 0o120777),
                    ],
                )
                with self.assertRaisesRegex(ValueError, "unsafe symlink target"):
                    MODULE.generate(archive, VERSION, URL)

    def test_requires_chrome_executable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = self.make_archive(
                Path(directory), [("chrome-mac-arm64/ABOUT", b"about", 0o100644)]
            )
            with self.assertRaisesRegex(ValueError, "Chrome executable missing"):
                MODULE.generate(archive, VERSION, URL)

    def test_rejects_noncanonical_members_and_file_directory_conflicts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            executable = (
                "chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
                b"executable",
                0o100755,
            )
            for unsafe in (
                "chrome-mac-arm64/../escape/",
                "chrome-mac-arm64/alias/./file",
                "chrome-mac-arm64/alias//file",
                "chrome-mac-arm64/alias\\file",
            ):
                archive = self.make_archive(root, [executable, (unsafe, b"bad", 0o100644)])
                with self.assertRaisesRegex(ValueError, "unsafe archive member"):
                    MODULE.generate(archive, VERSION, URL)

            archive = self.make_archive(
                root,
                [executable, ("chrome-mac-arm64/conflict/", b"", 0o40755), ("chrome-mac-arm64/conflict", b"file", 0o100644)],
            )
            with self.assertRaisesRegex(ValueError, "duplicate archive member"):
                MODULE.generate(archive, VERSION, URL)

    def test_requires_executable_mode_and_official_version_url_pair(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = self.make_archive(
                Path(directory),
                [
                    (
                        "chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
                        b"not executable",
                        0o100644,
                    )
                ],
            )
            with self.assertRaisesRegex(ValueError, "not executable"):
                MODULE.generate(archive, VERSION, URL)
            with self.assertRaisesRegex(ValueError, "official Mac arm64 URL"):
                MODULE.generate(archive, VERSION, "https://example.test/chrome.zip")
            with self.assertRaisesRegex(ValueError, "invalid Chrome for Testing version"):
                MODULE.generate(archive, "156.0.8078", URL)


if __name__ == "__main__":
    unittest.main()
