import sys
from pathlib import Path

target = Path(sys.argv[1])
new = Path(sys.argv[2]).read_text(encoding="utf-8")
s = target.read_text(encoding="utf-8")
old_main = '''def main():
    tok = load()
    cost_grid(tok)
    waterfall(tok, 25)
    waterfall(tok, 60)
    stepdown(tok)
    return 0
'''
assert s.count(old_main) == 1
s = s.replace(old_main, new + '''def main():
    tok = load()
    cost_grid(tok)
    waterfall(tok, 25)
    waterfall(tok, 60)
    stepdown(tok)
    epz_schematic()
    roadmap()
    pfd_report()
    return 0
''')
old_imp = "import json\nimport os\nimport sys\n"
assert s.count(old_imp) == 1
s = s.replace(old_imp, "import json\nimport math\nimport os\nimport sys\n")
target.write_text(s, encoding="utf-8")
print("spliced")
