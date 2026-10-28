"""Exercise the first-release macOS scope guard without touching a live install."""

from pathlib import Path
import stat
import platform
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
INSTALLER = ROOT / "scripts" / "install-macos.sh"


@unittest.skipUnless(platform.system() == "Darwin" and platform.machine() == "arm64",
                     "native macOS ARM64 installer guard")
class MacOSReleaseScopeTests(unittest.TestCase):
    def run_with_sw_vers(self, version: str) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory(prefix="mirket-scope-") as directory:
            root = Path(directory)
            fake_sw_vers = root / "sw_vers"
            fake_sw_vers.write_text(
                f"#!/bin/sh\nprintf '%s\\n' '{version}'\n", encoding="utf-8"
            )
            fake_sw_vers.chmod(fake_sw_vers.stat().st_mode | stat.S_IXUSR)
            # Substitute only the platform query in a disposable copy. This keeps
            # the production installer free of a version override while executing
            # the actual guard against controlled inputs.
            tested_installer = root / "install-macos.sh"
            source = INSTALLER.read_text(encoding="utf-8")
            source = source.replace("/usr/bin/sw_vers", str(fake_sw_vers))
            tested_installer.write_text(source, encoding="utf-8")
            tested_installer.chmod(tested_installer.stat().st_mode | stat.S_IXUSR)
            return subprocess.run(
                [str(tested_installer)],
                cwd=ROOT,
                text=True,
                capture_output=True,
                env={"PATH": "/usr/bin:/bin", "HOME": str(root)},
                check=False,
            )

    def test_refuses_unqualified_mac_version_before_argument_or_staging_work(self):
        for version in ("15.7", "26.5.3", "27.0", "unknown"):
            with self.subTest(version=version):
                result = self.run_with_sw_vers(version)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("qualified only for macOS 26.5.2 ARM64", result.stderr)
                self.assertNotIn("expected hashes", result.stderr)

    def test_qualified_version_reaches_normal_argument_validation(self):
        result = self.run_with_sw_vers("26.5.2")
        self.assertEqual(result.returncode, 2)
        self.assertIn("usage: install-macos.sh", result.stderr)
        self.assertNotIn("qualified only for macOS", result.stderr)


if __name__ == "__main__":
    unittest.main()
