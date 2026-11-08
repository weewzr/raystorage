"""Write THIRD_PARTY_NOTICES.md from the licence files of the crates compiled into the engine.

Usage: python notices.py <cargo registry src dir> <repo>
"""
import sys
from pathlib import Path

REG, REPO = Path(sys.argv[1]), Path(sys.argv[2])
# crate, version, licence offered, licence elected here, licence file shipped
CRATES = [
    ("itoa", "1.0.18", "MIT OR Apache-2.0", "MIT", "LICENSE-MIT"),
    ("memchr", "2.8.3", "Unlicense OR MIT", "Unlicense", "UNLICENSE"),
    ("serde", "1.0.229", "MIT OR Apache-2.0", "MIT", "LICENSE-MIT"),
    ("serde_core", "1.0.229", "MIT OR Apache-2.0", "MIT", "LICENSE-MIT"),
    ("serde_json", "1.0.154", "MIT OR Apache-2.0", "MIT", "LICENSE-MIT"),
    ("seuif97", "2.3.8", "MIT", "MIT", "LICENSE"),
    ("zmij", "1.0.23", "MIT", "MIT", "LICENSE-MIT"),
]
BUILD_ONLY = [
    ("serde_derive", "1.0.229", "MIT OR Apache-2.0"),
    ("proc-macro2", "1.0.107", "MIT OR Apache-2.0"),
    ("quote", "1.0.47", "MIT OR Apache-2.0"),
    ("syn", "3.0.9", "MIT OR Apache-2.0"),
    ("unicode-ident", "1.0.27", "(MIT OR Apache-2.0) AND Unicode-3.0"),
]

out = [
    "# Third-party notices",
    "",
    "The NUS-H2 engine is MIT licensed (see `LICENSE`). The engine, and the WebAssembly module",
    "`nush2.wasm` that the site serves, compile in the Rust crates below, each under its own licence.",
    "Where a crate offers a choice, the licence named as elected is the one this project uses, and its",
    "text follows. Versions are the ones `Cargo.lock` pins.",
    "",
    "| Crate | Version | Offered | Elected |",
    "|---|---|---|---|",
]
for name, ver, offered, elected, _ in CRATES:
    out.append(f"| {name} | {ver} | {offered} | {elected} |")
out += [
    "",
    "Build-time crates, which are the procedural macro serde_derive and the crates it uses; they run while the engine compiles, and none of their own code is in the module: "
    + ", ".join(f"{n} {v} ({lic})" for n, v, lic in BUILD_ONLY) + ".",
    "",
    "Figures and photographs in the report and the presentation that are credited to other sources,",
    "and any other third-party image in them, belong to their owners and are not covered by the MIT",
    "licence; see `README.md`, section \"Licence and credit\".",
    "",
]
for name, ver, _, elected, lic in CRATES:
    text = (REG / f"{name}-{ver}" / lic).read_text(encoding="utf-8").strip()
    out += [f"## {name} {ver}, {elected}", "", "```text", text, "```", ""]
(REPO / "THIRD_PARTY_NOTICES.md").write_text("\n".join(out), encoding="utf-8", newline="\n")
print(f"wrote THIRD_PARTY_NOTICES.md, {len(CRATES)} crates")
