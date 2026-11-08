#!/usr/bin/env python3
"""Plan step 3. Split the base report text into one seed file per chapter, map [n] to keys and
tokenise units, so that content/*.md can be edited against it. Writes seed/ only and never
touches content/, so a re-run cannot overwrite an edited chapter."""
import os
import re
from pathlib import Path

B = Path(__file__).resolve().parent
# The extracted text of the base report, passed in by environment so no local path is written here.
SRC = Path(os.environ["NUSH2_SEED_SRC"])
OUT = B / "seed"
UNITS = [(r"(\d) ?MWth", r"\1 [[MWth]]"), (r"(\d) ?MWe", r"\1 [[MWe]]"), (r"(\d) ?kg/s", r"\1 [[kg/s]]"),
         (r"(\d) ?GJ/tCO₂", r"\1 [[GJ/t]]"), (r"(\d) ?t H₂/y", r"\1 [[tH2/y]]"), (r"(\d)°C", r"\1 [[degC]]")]
BREAKS = [("00_summary", r"^Summary$"), ("01_problem", r"^1\. Problem"), ("02_feasibility", r"^2\. Feasibility"),
          ("03_abatement", r"^3\. Net Emissions"), ("04_economics", r"^Economics\b"),
          ("05_discussion", r"^5\. Discussion"), ("06_refs", r"^6 References")]


def main():
    OUT.mkdir(exist_ok=True)
    lines = [re.sub(r"^\[normal\] ", "", x) for x in SRC.read_text(encoding="utf-8").splitlines()]
    part, parts = "cover", {"cover": []}
    for line in lines:
        for name, pat in BREAKS:
            if re.match(pat, line):
                part = name
                parts[part] = []
        parts[part].append(line)
    total = 0
    for name, body in parts.items():
        text = "\n\n".join(x for x in body if x.strip())
        text = re.sub(r"\[(\d+(?:,\d+)*)\]", lambda m: "[@" + ",".join("r" + n for n in m.group(1).split(",")) + "]", text)
        for pat, rep in UNITS:
            text = re.sub(pat, rep, text)
        (OUT / f"{name}.md").write_text(text + "\n", encoding="utf-8")
        total += len(text.split())
    src_words = sum(len(x.split()) for x in lines)
    print(f"seed words {total}, source words {src_words}, ratio {total / src_words:.3f}")


if __name__ == "__main__":
    main()
