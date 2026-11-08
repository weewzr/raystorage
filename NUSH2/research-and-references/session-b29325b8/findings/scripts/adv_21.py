import pymupdf, re, sys
sys.stdout.reconfigure(encoding="utf-8")
SP = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
d = pymupdf.open(SP + "/findings/dl_econ/eria/ch15_2023.pdf")
txt = " ".join(p.get_text() for p in d)
print("amine hits in Sheng:", len(re.findall(r"amine", txt, re.I)), " syngas:", len(re.findall(r"syngas|synthesis gas", txt, re.I)), " SMR/reform:", len(re.findall(r"reform", txt, re.I)))
