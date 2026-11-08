#!/usr/bin/env python3
"""Rebuild the report end to end with no hand edits.

  python report/_build/run_all.py            refresh numbers if stale, rebuild, export, QA
  python report/_build/run_all.py --no-qa    skip the QA step

Run with the system Python, which has python-docx, lxml, pymupdf, PIL and matplotlib.
"""
import shutil
import subprocess
import sys
from pathlib import Path

B = Path(__file__).resolve().parent
ROOT = B.parents[1]
DOCX = ROOT / "report" / "NUSH2_Final_Report.docx"
PDF = ROOT / "report" / "NUSH2_Final_Report.pdf"
RAW = B / "tmp" / "report_raw.docx"


def run(args, **kw):
    print("+", " ".join(str(a) for a in args), flush=True)
    return subprocess.run([str(a) for a in args], cwd=ROOT, **kw)


def main():
    if run([sys.executable, "scripts/build_numbers.py", "--verify"]).returncode != 0:
        if run([sys.executable, "scripts/build_numbers.py"]).returncode != 0:
            return 1
    numbers = ROOT / "build" / "numbers.json"
    chart = ROOT / "presentation" / "figures" / "Chart_Cost_Grid_L5.png"
    if not chart.is_file() or chart.stat().st_mtime < numbers.stat().st_mtime:
        if run([sys.executable, "presentation/figures/build_figures.py"]).returncode != 0:
            print("figure rebuild failed; the existing figures stay in use")
    for step in ("resolve_numbers.py", "build_report.py"):
        if run([sys.executable, B / step]).returncode != 0:
            return 1
    code = run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(B / "word_run.ps1"),
                "-InDocx", str(RAW), "-OutDocx", str(DOCX), "-OutPdf", str(PDF), "-TimeoutSec", "300"]).returncode
    if code != 0:
        print(f"Word step failed with code {code}")
        return code
    (B / "logs" / "pdf_route.txt").write_text("P1, ExportAsFixedFormat with heading bookmarks", encoding="utf-8")
    if "--no-qa" not in sys.argv:
        return run([sys.executable, B / "qa_report.py"]).returncode
    return 0


if __name__ == "__main__":
    sys.exit(main())
