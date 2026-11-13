#!/usr/bin/env python3
"""Generate a stable macOS/Linux formula from the actual release archives."""

import argparse
import hashlib
import re
from pathlib import Path


def render_formula(version: str, dist: Path) -> str:
    if not re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", version):
        raise ValueError("Homebrew stable releases require a numeric major.minor.patch version")
    platforms = []
    for platform, condition in [("darwin", "on_macos"), ("linux", "on_linux")]:
        architectures = []
        for arch, architecture in [("arm64", "on_arm"), ("amd64", "on_intel")]:
            archive = dist / f"zuuid_{version}_{platform}_{arch}.tar.gz"
            with archive.open("rb") as source:
                checksum = hashlib.file_digest(source, "sha256").hexdigest()
            url = f"https://github.com/zhangbyby/zuuid/releases/download/v{version}/{archive.name}"
            architectures.append(f'''    {architecture} do
      url "{url}"
      sha256 "{checksum}"
    end''')
        platforms.append(f"  {condition} do\n" + "\n".join(architectures) + "\n  end")
    downloads = "\n\n".join(platforms)
    return f'''class Zuuid < Formula
  desc "Generate UUID v4 and v7 identifiers"
  homepage "https://github.com/zhangbyby/zuuid"
  version "{version}"
  license "MIT"

{downloads}

  def install
    bin.install "zuuid"
  end

  test do
    assert_match version.to_s, shell_output("#{{bin}}/zuuid --version")
    assert_match(/\\A[0-9a-f]{{8}}-[0-9a-f]{{4}}-4[0-9a-f]{{3}}-[89ab][0-9a-f]{{3}}-[0-9a-f]{{12}}\\n\\z/,
                 shell_output("#{{bin}}/zuuid"))
  end
end
'''


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--dist", type=Path, default=Path("dist"))
    parser.add_argument("--output", type=Path, default=Path("dist/zuuid.rb"))
    args = parser.parse_args()
    formula = render_formula(args.version, args.dist)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(formula, encoding="utf-8")


if __name__ == "__main__":
    main()
