import re, sys
PAT = ["plan section","screening script","screening ledger","smr_wgs","critic.md","audit.md","inventory.md",
"decision s8","item m14","inl.md","econ_lit","htgr_ihx.md","htgr_calc","amine_spec","mea_columns","mea_thermo",
"mea_calc","final_v1_7","inl_calcs","old repo","old_repo","old-repo","audit_recompute","freeze section","freeze s",
"screening.py","user decision","user_answers","rest row","deck slide","q8_check","qproc_kcheck","build order section",
"freeze","screening","the plan","build plan","stage ","workflow","agent","lead ","claude","subagent","questionnaire",
"scratch","findings/","design/","wf_results","review","the brief","rubric","guideline","lecturer","course","module","team",
"group","teammate","our ","we ","small modular","smr"]
PUBLIC_MD = {"docs/final_design_decisions.md","docs/validation_matrix.md","readme.md","references/references.md"}
def wk(w):
    if not w: return False
    f, rest = w[0], w[1:]
    d = re.match(r"\d*", rest).group(0); tail = rest[len(d):]
    return f in "sQ" and d and (tail == "" or (len(tail)==1 and tail.islower()))
for fn in sys.argv[1:]:
    for i, line in enumerate(open(fn, encoding="utf-8"), 1):
        low = line.lower()
        hs = [p for p in PAT if p in low]
        for w in re.split(r"[^A-Za-z0-9_./-]", low):
            w = w.rstrip(".")
            if w.endswith(".md") and w not in PUBLIC_MD: hs.append("file "+w)
            if w.endswith(".py") and not w.startswith("scripts/"): hs.append("file "+w)
        for w in re.split(r"[^A-Za-z0-9]", line):
            if wk(w): hs.append("key "+w)
        if "\u2014" in line: hs.append("EMDASH")
        if hs: print(f"{fn}:{i}: {sorted(set(hs))}")
