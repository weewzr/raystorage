#!/usr/bin/env python3
"""Build the wasm module with the wasm-release profile and copy it out, by
default to frontend/nush2.wasm, which stays git-ignored.

CARGO_TARGET_DIR is honoured. GIT_SHA, when set, becomes the commit id the
module reports; otherwise it reports "local". Source paths that panic messages
carry (this checkout, the cargo home, the rustup home) are remapped to short
neutral prefixes, so the module names no folder of the machine that built it.

Usage: python scripts/build_wasm.py [--out PATH]
"""
import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DST = ROOT / "frontend/nush2.wasm"
TARGET = "wasm32-unknown-unknown"
PROFILE = "wasm-release"


def remap_flags():
    home = Path.home()
    pairs = [
        (ROOT, "/nush2"),
        (Path(os.environ.get("CARGO_HOME") or home / ".cargo"), "/cargo"),
        (Path(os.environ.get("RUSTUP_HOME") or home / ".rustup"), "/rustup"),
    ]
    return [f"--remap-path-prefix={src}={dst}" for src, dst in pairs]


def main(argv=None):
    ap = argparse.ArgumentParser(description="Build nush2_wasm and copy the module out")
    ap.add_argument("--out", type=Path, default=DST)
    args = ap.parse_args(argv)
    target_dir = Path(os.environ.get("CARGO_TARGET_DIR") or "target")
    if not target_dir.is_absolute():
        target_dir = ROOT / target_dir
    env = dict(os.environ)
    encoded = env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    flags = encoded.split("\x1f") if encoded else env.pop("RUSTFLAGS", "").split()
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join([f for f in flags if f] + remap_flags())
    subprocess.run(["cargo", "build", "--profile", PROFILE, "-p", "nush2_wasm", "--target", TARGET],
                   cwd=ROOT, env=env, check=True)
    src = target_dir / TARGET / PROFILE / "nush2_wasm.wasm"
    out = args.out if args.out.is_absolute() else Path.cwd() / args.out
    out.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(src, out)
    print(f"copied {out.stat().st_size} bytes to {out.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
