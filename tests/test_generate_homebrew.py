import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from scripts.generate_homebrew import render_formula


# Literal known SHA-256 values ensure the generator really hashes each archive.
ARCHIVES = [
    ("darwin_arm64", b"hello", "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"),
    ("darwin_amd64", b"world", "486ea46224d1bb4fb680f34f7c9ad96a8f24ec88be73ea8e5a6c65260e9cb8a7"),
    ("linux_arm64", b"", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
    ("linux_amd64", b"abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
]


def write_archives(dist):
    for platform, content, _ in ARCHIVES:
        (dist / f"zuuid_1.2.3_{platform}.tar.gz").write_bytes(content)


class FormulaTests(unittest.TestCase):
    def test_all_platform_archives_have_correct_checksums_and_urls(self):
        with tempfile.TemporaryDirectory() as directory:
            dist = Path(directory)
            write_archives(dist)
            formula = render_formula("1.2.3", dist)
        self.assertIn("class Zuuid < Formula", formula)
        self.assertIn("  on_macos do\n    on_arm do", formula)
        self.assertIn("  on_linux do\n    on_arm do", formula)
        self.assertEqual(formula.count("    on_intel do"), 2)
        self.assertNotIn("depends_on :macos", formula)
        self.assertNotIn("windows", formula)
        for platform, _, checksum in ARCHIVES:
            url = f"https://github.com/zhangbyby/zuuid/releases/download/v1.2.3/zuuid_1.2.3_{platform}.tar.gz"
            self.assertIn(f'url "{url}"\n      sha256 "{checksum}"', formula)
        self.assertIn('shell_output("#{bin}/zuuid --version")', formula)
        self.assertIn('bin.install "zuuid"', formula)

    def test_missing_any_archive_fails(self):
        for platform, _, _ in ARCHIVES:
            with self.subTest(platform=platform), tempfile.TemporaryDirectory() as directory:
                dist = Path(directory)
                write_archives(dist)
                (dist / f"zuuid_1.2.3_{platform}.tar.gz").unlink()
                with self.assertRaises(FileNotFoundError):
                    render_formula("1.2.3", dist)

    def test_unstable_or_invalid_versions_are_rejected(self):
        for version in ["1.2.3-rc.1", "1.2.3+build", "v1.2.3", "01.2.3", 'bad"version', ""]:
            with self.subTest(version=version), self.assertRaises(ValueError):
                render_formula(version, Path("not-used"))

    def test_command_writes_requested_destination(self):
        with tempfile.TemporaryDirectory() as directory:
            dist = Path(directory)
            write_archives(dist)
            output = dist / "formula" / "zuuid.rb"
            subprocess.run([
                sys.executable, "scripts/generate_homebrew.py", "--version", "1.2.3",
                "--dist", str(dist), "--output", str(output),
            ], check=True)
            self.assertEqual(output.read_text(encoding="utf-8"), render_formula("1.2.3", dist))


if __name__ == "__main__":
    unittest.main()
