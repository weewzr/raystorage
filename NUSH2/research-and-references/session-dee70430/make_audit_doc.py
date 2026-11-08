"""Write docs/ABATEMENT_SUBSIDY_AUDIT.md from the saved Phase 1 audit results."""
import json
import re
import sys

SRC = sys.argv[1]
OUT = sys.argv[2]

res = json.load(open(SRC, encoding="utf-8"))
ORDER = [
    ("ae87df3d5d2f9884e", "Physical carbon balance and CO2 accounting"),
    ("ad5d97fc0494d4ae5", "Provenance of external values (independent critic)"),
    ("a57f4a86fcb0a6422", "Method 1: the annual incremental ledger L0 to L5"),
    ("a19ef4e33e53323aa", "Method 2: the discounted cash flow"),
    ("a0830c2f164e90dc8", "Parameters and the zero-subsidy baseline"),
    ("a29222ceeb33a4a71", "Exporter, tests and documentation chain"),
    ("a07b8083510654136", "Browser simulator and wasm"),
]

PATH = re.compile(r"(?<![A-Za-z])[A-Za-z]:[/\\][^\s,;)\"']*")


def clean(text):
    text = text.replace("—", ", ").replace("–", "-")
    text = PATH.sub("<scratch>", text)
    text = re.sub(r"check_site WITHDRAWN bans .*? in site files,",
                  "check_site keeps a list of withdrawn figures that site files must not print,", text)
    text = text.replace("|", "/").replace("\r", " ").replace("\n", " ")
    return re.sub(r"\s+", " ", text).strip()


lines = []
w = lines.append
w("# Phase 1 audit: CO2 abatement, economics and simulator")
w("")
w("Scope: the NUS-H2 repository at `main` commit 8750b09. Seven read-only audits covered the carbon balance, the provenance of external values, the L0 to L5 ledger, the discounted cash flow, the parameters, the exporter and test chain, and the browser simulator. Each finding carries the file and line that its auditor opened. The auditors did not build or run the engine; the baseline run is recorded in `docs/ABATEMENT_SUBSIDY_LOG.md`.")
w("")
w("Kind labels: `independent_physical_input`, `process_assumption`, `validation_reference`, `copied_result` and `calculated`. Risk is the risk to the redevelopment if the item is left as it is.")
w("")
w("## Headline finding")
w("")
w("The headline CO2 abatement of Case B (578,052 t/y) is the INL TEV-953 unabated reference (3,205 st/d scaled by the hydrogen ratio, 616,361 t/y) minus the carbon burned in an algebraic ledger that is anchored to INL Case 5 and Case 6 numbers (34 MMSCFD of gas and 1,927 st/d captured). The recycle fraction of 0.775 also comes from Case 5. So an external plant result sets 93.8 percent of the abatement. Air Liquide appears once, as the 100,000 Nm3/h hydrogen capacity (`H2_B`), and it sets only the plant scale; it feeds no emission factor.")
w("")
w("A stream-based flowsheet (Level 3) with a carbon ledger that closes to 5e-9 kmol C/h already exists, but it only validates the anchored ledger. It disagrees with it by 19.7 percent on the stack carbon and fails its own gate. Carbon closure in both ledgers holds by construction, so closure is a bookkeeping check and does not test capture performance.")
w("")
w("The existing co-funding is a reactor-capital multiplier that also removes interest during construction on the granted share. It has no grant timing and no eligible set, offers no share grid, and aims at S$96/t instead of S$100/t.")
w("")

for key, title in ORDER:
    r = res[key]
    w(f"## {title}")
    w("")
    w(clean(r["summary"]))
    w("")
    w("| Id | Where | Finding | Kind | Risk |")
    w("|---|---|---|---|---|")
    for it in r["items"]:
        w("| {} | {} | {} | {} | {} |".format(
            clean(it["id"]), clean(it["where"]), clean(it["finding"]),
            it.get("kind", "-"), it.get("risk", "-")))
    w("")
    if r["hooks"]:
        w("Where new work attaches:")
        w("")
        for h in r["hooks"]:
            note = f" {clean(h['note'])}" if h.get("note") else ""
            w(f"- {clean(h['purpose'])}: `{clean(h['where'])}`.{note}")
        w("")
    if r["gaps"]:
        w("Not verified by this audit:")
        w("")
        for g in r["gaps"]:
            w(f"- {clean(g)}")
        w("")

text = "\n".join(lines) + "\n"
bad = [c for c in ("—", "–") if c in text]
assert not bad, bad
open(OUT, "w", encoding="utf-8", newline="\n").write(text)
print("written", OUT, len(text), "chars")
