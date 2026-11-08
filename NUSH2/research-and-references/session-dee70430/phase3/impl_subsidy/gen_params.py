"""Write the subsidy block of engine/data/parameters.toml (scratch generator).

Run once: python gen_params.py <path to parameters.toml>. It inserts the block before
the SMR_ISLAND_TCR_EURM entry and refuses to run twice.
"""
import sys
from pathlib import Path

LOG = "docs/ABATEMENT_SUBSIDY_LOG.md"
DER = "docs/derivation/02_cost_and_subsidy.md"
REFS = "docs/ABATEMENT_SUBSIDY_REFERENCES.md"

entries = []


def add(pid, value, unit, cls, source, locator, status, name, low=None, high=None):
    entries.append((pid, value, unit, cls, source, locator, status, name, low, high))


grid = [0.0, 0.25, 0.5, 0.75]
for k, v in enumerate(grid, start=1):
    extra = (
        "; the 75 percent case is a stress case, because no plant-capital precedent was found"
        if k == 4
        else ""
    )
    add(
        f"SUBSIDY_SHARE_GRID_{k}",
        v,
        "-",
        "user_decision",
        f"Grant share grid of project decision D7 in {LOG}: 0, 25, 50 and 75 percent of eligible expenditure{extra}",
        "decision D7",
        "primary",
        f"Subsidy share, grid point {k}",
    )

items = [
    ("REACTOR", "reactor_capital", "the reactor overnight capital"),
    ("IHX", "ihx_capital", "the intermediate heat exchanger"),
    ("LICENCE", "licence_capital", "the licence"),
    ("POWER_ISLAND", "power_island_capital", "the power island"),
    ("CAPTURE", "capture_capital", "the capture plant"),
    ("RECYCLE", "recycle_capital", "the recycle train"),
]
flags_a = {"REACTOR": 1.0}
flags_b = {"REACTOR": 1.0, "IHX": 1.0, "LICENCE": 1.0, "CAPTURE": 1.0, "RECYCLE": 1.0}

for code, field, words in items:
    on = flags_a.get(code, 0.0)
    state = "eligible" if on else "outside the grant"
    add(
        f"SUBSIDY_ELIG_A_{code}",
        on,
        "flag",
        "user_decision",
        f"Definition A of project decision D7b in {LOG}: the reactor overnight capital alone. "
        f"Flag of cost item {field} ({words}); 1 means eligible, 0 means outside the grant. "
        f"In definition A this item is {state}",
        "decision D7b",
        "primary",
        f"Definition A eligibility, {words.replace('the ', '')}",
    )

for code, field, words in items:
    on = flags_b.get(code, 0.0)
    why = (
        "P4 in " + REFS + " lists power generation assets as ineligible, and decision D7b leaves the power island out"
        if code == "POWER_ISLAND"
        else "decision D7b lists this item in definition B"
    )
    add(
        f"SUBSIDY_ELIG_B_{code}",
        on,
        "flag",
        "user_decision",
        f"Definition B of project decision D7b in {LOG}: reactor, IHX, capture plant, recycle train and licence. "
        f"The list follows the construction cost categories of the EU Innovation Fund relevant cost methodology "
        f"(P3 in {REFS}) and the UK Net Zero Hydrogen Fund capital strand (P4 in the same file); that each item of "
        f"this plant falls inside those categories is a modelling judgement. Flag of cost item {field} ({words}); "
        f"1 means eligible, 0 means outside the grant. {why[0].upper() + why[1:]}",
        "decision D7b",
        "primary",
        f"Definition B eligibility, {words.replace('the ', '')}",
    )

add(
    "SUBSIDY_DEFAULT_DEFINITION",
    0.0,
    "flag",
    "user_decision",
    f"Default eligible set of the co-funding scenarios, project decision D7 in {LOG}: 0 selects definition A "
    "(reactor only), 1 selects definition B. The exported table prints both definitions",
    "decision D7",
    "primary",
    "Subsidy default eligibility definition",
)
add(
    "SUBSIDY_DEFAULT_TIMING",
    0.0,
    "flag",
    "user_decision",
    f"Default grant timing of the co-funding scenarios, project decision D7 in {LOG}: 0 selects a progressive grant "
    "paid pro rata with the eligible spend, 1 selects a grant paid at financial close. The exported table prints both timings",
    "decision D7",
    "primary",
    "Subsidy default grant timing",
)
add(
    "SUBSIDY_SHARE_MAX",
    1.0,
    "-",
    "calculated",
    f"A share is a fraction of eligible expenditure and cannot exceed 1 (equation S.2 of {DER})",
    "equation S.2",
    "primary",
    "Subsidy share upper limit",
)
add(
    "SUBSIDY_SOLVER_TOL",
    1e-12,
    "-",
    "assumed",
    f"Modelling assumption, no published value: bracketing tolerance of the bisection fallback for the break-even "
    f"share (equation S.7d of {DER})",
    "equation S.7d",
    "not_verified",
    "Subsidy break-even bisection tolerance",
)
add(
    "SUBSIDY_RESIDUAL_TOL",
    1e-9,
    "S$/t",
    "assumed",
    f"Modelling assumption, no published value: largest distance between the cost at a closed-form break-even share "
    f"and the target before the bisection fallback takes over (equations S.7b to S.7d of {DER})",
    "equations S.7b to S.7d",
    "not_verified",
    "Subsidy break-even residual tolerance",
)
add(
    "SUBSIDY_GRANT_CASH_RATE",
    0.0,
    "1/y",
    "assumed",
    f"Modelling assumption, no published value: grant cash that has arrived and is not yet spent earns nothing "
    f"(convention 3 in {DER}). Only 0 is implemented; equation S.3d gives the bound at the discount rate",
    "convention 3, equation S.3d",
    "not_verified",
    "Subsidy return on idle grant cash",
)
add(
    "SUBSIDY_IDC_BASIS",
    2.0,
    "-",
    "assumed",
    f"Modelling assumption, no published value: interest during construction is charged on the private balance "
    f"outstanding, basis 2 of equation S.5a in {DER}. Basis 1 is the gross spend and basis 3 is the part the grant "
    f"saves, which is reported and never an input. Only basis 2 is implemented",
    "equation S.5a",
    "not_verified",
    "Subsidy interest during construction basis",
)
add(
    "SUBSIDY_GAP_LIMIT",
    0.01,
    "-",
    "user_decision",
    f"Largest accepted gap between the ledger and the discounted cash flow at a subsidised point, as a share of "
    f"the ledger cost: 1 percent (project decision D11 in {LOG}). The comparison with the gap predicted by the "
    f"derivation is a separate check",
    "decision D11",
    "primary",
    "Subsidy ledger and cash flow gap limit",
)


def fmt(v):
    if v == 0:
        return "0.0"
    if abs(v) < 1e-3:
        return f"{v:.0e}".replace("e-0", "e-")
    return repr(float(v))


out = []
for pid, value, unit, cls, source, locator, status, name, low, high in entries:
    out.append("[[param]]")
    out.append(f'id = "{pid}"')
    out.append(f"value = {fmt(value)}")
    out.append(f'unit = "{unit}"')
    out.append('group = "cost"')
    out.append(f'class = "{cls}"')
    out.append('rigour = "fixed_assumption"')
    out.append(f'source = "{source}"')
    out.append(f'locator = "{locator}"')
    out.append(f'source_status = "{status}"')
    out.append(f'name = "{name}"')
    out.append("")
block = "\n".join(out) + "\n"

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
anchor = '[[param]]\nid = "SMR_ISLAND_TCR_EURM"\n'
if "SUBSIDY_SHARE_GRID_1" in text:
    sys.exit("block already present")
if text.count(anchor) != 1:
    sys.exit("anchor not unique")
path.write_text(text.replace(anchor, block + anchor), encoding="utf-8", newline="\n")
print(f"inserted {len(entries)} entries")
