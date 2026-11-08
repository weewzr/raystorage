"""Remove the local path and the clear-text names from two report builder files."""
import re
import sys
from pathlib import Path

B = Path(sys.argv[1])

p = B / "seed_content.py"
s = p.read_text(encoding="utf-8")
m = re.search(r'SP = Path\("[^"]*"\n\s*"[^"]*"\)\nSRC = SP / "txt" / "report_latest.txt"\n', s)
assert m, "seed path block"
s = s.replace(m.group(0), '# The extracted text of the base report, passed in by environment so no local path is written here.\n'
                          'SRC = Path(os.environ["NUSH2_SEED_SRC"])\n')
s = s.replace("import re\nfrom pathlib import Path\n", "import os\nimport re\nfrom pathlib import Path\n", 1)
m = re.search(r'\("04_economics", r"\^Economics \\\([a-z]+\\\)"\)', s)
assert m, "economics break"
s = s.replace(m.group(0), '("04_economics", r"^Economics\\b")')
p.write_text(s, encoding="utf-8")

p = B / "qa_report.py"
s = p.read_text(encoding="utf-8")
forb = re.search(r'"66,927", "([A-Z][a-z]+)", "Gungsuh"', s)
assert forb
s = s.replace(forb.group(0), '"66,927", "Gungsuh"')
ad = re.search(r'author_date = re\.findall\(r"[^"]+", alltext\)', s)
assert ad
s = s.replace(ad.group(0), 'author_date = re.findall(r"\\([A-Z][A-Za-z-]+(?: et al\\.| and [A-Z][A-Za-z-]+)?,? (?:19|20)\\d\\d[a-z]?\\)", alltext)')
p.write_text(s, encoding="utf-8")
print("sanitized")
