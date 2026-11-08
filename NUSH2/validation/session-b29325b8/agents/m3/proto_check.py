"""Run the prototype's 104-point check, for a side by side with the Rust port."""
import importlib.util, io, contextlib, math
S = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/findings/scripts/mea_thermo_final.py"
spec = importlib.util.spec_from_file_location("fin", S)
fin = importlib.util.module_from_spec(spec); spec.loader.exec_module(fin)
fin.code_verif()
print("k2 298.15", 10**(10.99-2152/298.15), "versteeg", 4.4e11*math.exp(-5400/298.15))
h = 10**(5.3-1140/298.15)
print("Eq 8 at C=0, 298.15:", h, "Aronu Eq17 in atm L/mol", 2.82e6*math.exp(-2044/298.15)/101.325)
