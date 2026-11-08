#!/usr/bin/env python3
"""Build the wasm module and copy it to frontend/nush2.wasm, which stays git-ignored.

Usage: python scripts/build_wasm.py
"""
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "target/wasm32-unknown-unknown/release/nush2_wasm.wasm"
DST = ROOT / "frontend/nush2.wasm"


def main():
    subprocess.run(["cargo", "build", "--release", "-p", "nush2_wasm", "--target", "wasm32-unknown-unknown"],
                   cwd=ROOT, check=True)
    DST.parent.mkdir(exist_ok=True)
    shutil.copyfile(SRC, DST)
    print(f"copied {DST.stat().st_size} bytes to frontend/nush2.wasm")
    return 0


if __name__ == "__main__":
    sys.exit(main())
