#!/usr/bin/env python3
"""Check a wasm module with Python only: required exports, zero imports, size limit.

Usage: python scripts/check_wasm.py target/wasm32-unknown-unknown/release/nush2_wasm.wasm
Exit 0 on pass, 1 on fail.
"""
import sys
from pathlib import Path

REQUIRED = {"memory", "alloc", "dealloc", "call", "abi_version"}
MAX_BYTES = 3 * 1024 * 1024


def leb128(data, i):
    """Read one unsigned LEB128 value. Returns (value, next index)."""
    out = shift = 0
    while True:
        b = data[i]
        i += 1
        out |= (b & 0x7F) << shift
        if not b & 0x80:
            return out, i
        shift += 7


def name(data, i):
    n, i = leb128(data, i)
    return data[i:i + n].decode("utf-8"), i + n


def sections(data):
    """Yield (section id, payload start, payload end)."""
    if data[:4] != b"\0asm":
        raise SystemExit("not a wasm module")
    i = 8
    while i < len(data):
        sid = data[i]
        size, j = leb128(data, i + 1)
        yield sid, j, j + size
        i = j + size


def read_exports(data, start):
    count, i = leb128(data, start)
    out = []
    for _ in range(count):
        n, i = name(data, i)
        i += 1  # export kind
        _, i = leb128(data, i)
        out.append(n)
    return out


def main():
    path = Path(sys.argv[1])
    data = path.read_bytes()
    imports, exports = 0, []
    for sid, start, _end in sections(data):
        if sid == 2:
            imports, _ = leb128(data, start)
        elif sid == 7:
            exports = read_exports(data, start)
    missing = REQUIRED - set(exports)
    print(f"size {len(data)} bytes, imports {imports}, exports {sorted(exports)}")
    fails = []
    if missing:
        fails.append(f"missing exports {sorted(missing)}")
    if imports:
        fails.append(f"{imports} imports, expected zero")
    if len(data) >= MAX_BYTES:
        fails.append(f"size {len(data)} over {MAX_BYTES}")
    if fails:
        print("FAIL: " + "; ".join(fails))
        return 1
    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
