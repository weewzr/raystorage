#!/usr/bin/env python3
"""Build helpers for math/NUSH2_First_Principles_Derivations.tex.

Subcommands
  register  Parse the equation tables of the private outline into math/eq_register.csv.
  values    Write math/generated/values.tex from the engine reference data and from
            the small calculations in this file. Hand-typed engine numbers are not
            allowed in the derivations, so every calculated number passes through here.
  frame     Write math/sections/00_reference_frame.tex from the register, the cards
            and math/frame_seed.csv plus math/frame_assumptions.csv.
  check     Run the final checks. Pass --sections A,B,C,D to scope the card checks while
            sections are still being written.

Python 3.10, standard library only. Run from any folder.
"""
from __future__ import annotations

import argparse
import csv
import json
import math
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MATH = ROOT / "math"
SECTIONS = MATH / "sections"
GENERATED = MATH / "generated"
REGISTER = MATH / "eq_register.csv"
SEED = MATH / "frame_seed.csv"
ASSUMPTIONS = MATH / "frame_assumptions.csv"
VALUES = GENERATED / "values.tex"
FRAME = SECTIONS / "00_reference_frame.tex"
GOLDENS = ROOT / "engine" / "vv" / "data" / "engine_goldens.json"
FREEZE = ROOT / "engine" / "vv" / "data" / "freeze_values.json"

REGISTER_FIELDS = [
    "id", "section", "tier", "title", "latex", "variables", "assumptions",
    "source", "tag", "engine_fn", "test_id", "validation",
]

SECTION_FILES = {
    "A": "A_conservation.tex", "B": "B_htgr.tex", "C": "C_ihx.tex", "D": "D_smr.tex",
    "E": "E_wgs.tex", "F": "F_cooling.tex", "G": "G_mea_thermo.tex", "H": "H_absorber.tex",
    "I": "I_stripper.tex", "J": "J_plant.tex", "K": "K_emissions.tex", "L": "L_economics.tex",
}

# Module names in the outline, mapped to the crates of the engine plan. Equation numbers
# never change; only these strings do.
ENGINE_MODULES = {
    "thermo::ideal::h_i": "props::shomate::h_i",
    "thermo::ideal::cp_i": "props::shomate::cp_i",
    "thermo::ideal::dh_rxn": "props::shomate::dh_rxn",
    "thermo::ideal::conc": "props::mixture::conc",
    "thermo::friction::dp_dz": "props::transport::dp_dz",
    "thermo::helium": "props::helium",
    "thermo::water": "props::steam",
    "thermo::eos": "props::mixture",
    "flowsheet": "plant::flowsheet",
    "nuclear": "units::htgr",
    "helium_loop": "units::circulator",
    "ihx": "units::ihx",
    "solvers": "numerics::ode",
    "validation": "vv::harness",
    "heatrec": "plant::heatrec",
    "utility": "plant::utility",
    "emissions": "plant::emissions",
    "economics": "plant::economics",
    "wgs": "units::wgs",
    "cooling": "units::cooler",
    "separation": "units::flash",
    "absorber": "units::absorber",
    "stripper": "units::stripper",
    "regen": "units::regen",
    "mea_props": "mea",
}
SMR_EQ = {"STOICH", "lump_heavies", "feed", "mole_table", "equilibrium", "performance",
          "duty_check", "sensitivity_exit"}
SMR_XF = {"k_rate", "k_ads", "den", "rates", "eta_cat"}
SMR_PROPS = {"k_eq", "quotient"}

# Where a page differs from the outline, the page wins. Each entry states the page.
PAGE_OVERRIDES = {
    ("C.16", "statement"): (
        "Tube friction: $\\Delta p=f_D(L/D)\\rho u^2/2$, $f_D=0.3164Re^{-1/4}$",
        "Sato Eq 10 PDF p4 prints 0.3164; the outline carried 0.316",
    ),
    ("D.5", "source"): (
        "[P] TEV-953 PDF p18; Case 6 [M1]",
        "The S/C of 3.0 is printed in TEV-953 PDF p18 and p64; TEV-961 sheets print none",
    ),
    ("B.1", "source"): (
        "[P] Petersen Eq 3-19, printed p6, [M5]",
        "Read from the page image of the report",
    ),
    ("B.2", "source"): (
        "[P] Petersen Eq 4-4 to 4-6, printed p6. $5R/2M=5193.2$ [C]. INL implied 5190",
        "Read from the page image of the report",
    ),
    ("B.3", "source"): ("[P] Petersen Eq 6-1, printed p6", "Read from the page image"),
    ("B.4", "source"): ("[P] Petersen Eq 7-1, printed p7", "Read from the page image"),
    ("A.7", "statement"): (
        "First law: $dE/dt=\\dot Q-\\dot W_{sh}+\\sum_{in}\\dot m(h+u^2/2+gz)-\\sum_{out}\\dot m(h+u^2/2+gz)$",
        "Shaft work is written with its own symbol because $\\dot W_s$ is the secondary circulator",
    ),
    ("A.8", "statement"): (
        "Enthalpy balance: $\\sum_{out}F_ih_i-\\sum_{in}F_ih_i=\\dot Q-\\dot W_{sh}$",
        "As A.7",
    ),
    ("A.9", "statement"): (
        "Species enthalpy: $h_i(T)=\\Delta_fh_i^\\circ+\\int_{T_{ref}}^Tc_{p,i}dT$, Shomate "
        "$c_{p,i}=A+Bt+Ct^2+Dt^3+E_S/t^2$, $t=T/1000$",
        "The Shomate E carries a subscript to keep it apart from electricity",
    ),
    ("C.4", "statement"): (
        "Network: $\\frac1{UA}=\\frac1{h_{fi}A_i}+\\frac{R_{f,i}}{A_i}+\\frac{\\ln(d_o/d_i)}{2\\pi k_wLN_t}"
        "+\\frac{R_{f,o}}{A_o}+\\frac1{h_{fo}A_o}$",
        "Film coefficients carry their own symbol because $h_i$ is species enthalpy",
    ),
    ("D.6", "statement"): (
        "Extents, per mole CH4 fed: CH4 $1-x_1$; CO $x_1-x_2$; CO2 $\\xi_{CO_2}+x_2$; "
        "H2 $\\xi_{H_2}+3x_1+x_2$; H2O $S_C-x_1-x_2$; total $\\Sigma_0+2x_1$",
        "$S_C$ keeps steam apart from surface area $S$ in A.1",
    ),
    ("D.13", "statement"): (
        "Rates: $k_i=A(k_i)\\exp(-E_{a,i}/RT)$; $A=4.225\\times10^{15}$, $1.955\\times10^{6}$, $1.020\\times10^{15}$; "
        "$E_a=240.1$, $67.13$, $243.9$ kJ/mol",
        "Activation energy carries a subscript to keep it apart from electricity",
    ),
    ("D.20", "statement"): (
        "Effectiveness: $\\eta_j^{cat}=\\int\\rho_sr_j\\,dV/(\\rho_sr_j^sV_p)$",
        "XF2 Eq 9 PDF p3",
    ),
    ("D.27", "statement"): (
        "Wall: $1/U=1/h_{fi}+R_f+t_w/k_w+(A_i/A_o)/h_{fo}$",
        "As C.4",
    ),
    ("D.28", "statement"): (
        "Helium side: $\\dot m_{He}c_p\\,dT_{He}/dz=\\pm\\pi d_iN_tU(T_{He}-T)$",
        "U in D.27 is on the inner area, so the perimeter is the inner one; the outline carried d_o",
    ),
    ("D.29", "statement"): (
        "Finned annulus: $Nu=0.039Re^{0.87}Pr^{0.4}(S_f/D_e)^{0.4}(H_f/D_e)^{-0.19}$",
        "$S_f$ keeps fin spacing apart from surface area",
    ),
    ("D.32", "statement"): (
        "Radiation: $q=\\sigma\\varepsilon_{r,a}\\varepsilon_{r,b}\\pi d_o(T_a^4-T_b^4)/A_{sh}$",
        "$\\varepsilon_b$ is bed porosity, so radiation carries its own symbol",
    ),
    ("D.15", "source"): (
        "[P] XF1 Eq 3 PDF p5",
        "DEN is printed with the Eq 3 group; Eq 5 on that page is the reverse-reaction set",
    ),
    ("D.20", "source"): (
        "[P] XF2 Eq 7 and Eq 9 PDF p3; Fig 6 PDF p5 by eye. [M16]",
        "Eq 7 is the observed rate and Eq 9 is the effectiveness definition",
    ),
}

# Equation titles where the outline statement opens with a formula.
TITLES = {
    "B.2": "Helium heat capacities", "C.10": "Number of transfer units", "D.6": "Mole table from extents",
    "D.7": "Equilibrium constant of reaction I", "D.8": "Equilibrium constant of reaction II",
    "D.9": "Equilibrium constant of reaction III", "D.15": "Adsorption denominator",
    "D.16": "Rate of reaction I", "D.17": "Rate of reaction II", "D.18": "Rate of reaction III",
    "D.22": "Methane continuity", "D.23": "Carbon dioxide continuity", "D.24": "Partial pressures",
}

# The build order withdraws the deck ledger, so L.28 carries the discounted cash flow result.
# Numbers stay fixed; only the content of the row changes.
ROW_OVERRIDES = {
    "L.28": {
        "title": "Levelised abatement cost",
        "latex": "Levelised abatement cost: $LAC=\\mathrm{NPV}(\\Delta C)/\\mathrm{NPV}(\\mathcal A)$",
        "variables": "S$/t",
        "assumptions": "Tonnes discounted at the same rate as cost",
        "source": "[C]",
        "tag": "[C]",
        "engine_fn": "plant::dcf::lac",
        "validation": "Compared with L5. A difference over 10% is explained by a named term",
    },
}
# The build order adds Method 2, the discounted cash flow of plant/dcf.rs, after the outline
# was frozen. Its cards follow the outline's last L row, so they extend the numbering.
EXTRA_ROWS = [
    {
        "id": "L.34", "section": "L", "tier": "R", "title": "Discounted cash flow",
        "latex": "Discounted cash flow: $\\mathrm{NPV}_x=\\sum_j\\sum_y c_{j,x}(y)(1+i)^{-y}$, "
                 "25 y items bought again with a straight-line credit at the horizon",
        "variables": "S$",
        "assumptions": "Real flows at 8%; whole-year horizon; 25 y items replaced",
        "source": "[P] IEAGHG 2017-02 Table 3 PDF p17 and Table 4 PDF p18; NZW09 Table ES-1 PDF p7; [C] method",
        "tag": "[C] [P]",
        "engine_fn": "plant::dcf::dcf; plant::dcf::uniform_spend; plant::dcf::with_replacement",
        "test_id": "eq_l_34",
        "validation": "ID: spend sum, replacement schedule and availability scaling",
    },
    {
        "id": "L.35", "section": "L", "tier": "R", "title": "Levelised cost of hydrogen",
        "latex": "Levelised cost of hydrogen: $LCOH_x=\\mathrm{NPV}_x/\\mathrm{NPV}(M_{H_2})$",
        "variables": "S$/kg",
        "assumptions": "Same hydrogen on both plants",
        "source": "[C]", "tag": "[C]",
        "engine_fn": "plant::dcf::dcf",
        "test_id": "eq_l_35",
        "validation": "ID",
    },
    {
        "id": "L.36", "section": "L", "tier": "R", "title": "Levelised abatement cost",
        "latex": "Levelised abatement cost: "
                 "$LAC=(\\mathrm{NPV}_{NUS}-\\mathrm{NPV}_{SMR})/(\\mathcal A\\sum_y(1+i)^{-y})$",
        "variables": "S$/t",
        "assumptions": "Tonnes discounted at the same rate as cost",
        "source": "[C]", "tag": "[C]",
        "engine_fn": "plant::dcf::dcf",
        "test_id": "eq_l_36",
        "validation": "ID: recomputed from the printed equations",
    },
    {
        "id": "L.37", "section": "L", "tier": "R", "title": "Gap test between the methods",
        "latex": "Gap test: $g=(LAC-AC_{L5})/AC_{L5}$, $|g|\\le0.10$",
        "variables": "1",
        "assumptions": "Same inputs in both methods; limit 10%",
        "source": "[C]", "tag": "[C]",
        "engine_fn": "plant::dcf::dcf",
        "test_id": "eq_l_37",
        "validation": "ID: LAC_DCF_VS_L5 in the validation results",
    },
]
VALIDATION_TRIMS = {
    "J.3": " The 66,927 in the deck is 2 t/y high",
    "K.11": " The deck's 0.585 becomes 0.578 Mt/y",
}

PRIVATE_PATTERNS = [
    (r",?\s*trap in section 7\.2", ", see Section M"),
    (r",?\s*section 8 item \d+", ", see Section M"),
    (r"`?design/[\w./-]+`?", "project calculation"),
    (r"htgr_ihx central", "the project central estimate"),
    (r"\(Q\d+\)", "(project decision)"),
    (r"\bQ\d+\b", "Project decision"),
    (r",\s*s8\.8\b", ""),
    (r"`?findings/[\w./-]+`?", "project calculation"),
    (r"`mea_calc2\.py`", "project calculation"),
    (r"screening\.py\s*(`\w+`)?", "screening model"),
    (r"`?[\w]+\.py`?", "project calculation"),
    (r"`?results/\*?[\w.*]*`?", "results"),
    (r"\(?\brefute\.json\)?", ""),
    (r"\(?\bcomplete\.json\)?", ""),
    (r",?\s*(htgr_ihx|smr_wgs|outram|mea_thermo|mea_columns|econ_lit|old_repo|critic)\s+"
     r"(s|T)\d+[\w.]*(\s*\w\d*)?", ""),
    (r",?\s*(inl|screening)(\.md)?\s+s\d+[\w.]*", ""),
    (r",?\s*(htgr_ihx|smr_wgs|outram|mea_thermo|mea_columns|econ_lit|screening)\.md(\s+s\d+[\w.]*)?", ""),
    (r",?\s*`?engine_goldens\.json`?", ""),
    (r"`", ""),
]


def clean_private(text: str) -> str:
    for pattern, repl in PRIVATE_PATTERNS:
        text = re.sub(pattern, repl, text)
    return re.sub(r"\s{2,}", " ", text).strip(" ,;")


def split_row(line: str) -> list[str]:
    body = line.strip()
    if body.startswith("|"):
        body = body[1:]
    if body.endswith("|"):
        body = body[:-1]
    return [c.strip() for c in body.split("|")]


def map_engine(cell: str) -> str:
    names = re.findall(r"`([^`]+)`", cell)
    if not names:
        return cell.strip() or "theory"
    out = []
    prefix = ""
    for name in names:
        if "::" not in name and prefix:
            name = prefix + "::" + name
        if "::" in name:
            prefix = name.rsplit("::", 1)[0]
        out.append(map_one_engine(name))
    suffix = " (disabled)" if "disabled" in cell else ""
    return "; ".join(out) + suffix


def map_one_engine(name: str) -> str:
    if name.startswith("smr::"):
        fn = name.split("::", 1)[1]
        if fn in SMR_EQ:
            return "units::smr_eq::" + fn
        if fn in SMR_XF:
            return "units::xf::" + fn
        if fn in SMR_PROPS:
            return "props::equilibrium::" + fn
        return "units::smr_tube::" + fn
    if name in ENGINE_MODULES:
        return ENGINE_MODULES[name]
    for key in sorted(ENGINE_MODULES, key=len, reverse=True):
        if name.startswith(key + "::"):
            return ENGINE_MODULES[key] + name[len(key):]
    return name


def title_of(statement: str) -> str:
    head = statement.split(":", 1)[0]
    if ":" in statement and "$" not in head and len(head) < 60:
        return head.strip()
    words = re.sub(r"\$.*?\$", "", statement).split()
    return " ".join(words[:5]).strip(" :,.") or "Equation"


def cmd_register(args: argparse.Namespace) -> int:
    plan = Path(args.plan).read_text(encoding="utf-8")
    start = plan.index("## 6. Equations")
    stop = plan.index("## 7. Source gaps")
    rows, previous, section = [], None, None
    for line in plan[start:stop].splitlines():
        heading = re.match(r"### ([A-L])\.", line)
        if heading:
            section = heading.group(1)
            previous = None
            continue
        if not line.startswith("| ") or section is None:
            continue
        cells = split_row(line)
        match = re.match(r"([A-L]\.\d+)\s+([ROEFX])$", cells[0]) if cells else None
        if not match:
            continue
        if len(cells) != 7:
            print(f"bad column count {len(cells)} in {cells[0]}", file=sys.stderr)
            return 1
        eq_id, tier = match.groups()
        for i in range(1, 7):
            if cells[i].lower() in ("as above", "") and previous is not None and i != 3:
                cells[i] = previous[i]
        previous = cells
        statement, variables, assumptions, source, engine, validation = cells[1:7]
        if (eq_id, "statement") in PAGE_OVERRIDES:
            statement = PAGE_OVERRIDES[(eq_id, "statement")][0]
        if (eq_id, "source") in PAGE_OVERRIDES:
            source = PAGE_OVERRIDES[(eq_id, "source")][0]
        number = int(eq_id.split(".")[1])
        rows.append({
            "id": eq_id,
            "section": section,
            "tier": tier,
            "title": TITLES.get(eq_id, title_of(statement)),
            "latex": clean_private(statement),
            "variables": clean_private(variables),
            "assumptions": clean_private(assumptions),
            "source": clean_private(source),
            "tag": " ".join(sorted(set(re.findall(r"\[(?:[PTCA]|M\d+)\]", source)))),
            "engine_fn": map_engine(engine),
            "test_id": f"eq_{section.lower()}_{number:02d}",
            "validation": clean_private(validation),
        })
    expected = dict(zip("ABCDEFGHIJKL", (15, 23, 19, 36, 16, 13, 33, 31, 18, 36, 22, 33)))
    for sec, want in expected.items():
        got = sum(1 for r in rows if r["section"] == sec)
        if got != want:
            print(f"section {sec}: {got} rows, outline says {want}", file=sys.stderr)
    for row in rows:
        row.update(ROW_OVERRIDES.get(row["id"], {}))
        row["assumptions"] = row["assumptions"] or "None stated"
        row["tag"] = row["tag"] or "see source"
        if row["id"] in VALIDATION_TRIMS:
            row["validation"] = row["validation"].replace(VALIDATION_TRIMS[row["id"]], "").strip()
    outline_rows = len(rows)
    rows.extend(dict(r) for r in EXTRA_ROWS)
    # A card written from the page may find no benchmark where the outline gave a note.
    # The card wins, so the register says NONE as the card does.
    none_cards = {c["id"] for c in all_cards()
                  if c["fields"].get("ValidCode", "").strip().startswith("NONE")}
    for row in rows:
        if row["id"] in none_cards and "NONE" not in row["validation"]:
            row["validation"] = "NONE: " + row["validation"]
    with REGISTER.open("w", newline="\n", encoding="utf-8") as fh:
        writer = csv.DictWriter(fh, REGISTER_FIELDS, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    print(f"register: {len(rows)} rows written to {REGISTER.relative_to(ROOT)}, "
          f"{outline_rows} from the outline and {len(EXTRA_ROWS)} from the build order")
    return 0 if outline_rows == sum(expected.values()) else 1


# ----------------------------------------------------------------------------------------
# Calculations behind the worked examples. Constants carry their page in the card that
# prints them. Nothing here feeds the engine.
# ----------------------------------------------------------------------------------------
R_GAS = 8.314462
CP_HE = 5195.0
M_HE = 4.002602e-3


# NIST Chemistry WebBook (SRD 69), gas phase Shomate tables, Chase 1998, read 2026-11-02.
# Each row: Tmin, Tmax, A, B, C, D, E, F, G, H.
SHOMATE = {
    "CH4": [(298, 1300, -0.703029, 108.4773, -42.52157, 5.862788, 0.678565, -76.84376, 158.7163, -74.87310)],
    "CO": [(298, 1300, 25.56759, 6.096130, 4.054656, -2.671301, 0.131021, -118.0089, 227.3665, -110.5271)],
    "CO2": [(298, 1200, 24.99735, 55.18696, -33.69137, 7.948387, -0.136638, -403.6075, 228.2431, -393.5224),
            (1200, 6000, 58.16639, 2.720074, -0.492289, 0.038844, -6.447293, -425.9186, 263.6125, -393.5224)],
    "H2": [(298, 1000, 33.066178, -11.363417, 11.432816, -2.772874, -0.158558, -9.980797, 172.707974, 0.0),
           (1000, 2500, 18.563083, 12.257357, -2.859786, 0.268238, 1.977990, -1.147438, 156.288133, 0.0)],
    "H2O": [(500, 1700, 30.09200, 6.832514, 6.793435, -2.534480, 0.082139, -250.8810, 223.3967, -241.8264)],
    "N2": [(500, 2000, 19.50583, 19.88705, -8.598535, 1.369784, 0.527601, -4.935202, 212.3900, 0.0)],
}
H_FORM = {"CH4": -74.8731, "CO": -110.5271, "CO2": -393.5224, "H2": 0.0, "H2O": -241.8264, "N2": 0.0}


def sh_row(sp: str, t: float) -> tuple:
    for row in SHOMATE[sp]:
        if row[0] <= t <= row[1]:
            return row
    raise ValueError((sp, t))


def sh_h(sp: str, t: float) -> float:
    """Molar enthalpy on the formation basis, kJ/mol."""
    _, _, a, b, c, d, e, f, _g, h = sh_row(sp, t)
    x = t / 1000.0
    return H_FORM[sp] + a * x + b * x ** 2 / 2 + c * x ** 3 / 3 + d * x ** 4 / 4 - e / x + f - h


def sh_s(sp: str, t: float) -> float:
    """Standard molar entropy, J/mol/K."""
    _, _, a, b, c, d, e, _f, g, _h = sh_row(sp, t)
    x = t / 1000.0
    return a * math.log(x) + b * x + c * x ** 2 / 2 + d * x ** 3 / 3 - e / (2 * x ** 2) + g


def sh_g(sp: str, t: float) -> float:
    return sh_h(sp, t) - t * sh_s(sp, t) / 1000.0


REACTIONS = {
    "one": {"CH4": -1, "H2O": -1, "CO": 1, "H2": 3},
    "two": {"CO": -1, "H2O": -1, "CO2": 1, "H2": 1},
    "three": {"CH4": -1, "H2O": -2, "CO2": 1, "H2": 4},
}


def dh_rxn(key: str, t: float) -> float:
    return sum(nu * sh_h(sp, t) for sp, nu in REACTIONS[key].items())


def k_nist(key: str, t: float) -> float:
    dg = sum(nu * sh_g(sp, t) for sp, nu in REACTIONS[key].items())
    return math.exp(-dg * 1000.0 / (R_GAS * t))


def rank_int(rows: list) -> int:
    """Rank of an integer matrix by Gaussian elimination on fractions."""
    from fractions import Fraction
    mat = [[Fraction(v) for v in r] for r in rows]
    rank, col = 0, 0
    while rank < len(mat) and col < len(mat[0]):
        pivot = next((i for i in range(rank, len(mat)) if mat[i][col] != 0), None)
        if pivot is None:
            col += 1
            continue
        mat[rank], mat[pivot] = mat[pivot], mat[rank]
        for i in range(len(mat)):
            if i != rank and mat[i][col] != 0:
                ratio = mat[i][col] / mat[rank][col]
                mat[i] = [a - ratio * b for a, b in zip(mat[i], mat[rank])]
        rank, col = rank + 1, col + 1
    return rank


def scoped_tube_duty(tube_carbon: float) -> float:
    """Enthalpy rise across the reformer on INL Case 5 anchors, scaled on tube carbon, MW.

    Outlet: raw syngas composition and mass of Case 5. Inlet: element balance on that outlet,
    with steam and water from the flowsheet labels. Heavier hydrocarbons count as CH4 carbon.
    A scoping check on the INL heat basis, nothing more.
    """
    lb = 0.45359237
    mw = {"H2": 2.016, "CO": 28.01, "CO2": 44.01, "N2": 28.013, "H2O": 18.015, "CH4": 16.043}
    y5 = {"H2": .488, "CO": .087, "CO2": .048, "N2": .007, "H2O": .334, "CH4": .036}
    mbar = sum(y5[k] * mw[k] for k in y5)
    n5 = 349724 * lb / mbar
    out5 = {k: y5[k] * n5 for k in y5}
    water = (248376 + 13513) * lb / 18.015
    h2_rec = 3077 * lb / 2.016
    co2_in = 0.0025 * 34.5e6 / 379.49 / 24 * lb + 0.012 * 1155
    co_in = 0.022 * 1155
    carbon_out = out5["CH4"] + out5["CO"] + out5["CO2"]
    ch4_in = carbon_out - co2_in - co_in
    in5 = {"CH4": ch4_in, "CO": co_in, "CO2": co2_in, "H2": h2_rec, "H2O": water, "N2": out5["N2"]}
    t_out = (1600 - 32) / 1.8 + 273.15
    t_in = (718 - 32) / 1.8 + 273.15
    f = tube_carbon / carbon_out
    q = sum(f * out5[k] * sh_h(k, t_out) for k in out5) - sum(f * in5[k] * sh_h(k, t_in) for k in in5)
    return q / 3.6e3


def k1_bar2(t: float) -> float:
    return 1.198e13 * math.exp(-26830.0 / t)


def k2(t: float) -> float:
    return 1.767e-2 * math.exp(4400.0 / t)


def smr_equilibrium(t: float, p_bar: float, s_c: float) -> dict[str, float]:
    """Two-reaction equilibrium for one mole of pure CH4. Damped Newton on (x1, x2)."""
    x1, x2 = 0.6, 0.2

    def resid(a: float, b: float) -> tuple[float, float]:
        total = 1.0 + s_c + 2.0 * a
        n = {"CH4": 1 - a, "CO": a - b, "CO2": b, "H2": 3 * a + b, "H2O": s_c - a - b}
        f1 = (3 * math.log(n["H2"]) + math.log(n["CO"]) - math.log(n["CH4"]) - math.log(n["H2O"])
              + 2 * math.log(p_bar / total) - math.log(k1_bar2(t)))
        f2 = (math.log(n["H2"]) + math.log(n["CO2"]) - math.log(n["CO"]) - math.log(n["H2O"])
              - math.log(k2(t)))
        return f1, f2

    for _ in range(200):
        f1, f2 = resid(x1, x2)
        if abs(f1) + abs(f2) < 1e-12:
            break
        h = 1e-7
        a11 = (resid(x1 + h, x2)[0] - f1) / h
        a12 = (resid(x1, x2 + h)[0] - f1) / h
        a21 = (resid(x1 + h, x2)[1] - f2) / h
        a22 = (resid(x1, x2 + h)[1] - f2) / h
        det = a11 * a22 - a12 * a21
        dx1 = (-f1 * a22 + f2 * a12) / det
        dx2 = (-a11 * f2 + a21 * f1) / det
        step = 1.0
        while not (0 < x1 + step * dx1 < 1 and 0 < x2 + step * dx2 < x1 + step * dx1
                   and x1 + step * dx1 + x2 + step * dx2 < s_c):
            step *= 0.5
        x1 += step * dx1
        x2 += step * dx2
    total = 1.0 + s_c + 2.0 * x1
    return {
        "x1": x1, "x2": x2, "total": total,
        "yCH4": (1 - x1) / total, "yCO": (x1 - x2) / total, "yCO2": x2 / total,
        "yH2": (3 * x1 + x2) / total, "yH2O": (s_c - x1 - x2) / total,
    }


def xf_den(t: float, p: dict[str, float]) -> float:
    r = 8.314e-3
    kco = 8.23e-5 * math.exp(70.65 / (r * t))
    kh2 = 6.12e-9 * math.exp(82.90 / (r * t))
    kch4 = 6.65e-4 * math.exp(38.28 / (r * t))
    kh2o = 1.77e5 * math.exp(-88.68 / (r * t))
    return (1 + kco * p["CO"] + kh2 * p["H2"] + kch4 * p["CH4"] + kh2o * p["H2O"] / p["H2"])


def xf_table_ratios() -> list[float]:
    """Table 6 evaluated at the Table 5 reference temperature, over the Table 5 value."""
    r = 8.314e-3
    rows = [
        (4.225e15, 240.1, 648.0, 1.842e-4), (1.955e6, 67.13, 648.0, 7.558),
        (1.020e15, 243.9, 648.0, 2.193e-5), (8.23e-5, -70.65, 648.0, 40.91),
        (6.12e-9, -82.90, 648.0, 0.02960), (6.65e-4, -38.28, 823.0, 0.1791),
        (1.77e5, 88.68, 823.0, 0.4152),
    ]
    return [a * math.exp(-e / (r * t)) / v for a, e, t, v in rows]


def eps_ntu(ntu: float, c_star: float) -> float:
    if abs(1.0 - c_star) < 1e-12:
        return ntu / (1.0 + ntu)
    term = math.exp(-ntu * (1.0 - c_star))
    return (1.0 - term) / (1.0 - c_star * term)


def lmtd(d1: float, d2: float) -> float:
    return d1 if abs(d1 - d2) < 1e-9 else (d1 - d2) / math.log(d1 / d2)


def ihx_rate(t_hi: float, t_ci: float, m_h: float, m_c: float, ua: float) -> tuple[float, float]:
    c_h, c_c = m_h * CP_HE, m_c * CP_HE
    c_min, c_max = min(c_h, c_c), max(c_h, c_c)
    eps = eps_ntu(ua / c_min, c_min / c_max)
    q = eps * c_min * (t_hi - t_ci)
    return t_ci + q / c_c, t_hi - q / c_h


def fmt_int(v: float) -> str:
    return f"{v:,.0f}".replace(",", "{,}")


def add(store: list, name: str, text: str, cls: str = "calculated", note: str = "") -> None:
    store.append((name, text, cls, note))


def cmd_values(args: argparse.Namespace) -> int:
    gold = json.loads(Path(args.goldens).read_text(encoding="utf-8"))
    freeze = json.loads(Path(args.freeze).read_text(encoding="utf-8"))
    case_b, case_a = gold["cases"]["B|REC_F"], gold["cases"]["A|REC_F"]
    fb, fa = freeze["0.85"]["B"], freeze["0.85"]["A"]
    bb, ba = case_b["balance"], case_a["balance"]
    eb, ea = case_b["emissions"], case_a["emissions"]
    carbon = case_b["carbon"]
    st: list = []

    def pair(name: str, vb: float, va: float, spec: str, note: str = "") -> None:
        add(st, name + "B", format(vb, spec), "calculated", note)
        add(st, name + "A", format(va, spec), "calculated", note)

    def pair_int(name: str, vb: float, va: float, note: str = "") -> None:
        add(st, name + "B", fmt_int(vb), "calculated", note)
        add(st, name + "A", fmt_int(va), "calculated", note)

    s_b = bb["s"]
    pair("vScale", s_b, 1.0, ".4f")
    pair("vMdotHtwo", fb["h2_kg_h"] / 3600, fa["h2_kg_h"] / 3600, ".3f")
    pair_int("vMHtwo", fb["h2_t_y"], fa["h2_t_y"])
    add(st, "vHours", fmt_int(fb["hours"]))
    pair("vNG", fb["ng_mmscfd"], fa["ng_mmscfd"], ".1f")
    pair("vQR", bb["R"], ba["R"], ".1f")
    pair("vQRx", bb["R"], ba["R"], ".2f")
    pair("vSpare", 210.0 - bb["R"], 210.0 - ba["R"], ".1f")
    pair("vMs", 78.49 * s_b, 78.49, ".1f", "78.49 kg/s scaled by s")
    add(st, "vMsA", "78.49", "published")
    st[:] = [x for i, x in enumerate(st) if not (x[0] == "vMsA" and x[2] == "calculated")]
    pair("vMsc", fb["he_sec_kg_s"], fa["he_sec_kg_s"], ".1f", "carries the slip heat too")
    pair("vMp", fb["he_prim_kg_s_at_R"], fa["he_prim_kg_s_at_R"], ".1f")
    add(st, "vMpAr", format(fa["he_prim_kg_s_at_R"], ".0f"))
    pair("vQproc", bb["q_proc"], ba["q_proc"], ".1f")
    pair("vQprocx", bb["q_proc"], ba["q_proc"], ".2f")
    pair("vQrec", bb["q_recyc"], ba["q_recyc"], ".2f")
    pair("vWs", bb["w_s"], ba["w_s"], ".2f")
    pair("vWp", bb["w_p"], ba["w_p"], ".2f")
    pair("vWsum", bb["w_s"] + bb["w_p"], ba["w_s"] + ba["w_p"], ".2f")
    pair("vQves", bb["qv"], ba["qv"], ".2f")
    pair("vQihx", bb["q_ihx"], ba["q_ihx"], ".2f")
    pair("vQduct", bb["q_duct"], ba["q_duct"], ".2f")
    pair("vQbr", bb["branch"], ba["branch"], ".2f")
    pair("vQreb", bb["q_reb"], ba["q_reb"], ".1f")
    pair("vQstack", bb["avail"], ba["avail"], ".1f")
    pair("vQgap", bb["gap"], ba["gap"], ".1f")
    pair("vQgapx", bb["gap"], ba["gap"], ".2f")
    pair("vEproc", bb["e_proc"], ba["e_proc"], ".2f")
    pair("vEcirc", bb["e_circ"], ba["e_circ"], ".2f")
    pair("vEtot", bb["e_total"], ba["e_total"], ".2f")
    add(st, "vEtaPl", format(carbon["cap"] / carbon["F"], ".4f"))
    add(st, "vPhiC", format(carbon["co2_abs"] / carbon["T"], ".3f"))
    add(st, "vFr", format(carbon["R"] / carbon["G"], ".3f"))
    pair("vMcotwo", bb["cap_t"], ba["cap_t"], ".1f")
    pair_int("vMsolv", fb["solvent_t_h"], fa["solvent_t_h"])
    add(st, "vEbaseInt", format(gold["constants"]["BASE_INL"], ".2f"))
    pair_int("vEbase", eb["baseline"], ea["baseline"])
    pair_int("vEatm", eb["atm"], ea["atm"])
    pair_int("vAbate", eb["A_atm"], ea["A_atm"])
    ladder = case_b["ladder_SM_y"]
    for life, tag in ((25, "TwentyFive"), (60, "Sixty")):
        cost = ladder[f"BOAK|n1.00|y{life}|R210.0"][3] * 1e6 / eb["A_atm"]
        add(st, "vAC" + tag, format(cost, ".1f"))

    # Section A worked example, carbon closure, kmol/h at Case B.
    add(st, "cFeedC", format(carbon["F"] * s_b, ".2f"))
    add(st, "cCapC", format(carbon["cap"] * s_b, ".2f"))
    add(st, "cBurnC", format(carbon["burn"] * s_b, ".2f"))
    add(st, "cProdC", format(carbon["P"] * s_b, ".2f"))
    add(st, "cTubeC", format(carbon["T"] * s_b, ".2f"))
    add(st, "cRecyC", format(carbon["R"] * s_b, ".2f"))

    # Section B.
    t_rot, t_sup, t_ret = 1198.15, 1173.15, 739.15
    m_p, q_r, w_p = fb["he_prim_kg_s_at_R"], bb["R"] * 1e6, bb["w_p"] * 1e6
    add(st, "cCpIdeal", format(2.5 * R_GAS / M_HE, ".1f"))
    p_bar, t_k = 70.0, t_rot
    z_he = 1.0 + 0.4446 * p_bar / t_k ** 1.2
    add(st, "cZHe", format(z_he, ".4f"))
    add(st, "cRhoHe", format(48.14 * p_bar / t_k / z_he, ".3f"))
    add(st, "cMuHe", format(3.674e-7 * 1173.15 ** 0.7 * 1e5, ".2f"))
    k_he = 2.682e-3 * (1 + 1.123e-3 * 51.5) * 1173.15 ** (0.71 * (1 - 2e-4 * 51.5))
    add(st, "cKHe", format(k_he, ".3f"))
    add(st, "cPrHe", format(CP_HE * 3.674e-7 * 1173.15 ** 0.7 / k_he, ".2f"))
    rise = w_p / (m_p * CP_HE)
    t_ri_c = t_ret + 25.0 - 273.15 + rise
    add(st, "cRise", format(rise, ".1f"))
    add(st, "cTRI", format(t_ri_c, ".1f"))
    fit = 0.8205 * 925.0 - 242.68
    add(st, "cRITfit", format(fit, ".1f"))
    pr_pet = 0.7117 / (1 + 1.123e-3 * 51.5) * 1173.15 ** (-(0.01 - 1.42e-4 * 51.5))
    add(st, "cPrPet", format(pr_pet, ".3f"))
    add(st, "cRITgap", format(fit - t_ri_c, ".1f"))
    q_core = m_p * CP_HE * (t_rot - 273.15 - t_ri_c) / 1e6
    add(st, "cCoreLHS", format((q_r - bb["qv"] * 1e6) / 1e6, ".2f"))
    add(st, "cCoreRHS", format(q_core, ".2f"))
    add(st, "cCoreGap", format(100 * abs(q_core - (q_r / 1e6 - bb["qv"])) / q_core, ".1f"))
    m_p_exact = (q_r - bb["qv"] * 1e6 + w_p) / (CP_HE * 434.0)
    add(st, "cMpExact", format(m_p_exact, ".1f"))
    add(st, "cQav", format(bb["R"] + bb["w_s"] + bb["w_p"] - bb["qv"] - bb["q_ihx"] - bb["q_duct"], ".1f"))
    add(st, "cExtractW", format(min(bb["e_total"], 0.169 * bb["gap"]), ".2f"))
    add(st, "cDuctRoot", format(0.45 * math.sqrt(78.49 * s_b / 78.49), ".2f"))
    add(st, "cDuctLong", format(1.82 * math.sqrt(78.49 / 26.67), ".2f"))
    add(st, "cDuctShort", format(0.26 * math.sqrt(78.49 / 26.67), ".2f"))
    add(st, "cExcess", format(78.49 * 5190 * 5 / 1e6, ".2f"))
    add(st, "cInlCheck", format(78.49 * 5190 * 434 / 1e6, ".1f"))
    add(st, "cSatoRise", format(2.519 * CP_HE * 17 / 1e6, ".3f"))
    add(st, "cCarnotRatio", format(1 - 0.475 / 0.66, ".3f"))
    add(st, "cVesFrac", format(100 * bb["qv"] / bb["R"], ".1f"))
    add(st, "cShaftSpec", format(CP_HE * t_ret * ((1 + 200 / 7000) ** 0.4 - 1) / 0.75 / 1e3, ".1f"))
    add(st, "cShaftSpecP", format(CP_HE * (t_ret + 25) * ((1 + 150 / 7000) ** 0.4 - 1) / 0.75 / 1e3, ".1f"))

    # Section C.
    q_b = bb["q_proc"]
    add(st, "cUAb", format(q_b / 25.0, ".2f"))
    add(st, "cAreab", fmt_int(q_b * 1e6 / 25.0 / 400.0))
    add(st, "cAreabHigh", fmt_int(q_b * 1e6 / 25.0 / 760.0))
    add(st, "cNTUb", format(434.0 / 25.0, ".1f"))
    add(st, "cEpsb", format(434.0 / 459.0, ".3f"))
    add(st, "cUAratio", format(q_b * 1e3 / 25.0 / 85.0, ".0f"))
    add(st, "cUAmodule", format(q_b / 25.0 / 4.0, ".2f"))
    add(st, "cUAmoduleRatio", format(q_b * 1e3 / 25.0 / 4.0 / 85.0, ".0f"))
    add(st, "cUAfifty", format(q_b / 50.0, ".2f"))
    add(st, "cHttrLmtd", format(lmtd(135.0, 77.0), ".1f"))
    q_httr = 3.6 * CP_HE * (751 - 248) / 1e6
    add(st, "cHttrQ", format(q_httr, ".2f"))
    ua_httr = q_httr * 1e3 / lmtd(135.0, 77.0)
    add(st, "cHttrUA", format(ua_httr, ".1f"))
    add(st, "cHttrEps", format(503.0 / 580.0, ".3f"))
    add(st, "cHttrNTU", format(ua_httr * 1e3 / (3.6 * CP_HE), ".1f"))
    tc_out, th_out = ihx_rate(828.0, 248.0, 4.1, 3.6, 91e3)
    add(st, "cHttrTc", format(tc_out, ".0f"))
    add(st, "cHttrTh", format(th_out, ".0f"))
    tc15, th15 = ihx_rate(468.0, 158.0, 4.1, 3.6, 80e3)
    add(st, "cHttrTcLow", format(tc15, ".0f"))
    add(st, "cHttrThLow", format(th15, ".0f"))
    add(st, "cHttrU", format(ua_httr * 1e3 / 212.9, ".0f"))
    m_600 = 176.8e6 / (CP_HE * 325.0)
    add(st, "cImbalMp", format(m_600, ".1f"))
    add(st, "cImbalUA", format(176.8 / lmtd(134.0, 25.0), ".2f"))
    add(st, "cImbalLm", format(lmtd(134.0, 25.0), ".1f"))
    add(st, "cImbalMpB", format(q_b * 1e6 / (CP_HE * 325.0), ".1f"))
    add(st, "cImbalUAB", format(q_b / lmtd(134.0, 25.0), ".2f"))
    add(st, "cImbalRatioB", format((q_b / lmtd(134.0, 25.0)) * 1e3 / 85.0, ".0f"))

    # Section D.
    t_ref = 1144.15
    eq = smr_equilibrium(t_ref, 30.3, 3.0)
    add(st, "cXeq", format(100 * eq["x1"], ".1f"))
    add(st, "cXtwoEq", format(eq["x2"], ".3f"))
    add(st, "cYield", format(3 * eq["x1"] + eq["x2"], ".2f"))
    add(st, "cSigma", format(eq["total"], ".3f"))
    for name, key in (("cyCH", "yCH4"), ("cyCO", "yCO"), ("cyCOtwo", "yCO2"), ("cyH", "yH2"), ("cyHO", "yH2O")):
        add(st, name, format(100 * eq[key], ".1f"))
    for tag, tc in (("Eighteen", 825.0), ("Eight", 800.0)):
        add(st, "cX" + tag, format(100 * smr_equilibrium(tc + 273.15, 30.3, 3.0)["x1"], ".1f"))
    for tag, tc, p in (("TeA", 800.0, 41.9), ("TeB", 860.0, 41.9), ("TeC", 800.0, 20.0)):
        add(st, "cX" + tag, format(100 * smr_equilibrium(tc + 273.15, p, 3.98)["x1"], ".1f"))
    add(st, "cKone", format(k1_bar2(t_ref), ".1f"))
    add(st, "cKtwo", format(k2(t_ref), ".4f"))
    add(st, "cSlopeOne", format(26830 * R_GAS / 1e3, ".1f"))
    add(st, "cSlopeTwo", format(4400 * R_GAS / 1e3, ".1f"))
    ratios = xf_table_ratios()
    add(st, "cRatioMin", format(min(ratios), ".3f"))
    add(st, "cRatioMax", format(max(ratios), ".3f"))
    den = xf_den(1000.0, {"CO": 0.9, "H2": 7.5, "CH4": 4.5, "H2O": 13.5})
    add(st, "cDenSq", format(den ** 2, ".1f"))
    add(st, "cTubesB", fmt_int(q_b * 1e6 / (math.pi * 0.14 * 14.0 * 50e3)))
    add(st, "cHaFlux", format(72.9e6 / (199 * math.pi * 0.14 * 14.0) / 1e3, ".1f"))
    add(st, "cCaseFive", format(568.4 * 0.29307, ".1f"))
    add(st, "cCaseFiveRef", format(404.2 * 0.29307, ".1f"))

    # Additions for sections A to D.
    add(st, "cQrated", "210", "assumed")
    for key, tag in (("one", "One"), ("two", "Two"), ("three", "Three")):
        add(st, "cDhRoom" + tag, format(sum(nu * H_FORM[sp] for sp, nu in REACTIONS[key].items()), ".2f"))
        add(st, "cDhNine" + tag, format(dh_rxn(key, 948.0), ".1f"))
    gaps = [abs(dh_rxn(k, 948.0) - v) for k, v in (("one", 224.0), ("two", -37.3), ("three", 187.5))]
    add(st, "cDhGapMax", format(max(gaps), ".1f"))
    grid = (680.0, 800.0, 950.0, 1073.15, 1098.15, 1144.15)
    dev1 = [abs(k1_bar2(t) / k_nist("one", t) - 1.0) * 100 for t in grid]
    dev2 = [abs(k2(t) / k_nist("two", t) - 1.0) * 100 for t in grid]
    add(st, "cKoneDevMin", format(min(dev1), ".1f"))
    add(st, "cKoneDevMax", format(max(dev1), ".1f"))
    add(st, "cKtwoDevMax", format(max(dev2), ".1f"))
    add(st, "cKoneDevExit", format(dev1[-1], ".1f"))
    add(st, "cKtwoDevExit", format(dev2[-1], ".1f"))
    add(st, "cDutySel", format(scoped_tube_duty(2157.2), ".1f"), "calculated", "Case A, INL selexol basis")
    add(st, "cDutyRec", format(scoped_tube_duty(carbon["T"]), ".1f"), "calculated", "Case A, recycle basis")
    add(st, "cCaseFivePre", format((568.4 - 404.2) * 0.29307, ".1f"))
    q_h30, q_c30 = 4.1 * CP_HE * 445 / 1e6, 3.6 * CP_HE * 503 / 1e6
    q_h15, q_c15 = 4.1 * CP_HE * 233 / 1e6, 3.6 * CP_HE * 262 / 1e6
    add(st, "cHttrQh", format(q_h30, ".2f"))
    add(st, "cHttrQlow", format(q_c15, ".2f"))
    add(st, "cHttrQhLow", format(q_h15, ".2f"))
    add(st, "cIhxResHigh", format(100 * (q_h30 - q_c30) / q_c30, ".2f"))
    add(st, "cIhxResLow", format(100 * (q_h15 - q_c15) / q_c15, ".2f"))
    add(st, "cVesHigh", format(0.027 * bb["R"], ".2f"))
    add(st, "cWsHigh", format(fb["he_sec_kg_s"] * CP_HE * t_ret * ((1 + 414 / 7000) ** 0.4 - 1) / 0.75 / 1e6, ".2f"))
    add(st, "cHttrULow", format(q_c15 * 1e6 / lmtd(77.0, 48.0) / 212.9, ".0f"))
    add(st, "cHttrUAlow", format(q_c15 * 1e3 / lmtd(77.0, 48.0), ".1f"))
    add(st, "cHttrLmtdLow", format(lmtd(77.0, 48.0), ".1f"))
    c_atoms = 93.571 + 2 * 3.749 + 3 * 0.920 + 4 * 0.260 + 5 * 0.040 + 6 * 0.010 + 0.250
    add(st, "cCperMol", format(c_atoms / 100.0, ".5f"))
    reformer = [(1, 4, 0), (0, 2, 1), (1, 0, 1), (1, 0, 2), (0, 2, 0)]
    add(st, "cRankRef", str(rank_int([list(r) for r in reformer])))
    mea = [(2, 7, 1, 1, 0), (2, 8, 1, 1, 1), (3, 6, 3, 1, -1), (1, 0, 2, 0, 0), (1, 1, 3, 0, -1),
           (1, 0, 3, 0, -2), (0, 2, 1, 0, 0), (0, 3, 1, 0, 1), (0, 1, 1, 0, -1)]
    add(st, "cRankMea", str(rank_int([list(r) for r in mea])))
    add(st, "cTubesBLow", fmt_int(q_b * 1e6 / (math.pi * 0.14 * 14.0 * 60e3)))
    add(st, "cTubesBHigh", fmt_int(q_b * 1e6 / (math.pi * 0.14 * 14.0 * 40e3)))

    GENERATED.mkdir(parents=True, exist_ok=True)
    lines = [
        "% Generated by scripts/derivations_tools.py values. Never edit by hand.",
        "% Pass 1: screening reference data plus the small calculations in the tools script.",
    ]
    seen = set()
    for name, text, cls, note in st:
        if name in seen:
            print(f"duplicate macro {name}", file=sys.stderr)
            return 1
        seen.add(name)
        if not re.fullmatch(r"[A-Za-z]+", name):
            print(f"bad macro name {name}", file=sys.stderr)
            return 1
        suffix = f"  {note}" if note else ""
        lines.append(f"\\newcommand{{\\{name}}}{{{text}}} % class={cls}{suffix}")
    VALUES.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    print(f"values: {len(st)} macros written to {VALUES.relative_to(ROOT)}")
    return 0


# ----------------------------------------------------------------------------------------
# Card parsing.
# ----------------------------------------------------------------------------------------
def brace_arg(text: str, start: int) -> tuple[str, int]:
    """Return the group opening at text[start] and the index after it."""
    if text[start] != "{":
        raise ValueError("expected a brace")
    depth, i = 0, start
    while i < len(text):
        c = text[i]
        if c == "\\":
            i += 2
            continue
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return text[start + 1:i], i + 1
        i += 1
    raise ValueError("unbalanced braces")


def strip_comments(tex: str) -> str:
    return re.sub(r"(?<!\\)%.*", "", tex)


def parse_cards(tex: str) -> list[dict]:
    cards = []
    pattern = re.compile(r"\\begin\{eqcard\}\{([A-L]\.\d+)\}\{([ROEFX])\}")
    for match in pattern.finditer(tex):
        end = tex.index("\\end{eqcard}", match.end())
        body = tex[match.end():end]
        title, _ = brace_arg(tex, match.end())
        fields: dict[str, str] = {}
        for name in ("Stmt", "Interp", "Deriv", "Vars", "Assume", "Source", "Impl", "Valid"):
            pos = re.search(r"\\" + name + r"(?=\{)", body)
            if pos:
                fields[name], after = brace_arg(body, pos.end())
                if name == "Valid":
                    code = fields[name]
                    test, after2 = brace_arg(body, after)
                    text, _ = brace_arg(body, after2)
                    fields["ValidCode"], fields["ValidTest"], fields["Valid"] = code, test, text
        symbols = []
        for pos in re.finditer(r"\\V(?=\{)", fields.get("Vars", "")):
            sym, a = brace_arg(fields["Vars"], pos.end())
            definition, b = brace_arg(fields["Vars"], a)
            units, _ = brace_arg(fields["Vars"], b)
            symbols.append((normal_symbol(sym), definition, units, sym.strip()))
        cards.append({"id": match.group(1), "tier": match.group(2), "title": title,
                      "fields": fields, "symbols": symbols})
    return cards


def normal_symbol(sym: str) -> str:
    return re.sub(r"\s+", "", sym)


def all_cards() -> list[dict]:
    cards: list[dict] = []
    for name in SECTION_FILES.values():
        path = SECTIONS / name
        if path.exists():
            cards.extend(parse_cards(strip_comments(path.read_text(encoding="utf-8"))))
    return cards


# ----------------------------------------------------------------------------------------
# Reference Frame.
# ----------------------------------------------------------------------------------------
def tex_text(value: str) -> str:
    """Escape text cells outside inline math, leaving LaTeX math untouched."""
    value = re.sub(r"\b(US|S)\$", r"\1\\$", value)
    parts = re.split(r"((?<!\\)\$(?:[^$]|\\\$)*?(?<!\\)\$)", value)
    out = []
    for i, part in enumerate(parts):
        if i % 2:
            out.append(part)
            continue
        part = part.replace("&", "\\&").replace("%", "\\%").replace("#", "\\#")
        part = re.sub(r"(?<!\\)_", r"\\_", part)
        part = part.replace("`", "")
        part = part.replace("~", "about ")
        out.append(part)
    return "".join(out)


UNIT_GROUP = re.compile(r"\\U\{((?:[^{}]|\{[^{}]*\})*)\}")


def breakable_units(units: str) -> str:
    """Split \\U{a\\ b} into \\U{a} \\U{b}, so a narrow table cell can wrap between factors."""
    def split(m: re.Match) -> str:
        parts = [p for p in m.group(1).split("\\ ") if p.strip()]
        return " ".join("\\U{" + p + "}" for p in parts) if parts else m.group(0)
    return UNIT_GROUP.sub(split, units)


def breakable_symbol(sym: str) -> str:
    """Allow a line break after each comma at brace depth 0 of a symbol list."""
    out, depth = [], 0
    for c in sym:
        depth += (c == "{") - (c == "}")
        out.append(c)
        if c == "," and depth == 0:
            out.append("\\allowbreak ")
    return "".join(out)


def cmd_frame(_: argparse.Namespace) -> int:
    rows = list(csv.DictReader(REGISTER.open(encoding="utf-8")))
    seed = list(csv.DictReader(SEED.open(encoding="utf-8")))
    assumptions = list(csv.DictReader(ASSUMPTIONS.open(encoding="utf-8")))
    known = {normal_symbol(r["symbol"]) for r in seed}
    extras: dict[str, tuple[str, str, str]] = {}
    for card in all_cards():
        for sym, definition, units, raw in card["symbols"]:
            if sym not in known and sym not in extras:
                extras[sym] = (definition, units, raw)
    out = [
        "% Generated by scripts/derivations_tools.py frame. Never edit by hand.",
        "\\section{Reference Frame}\\label{sec:frame}",
        "\\input{sections/00_frame_intro}",
        "\\subsection{Table of variables}",
        "\\begingroup\\small",
        "\\begin{longtable}{@{}>{\\raggedright\\arraybackslash}p{2.5cm}"
        ">{\\raggedright\\arraybackslash}p{4.5cm}"
        ">{\\raggedright\\arraybackslash}p{2.2cm}"
        ">{\\raggedright\\arraybackslash}p{2.0cm}"
        ">{\\raggedright\\arraybackslash}p{1.8cm}"
        ">{\\raggedright\\arraybackslash}p{1.8cm}@{}}",
        "\\toprule Symbol & Definition & SI units & Case B & Case A & Class\\\\\\midrule\\endfirsthead",
        "\\toprule Symbol & Definition & SI units & Case B & Case A & Class\\\\\\midrule\\endhead",
        "\\bottomrule\\endlastfoot",
    ]
    for r in seed:
        out.append(f"${breakable_symbol(r['symbol'])}$ & {tex_text(r['definition'])} & "
                   f"{breakable_units(r['units'])} & "
                   f"{r['case_b']} & {r['case_a']} & {r['class']}\\\\")
    for definition, units, raw in extras.values():
        out.append(f"${breakable_symbol(raw)}$ & {definition} & {breakable_units(units)} & "
                   f"varies & varies & variable\\\\")
    out += ["\\end{longtable}", "\\endgroup", "\\subsection{Numbered equation list}"]
    out.append("Every equation in the document, grouped by section, in compact form. "
               "The full card for each one follows in its section.")
    current = None
    for r in rows:
        if r["section"] != current:
            if current is not None:
                out.append("\\end{enumerate}")
            current = r["section"]
            out.append(f"\\paragraph{{Section {current}}}")
            out.append("\\begin{enumerate}[leftmargin=1.4cm,itemsep=0pt,topsep=2pt,parsep=0pt]\\small")
        out.append(f"\\item[{r['id']}] {tex_text(r['latex'])} {{\\footnotesize [{r['tier']}]}}")
    if current is not None:
        out.append("\\end{enumerate}")
    out += [
        "\\subsection{Assumptions and parameter ranges}",
        "\\begingroup\\small",
        "\\begin{longtable}{@{}>{\\raggedright\\arraybackslash}p{3.1cm}"
        ">{\\raggedright\\arraybackslash}p{3.1cm}"
        ">{\\raggedright\\arraybackslash}p{2.8cm}"
        ">{\\raggedright\\arraybackslash}p{6.6cm}@{}}",
        "\\toprule Parameter & Central & Range & Basis and status\\\\\\midrule\\endfirsthead",
        "\\toprule Parameter & Central & Range & Basis and status\\\\\\midrule\\endhead",
        "\\bottomrule\\endlastfoot",
    ]
    for r in assumptions:
        out.append(f"{tex_text(r['parameter'])} & {tex_text(r['central'])} & "
                   f"{tex_text(r['range'])} & {tex_text(r['basis'])}\\\\")
    out += ["\\end{longtable}", "\\endgroup"]
    FRAME.write_text("\n".join(out) + "\n", encoding="utf-8", newline="\n")
    print(f"frame: {len(seed)} seed rows, {len(extras)} card symbols, {len(rows)} equations")
    return 0


# ----------------------------------------------------------------------------------------
# Final checks.
# ----------------------------------------------------------------------------------------
BANNED = [r"Group\s*7", r"\btime\s*slot\b", r"\bdeadline\b", r"\bspeaker\b", r"AI[- ]declaration",
          r"AI[- ]use", r"\bA\d{7}[A-Z]\b"]


def public_files() -> list[Path]:
    files = sorted(MATH.rglob("*.tex")) + sorted(MATH.glob("*.csv")) + sorted(MATH.glob("*.bib"))
    return [p for p in files if p.is_file()]


def report(name: str, problems: list[str], limit: int = 12) -> bool:
    state = "PASS" if not problems else "FAIL"
    print(f"{state}  {name}  ({len(problems)} problems)")
    for p in problems[:limit]:
        print(f"      {p}")
    if len(problems) > limit:
        print(f"      and {len(problems) - limit} more")
    return not problems


def cmd_check(args: argparse.Namespace) -> int:
    scope = set(args.sections.split(",")) if args.sections else set("ABCDEFGHIJKL")
    register = {r["id"]: r for r in csv.DictReader(REGISTER.open(encoding="utf-8"))}
    cards = all_cards()
    by_id = {c["id"]: c for c in cards}
    ok = True

    want = {i for i, r in register.items() if r["section"] in scope}
    have = {c["id"] for c in cards if c["id"][0] in scope}
    ok &= report("Register", [f"no card for {i}" for i in sorted(want - have)]
                 + [f"card not in register {i}" for i in sorted(have - want)]
                 + [f"duplicate card {i}" for i in {c["id"] for c in cards
                                                    if sum(d["id"] == c["id"] for d in cards) > 1}])

    problems = []
    for c in cards:
        if c["id"][0] not in scope:
            continue
        for field in ("Stmt", "Interp", "Deriv", "Vars", "Assume", "Source", "Impl", "ValidCode", "Valid"):
            if not c["fields"].get(field, "").strip():
                problems.append(f"{c['id']} missing {field}")
        reg = register.get(c["id"])
        if reg and c["tier"] != reg["tier"]:
            problems.append(f"{c['id']} tier {c['tier']} against register {reg['tier']}")
        if reg and c["fields"].get("ValidTest") != reg["test_id"]:
            problems.append(f"{c['id']} test id differs from register")
    for c in cards:
        reg = register.get(c["id"])
        if c["id"][0] not in scope or reg is None:
            continue
        impl = c["fields"].get("Impl", "").replace("\\_", "_")
        for name in reg["engine_fn"].replace(" (disabled)", "").split("; "):
            if name != "theory" and name not in impl:
                problems.append(f"{c['id']} implementation does not name {name}")
    ok &= report("Card fields", problems)

    seed_symbols = {normal_symbol(r["symbol"]) for r in csv.DictReader(SEED.open(encoding="utf-8"))}
    frame_text = FRAME.read_text(encoding="utf-8") if FRAME.exists() else ""
    frame_norm = normal_symbol(frame_text.replace("\\allowbreak", ""))
    problems = [f"{c['id']}: {s}" for c in cards if c["id"][0] in scope
                for s, _, _, _ in c["symbols"] if s not in seed_symbols and s not in frame_norm]
    ok &= report("Frame", problems)

    problems = []
    macro_lines = VALUES.read_text(encoding="utf-8").splitlines() if VALUES.exists() else []
    literals = {}
    for line in macro_lines:
        m = re.match(r"\\newcommand\{\\(\w+)\}\{(.*?)\} % class=(\w+)", line)
        if m and m.group(3) == "calculated":
            value = m.group(2).replace("{,}", ",")
            if len(re.sub(r"\D", "", value)) >= 4:
                literals[value] = m.group(1)
    hand_written = [p for p in SECTIONS.glob("*.tex") if p.name != "00_reference_frame.tex"]
    for path in hand_written:
        text = strip_comments(path.read_text(encoding="utf-8")).replace("{,}", ",")
        for value, name in literals.items():
            if re.search(r"(?<![\d.,])" + re.escape(value) + r"(?![\d])", text):
                problems.append(f"{path.name}: {value} typed by hand, use \\{name}")
    seed_text = SEED.read_text(encoding="utf-8").replace("{,}", ",")
    ok &= report("Macros", problems)

    problems = []
    for c in cards:
        if c["id"][0] not in scope:
            continue
        src = c["fields"].get("Source", "")
        if ("[P]" in src or "\\tagP" in src) and not re.search(r"p\.|Table|Eq|Eqs|Fig|sheet", src):
            problems.append(f"{c['id']} has [P] with no page or table")
        for m_id in re.findall(r"\\tagM\{(\d+)\}", src):
            if not 1 <= int(m_id) <= 31:
                problems.append(f"{c['id']} cites M{m_id}, which is not in the register")
    ok &= report("Sources", problems)

    problems = []
    for path in public_files():
        text = path.read_text(encoding="utf-8")
        for pattern in BANNED:
            for m in re.finditer(pattern, text, flags=re.I):
                problems.append(f"{path.relative_to(ROOT)}: {m.group(0)}")
    ok &= report("Banned text", problems)

    problems = []
    for path in public_files():
        text = path.read_text(encoding="utf-8")
        if "\u2014" in text:
            problems.append(f"{path.relative_to(ROOT)} holds an em dash")
        if path.suffix == ".tex" and "---" in strip_comments(text):
            problems.append(f"{path.relative_to(ROOT)} holds ---")
    ok &= report("Style", problems)

    problems = []
    for path in public_files():
        for m in re.finditer(r"\b[A-Za-z]:[\\/][\w. ]", path.read_text(encoding="utf-8")):
            problems.append(f"{path.relative_to(ROOT)}: {m.group(0)}")
    for path in public_files():
        for m in re.finditer(r"\b(htgr_ihx|smr_wgs|refute\.json|engine_goldens|\w+\.md\b)", path.read_text(encoding="utf-8")):
            problems.append(f"{path.relative_to(ROOT)}: private name {m.group(0)}")
    ok &= report("Paths", problems)

    problems = []
    for path in hand_written:
        text = strip_comments(path.read_text(encoding="utf-8"))
        for m in re.finditer(r"\bvalidated\b", text, flags=re.I):
            problems.append(f"{path.name}: validated at {m.start()}")
        for m in re.finditer(r"(?<!not )\bverified\b", text, flags=re.I):
            problems.append(f"{path.name}: verified at {m.start()}")
    ok &= report("Verification wording", problems)

    if args.pass2:
        problems = []
        engine_text = "\n".join(p.read_text(encoding="utf-8") for p in (ROOT / "engine").rglob("*.rs"))
        for card_id, reg in register.items():
            if reg["section"] not in scope or reg["engine_fn"] == "theory":
                continue
            if f"eq: {card_id}" not in engine_text:
                problems.append(f"{card_id} has no doc tag in engine source")
        ok &= report("Engine links, pass 2", problems)
        problems = [f"{r['test_id']}" for r in register.values()
                    if r["section"] in scope and "NONE" not in r["validation"]
                    and f"fn {r['test_id']}" not in engine_text]
        ok &= report("Test links, pass 2", problems)

    if args.log:
        log = Path(args.log).read_text(encoding="utf-8", errors="replace")
        problems = re.findall(r"^!.*$", log, flags=re.M)
        problems += re.findall(r"LaTeX Warning: (?:Reference|Citation) .* undefined.*", log)
        ok &= report("Build log", problems)
    if args.aux:
        aux = Path(args.aux).read_text(encoding="utf-8", errors="replace")
        problems = [f"no label for {c['id']}" for c in cards
                    if c["id"][0] in scope and f"\\newlabel{{eq:{c['id']}}}" not in aux]
        ok &= report("Labels", problems)

    print("Card count in scope:", len(have), "of", len(want))
    tiers: dict[str, int] = {}
    for c in cards:
        if c["id"][0] in scope:
            tiers[c["tier"]] = tiers.get(c["tier"], 0) + 1
    print("Tiers:", dict(sorted(tiers.items())))
    print("NONE rows:", sum(1 for c in cards if c["id"][0] in scope
                            and c["fields"].get("ValidCode", "").strip().startswith("NONE")))
    return 0 if ok else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    reg = sub.add_parser("register")
    reg.add_argument("--plan", required=True, help="the private outline, Markdown")
    reg.set_defaults(func=cmd_register)
    val = sub.add_parser("values")
    val.add_argument("--goldens", default=str(GOLDENS))
    val.add_argument("--freeze", default=str(FREEZE))
    val.set_defaults(func=cmd_values)
    fr = sub.add_parser("frame")
    fr.set_defaults(func=cmd_frame)
    chk = sub.add_parser("check")
    chk.add_argument("--sections", default="")
    chk.add_argument("--pass2", action="store_true")
    chk.add_argument("--log", default="")
    chk.add_argument("--aux", default="")
    chk.set_defaults(func=cmd_check)
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
