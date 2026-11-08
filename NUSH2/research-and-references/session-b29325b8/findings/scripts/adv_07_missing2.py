import sys, re, pickle, pymupdf
sys.stdout.reconfigure(encoding="utf-8")
SP = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
idx = pickle.load(open(SP + "/findings/adv_index.pkl", "rb"))
p = "C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/webfetch-1793580492930-ryxsf0.pdf"
d = pymupdf.open(p)
print("=== RPT p26-27 size ===")
for n in (26, 27):
    t = re.sub(r"\s+", " ", d[n - 1].get_text())
    i = t.find("Grouping Based on Reactor Size")
    print(n, t[max(i, 0):max(i, 0) + 1500] if i >= 0 else t[:1100])
EX = re.compile(r"^Implementation-of-Nuclear|^Claude/|^HTGR_SMR_MEA_CCS_Research_GitHub|\.css$|\.js$")
Q = {
 "circ_power": r"(circulator|blower|compressor).{0,70}(\d[\d,.]*\s*(kW|MW)\b)|(\d[\d,.]*\s*(kW|MW)\b).{0,50}(circulator|blower)|(circulator|blower).{0,40}(power|heat|work)",
 "ihx_dp": r"(IHX|intermediate heat exchanger).{0,120}(pressure (drop|loss)|\bkPa\b)|(pressure (drop|loss)).{0,120}(IHX|intermediate heat exchanger)",
 "he_props": r"(helium).{0,60}(specific heat|heat capacity|viscosity|thermal conductivity)|(specific heat|heat capacity|viscosity|thermal conductivity).{0,60}(of|for) helium",
 "water_vp": r"Antoine|IAPWS|vapou?r pressure of water|water vapou?r pressure|saturation pressure|steam tables?\b",
 "effectiveness": r"effectiveness factor",
 "creep": r"creep",
 "cond": r"process condensate|condensate (stripper|treatment|recycle)",
 "HTR-PM": r"HTR-?PM.{0,120}(750|°C|C\b)",
 "hoa_table": r"Table \d[\d.]*.{0,80}(heat of absorption|enthalpy of absorption|heat capacity|volatility|speciation)",
 "srd_syngas": r"(GJ/t|GJ/tCO2|MJ/kg ?CO2|GJ ?/ ?ton).{0,80}(syngas|shifted|pre-?combustion|hydrogen)|(syngas|shifted|pre-?combustion).{0,80}(GJ/t|MJ/kg ?CO2)",
 "reboiler_duty_ieag": r"reboiler (duty|heat|energy)|specific reboiler|regeneration (energy|heat|duty)",
}
for name, pt in Q.items():
    pat = re.compile(pt, re.I)
    res = {}
    for (rel, pg), t in idx.items():
        if EX.search(rel):
            continue
        if name in ("reboiler_duty_ieag",) and not re.search(r"2017-02|2022-07|S2772656826|Low-Carbon", rel):
            continue
        tt = re.sub(r"\s+", " ", t)
        ms = list(pat.finditer(tt))
        if ms:
            res.setdefault(rel, []).append((pg, len(ms), tt[max(0, ms[0].start() - 100):ms[0].end() + 130]))
    print(f"\n#### {name}: {len(res)} files")
    for rel, v in sorted(res.items(), key=lambda kv: -sum(x[1] for x in kv[1]))[:9]:
        n = sum(x[1] for x in v)
        print(f" {rel[-58:]} hits={n} pages={[x[0] for x in v][:12]}")
        for x in v[:3]:
            print(f"    p{x[0]}: {x[2]}")
