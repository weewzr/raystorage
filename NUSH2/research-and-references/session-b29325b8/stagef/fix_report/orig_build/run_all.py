#!/usr/bin/env python3
"""Rebuild the report end to end with no hand edits.

  python report/_build/run_all.py            refresh numbers if stale, rebuild, export, QA
  python report/_build/run_all.py --no-qa    skip the QA step
  python report/_build/run_all.py --no-word  rebuild the raw docx only

Word jobs run one at a time across every builder on this machine. Set NUSH2_WORD_LOCK to the
shared lock directory: this script takes it with mkdir, waits while another job holds it,
treats a lock older than 40 minutes as stale, and removes it when the Word step ends.

Run with the system Python, which has python-docx, lxml, pymupdf, PIL and matplotlib.
"""
import os
import re
import shutil
import subprocess
import sys
import time
import zipfile
from pathlib import Path

B = Path(__file__).resolve().parent
ROOT = B.parents[1]
DOCX = ROOT / "report" / "NUSH2_Final_Report.docx"
PDF = ROOT / "report" / "NUSH2_Final_Report.pdf"
RAW = B / "tmp" / "report_raw.docx"
AUTHOR = "Wee Zhern Ray"
STALE_S = 40 * 60
WAIT_S = 30 * 60


def run(args, **kw):
    print("+", " ".join(str(a) for a in args), flush=True)
    return subprocess.run([str(a) for a in args], cwd=ROOT, **kw)


def take_lock(lock):
    lock.parent.mkdir(parents=True, exist_ok=True)
    start = time.time()
    while True:
        try:
            lock.mkdir()
            return True
        except FileExistsError:
            age = time.time() - lock.stat().st_mtime
            if age > STALE_S:
                print(f"Word lock is {age / 60:.0f} min old, treated as stale and taken over")
                shutil.rmtree(lock, ignore_errors=True)
                continue
            if time.time() - start > WAIT_S:
                print("Word lock still held after the wait; giving up")
                return False
            print("Word lock held by another job; waiting 20 s", flush=True)
            time.sleep(20)


def set_last_modified_by(path):
    """Word stamps its own user name as last modifier on save. Put the author back, metadata only."""
    tmp = path.with_suffix(".tmp.docx")
    with zipfile.ZipFile(path) as zin, zipfile.ZipFile(tmp, "w", zipfile.ZIP_DEFLATED) as zout:
        for item in zin.infolist():
            data = zin.read(item.filename)
            if item.filename == "docProps/core.xml":
                text = data.decode("utf-8")
                text = re.sub(r"<cp:lastModifiedBy>.*?</cp:lastModifiedBy>", f"<cp:lastModifiedBy>{AUTHOR}</cp:lastModifiedBy>", text)
                text = re.sub(r"<dc:creator>.*?</dc:creator>", f"<dc:creator>{AUTHOR}</dc:creator>", text)
                data = text.encode("utf-8")
            zout.writestr(item, data)
    tmp.replace(path)


def word_step():
    lock_env = os.environ.get("NUSH2_WORD_LOCK")
    if not lock_env:
        print("NUSH2_WORD_LOCK is not set; the Word step needs the shared lock and did not run")
        return 4
    lock = Path(lock_env)
    if not take_lock(lock):
        return 5
    try:
        code = run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(B / "word_run.ps1"),
                    "-InDocx", str(RAW), "-OutDocx", str(DOCX), "-OutPdf", str(PDF), "-TimeoutSec", "300"]).returncode
    finally:
        shutil.rmtree(lock, ignore_errors=True)
    if code != 0:
        print(f"Word step failed with code {code}")
        return code
    set_last_modified_by(DOCX)
    (B / "logs" / "pdf_route.txt").write_text("P1, ExportAsFixedFormat with heading bookmarks", encoding="utf-8")
    return 0


def main():
    if run([sys.executable, "scripts/build_numbers.py", "--verify"]).returncode != 0:
        if run([sys.executable, "scripts/build_numbers.py"]).returncode != 0:
            return 1
    for step in ("resolve_numbers.py", "figures.py", "build_report.py"):
        if run([sys.executable, B / step]).returncode != 0:
            return 1
    if "--no-word" in sys.argv:
        return 0
    code = word_step()
    if code != 0:
        return code
    if "--no-qa" not in sys.argv:
        return run([sys.executable, B / "qa_report.py"]).returncode
    return 0


if __name__ == "__main__":
    sys.exit(main())
