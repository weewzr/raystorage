import re, sys
sys.path.insert(0, r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/engine_final")
import tomlmini
from pathlib import Path
P = re.compile(r"inl\.md|econ_lit|htgr_ihx|amine_spec|mea_columns|mea_thermo|final_v1_7|inl_calcs|old repo|audit_recompute|freeze|screening\.py|screening|User decision|Rest row|\bQ\d+\b|qproc_kcheck|q8_check|USER_ANSWERS|_values\.json|goldens|\.py\b|\.md\b|Deck slide|deck|reader|working|\bs\d+[a-z]?\b|section [A-Z]\b|\b[A-Z]\d{1,2}\b", re.I)
d = {"param": tomlmini.load(sys.argv[1])}
n = 0
for e in d["param"]:
    txt = " | ".join(str(e.get(k, "")) for k in ("source", "locator"))
    if P.search(txt):
        n += 1
        print(f"{e['id']} [{e.get('source_status','')}] [{e.get('class','')}] dec={e.get('decision','')}\n   S: {e.get('source','')}\n   L: {e.get('locator','')}")
print(n, "of", len(d["param"]))
