#!/usr/bin/env python3
"""Build helpers for math/NUSH2_First_Principles_Derivations.tex.

Subcommands
  register  Parse the equation tables of the private outline into math/eq_register.csv.
  links     Write into the register the engine item each card links to: the item that
            carries its "eq: X.n" doc line, else the present code its card names.
  values    Pass 2: write math/generated/values.tex from the final results files, the
            build tokens, the parameter registry and the small calculations in this file,
            with the validation status of each linked card. Hand-typed engine numbers are
            not allowed in the derivations, so every calculated number passes through here.
            --test-log takes the output of cargo test for the test run sentence.
  frame     Write math/sections/00_reference_frame.tex from the register, the cards
            and math/frame_seed.csv plus math/frame_assumptions.csv.
  check     Run the final checks. Pass --sections A,B,C,D to scope the card checks while
            sections are still being written, and --pass2 --test-log for the engine and
            test links.

Order for a pass 2 build: links, values, frame, then Tectonic, then check with --log and --aux.

Python 3.10, standard library only. Run from any folder.
"""
from __future__ import annotations

import argparse
import csv
import math
import re
import statistics
import subprocess
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
    apply_links(rows, engine_index())
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


# ----------------------------------------------------------------------------------------
# Pass 2 inputs: the final results files, the build tokens made from them, the parameter
# registry and one benchmark fixture. Nothing here is typed from a printout of the engine.
# ----------------------------------------------------------------------------------------
RESULTS = ROOT / "results"
TOKENS = ROOT / "build" / "numbers_tokens.csv"
PARAMS = ROOT / "engine" / "data" / "parameters.toml"
DOCS = ROOT / "docs" / "FINAL_DESIGN_DECISIONS.md"
VINJ_FIXTURE = ROOT / "engine" / "vv" / "data" / "benchmarks" / "vinjarapu_2024_parametric.csv"

# Engine constants, engine/props/src/units.rs.
LB, ST, MMBTU = 0.45359237, 0.90718474, 1.05505585
MW_CO2, MW_H2, MW_CH4 = 44.0095, 2.01588, 16.043

# Macro classes. results: read from a results file or a build token. derived: arithmetic on
# results values. calculated: arithmetic on published inputs only. docs: read from the
# design decisions table. screening: a screening table value with no results row.
CHECKED_CLASSES = {"results", "derived", "calculated", "docs", "screening"}
CLASS_WORDS = {
    "results": "read from the results files", "derived": "computed from results values",
    "calculated": "computed here from published inputs", "docs": "read from the design decisions table",
    "screening": "taken from the screening table", "published": "published", "assumed": "assumed",
}


class Inputs:
    """The final results of the engine, as the repository holds them."""

    def __init__(self) -> None:
        def rows(name: str) -> list[dict]:
            return list(csv.DictReader((RESULTS / name).open(encoding="utf-8")))

        self.res = {r["id"]: r for r in rows("Final_Design_Basis.csv")}
        self.energy = {(r["run_id"], r["term_id"]): float(r["value_mw"] or "nan")
                       for r in rows("Integrated_Energy_Balance.csv")}
        self.val_rows = rows("Validation_Results.csv")
        self.val = {r["test_id"]: r for r in self.val_rows}
        self.sens = rows("Sensitivity_Results.csv")
        self.emis = rows("Emissions_Ledger.csv")
        self.ledger = rows("Economic_Ledger.csv")
        self.tok = {r["token"]: r for r in csv.DictReader(TOKENS.open(encoding="utf-8"))}
        self.par: dict[str, float] = {}
        for block in PARAMS.read_text(encoding="utf-8").split("[[param]]"):
            fields = dict(re.findall(r'^(\w+) = "?(.*?)"?\s*$', block, flags=re.M))
            if "id" in fields:
                try:
                    self.par[fields["id"]] = float(fields["value"].split("#")[0])
                except ValueError:
                    pass
        self.docs = DOCS.read_text(encoding="utf-8")

    def r(self, key: str, col: str = "case_b") -> float:
        return float(self.res[key][col])

    def e(self, run: str, term: str) -> float:
        return self.energy[(run, term)]

    def t(self, name: str) -> str:
        """Display text of a build token, with TeX thousands groups."""
        return self.tok[name]["text"].replace(",", "{,}")

    def tt(self, name: str) -> str:
        return self.tok[name]["table"].replace(",", "{,}")

    def sweep(self, sweep: str, metric: str, test: float | None = None, run: str = "B_REC_F") -> float:
        for row in self.sens:
            if row["sweep_id"] == sweep and row["metric"] == metric and row["run_id"] == run:
                if test is None or abs(float(row["test_value"]) - test) < 1e-6:
                    return float(row["result"])
        raise KeyError((sweep, metric, test, run))

    def line(self, stage: str, life: int, line_id: str, field: str = "value_sgdm_y") -> float:
        for row in self.ledger:
            if (row["run_id"], row["stage"], row["life_y"], row["n"], row["reactor_charged_mwth"],
                    row["line_id"]) == ("B_REC_F", stage, str(life), "1.0000", "210.00", line_id):
                return float(row[field] or 0)
        raise KeyError((stage, life, line_id))

    def basis(self, col: str, run: str) -> dict[str, float]:
        """The plant basis one results column holds, in the keys the worked examples use."""
        r = lambda key: self.r(key, col)  # noqa: E731
        e = lambda term: self.e(run, term)  # noqa: E731
        eta_m = r("ETA_MOTOR")
        return {
            "R": r("RES_REACTOR_NEEDED_MWTH"),
            "w_s": r("RES_E_CIRC_SEC_MWE") * eta_m, "w_p": r("RES_E_CIRC_PRIM_MWE") * eta_m,
            "qv": e("VESSEL_LOSS"), "q_ihx": e("IHX_LOSS"), "q_duct": e("DUCT_LOSS"),
            "branch": e("POWER_BRANCH"), "q_reb": r("RES_REBOILER_MW"), "avail": e("STEAM_AVAILABLE"),
            "gap": r("RES_REBOILER_GAP_MW"), "e_proc": r("RES_E_PROCESS_MWE"),
            "e_circ": e("HE_CIRCULATION"), "e_total": r("RES_E_TOTAL_MWE"),
            "cap_t": r("RES_CAPTURED_KMOL_H") * MW_CO2 / 1000, "q_proc": r("RES_PROCESS_HEAT_MW"),
            "q_recyc": r("RES_SLIP_HEAT_MW"), "h2_kg_h": r("RES_H2_KG_H"), "h2_t_y": r("RES_H2_T_Y"),
            "hours": r("RES_HOURS"), "ng_mmscfd": r("RES_NG_MMSCFD"),
            "he_sec_kg_s": r("RES_HE_SEC_KG_S"), "he_prim_kg_s_at_R": r("RES_HE_PRIM_KG_S"),
            "solvent_t_h": r("RES_SOLVENT_T_H"), "baseline": r("RES_BASELINE_T_Y"),
            "atm": r("RES_ATM_T_Y"), "A_atm": r("RES_ABATEMENT_T_Y"),
        }


def f1(v: float) -> str:
    return f"{v:,.1f}".replace(",", "{,}")


def vinjarapu_window() -> list[float]:
    """SRD of the Table D.19 steady states with capture of 88 to 92%, from the fixture."""
    by_state: dict[str, dict[str, float]] = {}
    for row in csv.DictReader(VINJ_FIXTURE.open(encoding="utf-8")):
        if row["locator"].startswith("Table D.19") and row["quantity"] in ("SRD", "capture"):
            by_state.setdefault(row["series"], {})[row["quantity"]] = float(row["value"])
    return [v["SRD"] for v in by_state.values() if "SRD" in v and 88.0 <= v.get("capture", 0.0) <= 92.0]


def values_i_to_l(d: Inputs, st: list) -> None:
    """Numbers of sections I to L. Each line states its class; see CLASS_WORDS."""
    P = d.par
    R = d.r
    s = R("RES_H2_KG_H") / R("RES_H2_KG_H", "case_a_mea")
    hours, days = R("RES_HOURS"), R("RES_DAYS")
    cap_kmol = R("RES_CAPTURED_KMOL_H")
    m_cap_th = cap_kmol * MW_CO2 / 1000
    q_reb, solv = R("RES_REBOILER_MW"), R("RES_SOLVENT_T_H")
    abate, h2_t_y = R("RES_ABATEMENT_T_Y"), R("RES_H2_T_Y")

    # Section I.
    add(st, "cMeaKmol", fmt_int(cap_kmol / 0.25), "derived")
    add(st, "cKtwoRatio", format(10 ** (2152 / 313.15 - 2152 / 393.15), ".0f"))
    dice_r = 3183 / 3519
    add(st, "cDiceRatio", format(dice_r, ".3f"))
    add(st, "cDicePstar", format(27.35 / (1 + dice_r) * 0.0689476, ".2f"))
    t_lean, t_rich = (253.8 - 32) / 1.8, (138.1 - 32) / 1.8
    add(st, "cDiceLeanC", format(t_lean, ".1f"))
    add(st, "cDiceLeanLoad", format(2345 / 14654, ".3f"))
    m_rich = 3242115 * LB / 3600
    dt = 274 * MMBTU / 3.6 * 1e3 / (m_rich * 3.6)
    appr = t_lean - (t_rich + dt)
    add(st, "cDiceApproach", format(appr, ".1f"))
    add(st, "cDiceBtu", format(242e6 / 263.3e3, ".0f"))
    lp_btu = 121.7e6 / 134540
    add(st, "cInlLpBtu", format(lp_btu, ".0f"))
    add(st, "cSteamKgS", format(q_reb * 3.6 / MMBTU * 1e6 / lp_btu * LB / 3600, ".1f"), "derived")
    tph = 1857 * ST / 24
    add(st, "cDiceTph", format(tph, ".1f"))
    srd_d, qstrip = 242 * MMBTU / tph, 64.4 * MMBTU / tph
    qsens = (3242115 * LB / 1000) * 3.6 * appr / 1000 / tph
    add(st, "cDiceSrd", format(srd_d, ".2f"))
    add(st, "cDiceQstrip", format(qstrip, ".2f"))
    add(st, "cDiceQsens", format(qsens, ".2f"))
    add(st, "cDiceQdes", format(srd_d - qstrip - qsens, ".2f"))
    rich_b = solv + m_cap_th
    qsens_b = rich_b * 3.6 * 10 / 1000 / m_cap_th
    add(st, "cQsensB", format(qsens_b, ".2f"), "derived")
    add(st, "cSrdBottomB", format(srd_d - qsens + qsens_b, ".2f"), "derived")
    add(st, "cStrInt", format(q_reb * 1e3 * 3600 / (rich_b * 1e3), ".0f"), "derived")
    p_mmhg = 60 * 0.0689476 / P["ANTOINE_MMHG_TO_BAR"]
    add(st, "cDiceTsat", format(P["ANTOINE_B"] / (P["ANTOINE_A"] - math.log10(p_mmhg)) - P["ANTOINE_C"], ".0f"))
    q_ih = 53 * 264 / 13.1 / 1000
    add(st, "cIhTerm", format(q_ih, ".2f"))
    add(st, "cIhTotal", format(2.71 + q_ih, ".2f"))
    win = vinjarapu_window()
    add(st, "cVinjN", str(len(win)))
    add(st, "cVinjMedian", format(statistics.median(win), ".2f"))
    add(st, "cVinjMin", format(min(win), ".2f"))
    add(st, "cVinjMax", format(max(win), ".2f"))
    # The modelled SRD of term T06 is left out on purpose: gate M6 failed, and decision S12
    # keeps that value to the drift table and the validation rows of the results.

    # Section J.
    add(st, "cChMass", format(100 * 0.001 * MW_CH4 / (0.999 * MW_H2 + 0.001 * MW_CH4), ".2f"))
    kmol_a = 130e6 / 379.49 / 24 * LB
    add(st, "cVolScale", format(100000 / (kmol_a * 22.414), ".4f"))
    add(st, "vHkgB", fmt_int(R("RES_H2_KG_H")), "results")
    add(st, "vNGxB", format(R("RES_NG_MMSCFD"), ".2f"), "results")
    add(st, "vCapA", f1(R("RES_CAPTURED_KMOL_H", "case_a_inl")), "results")
    add(st, "vTubeInlA", f1(R("RES_TUBE_C_KMOL_H", "case_a_inl")), "results")
    add(st, "vTgInlA", f1(R("RES_TAILGAS_C_KMOL_H", "case_a_inl")), "results")
    add(st, "vTgB", f1(R("RES_TAILGAS_C_KMOL_H")), "results")
    add(st, "vPsaFeed", fmt_int(R("RES_PSA_H2_FEED_KG_H")), "results")
    add(st, "vPsaRej", fmt_int(R("RES_PSA_H2_REJECT_KG_H")), "results")
    slip = R("RES_SLIP_KMOL_H", "case_a_mea") - R("RES_SLIP_KMOL_H", "case_a_inl")
    add(st, "cDfSlip", f1(slip), "derived")
    add(st, "vStackSel", format(d.e("B_REC_F", "REB_FROM_SELEXOL"), ".2f"), "results")
    add(st, "vStackExp", format(d.e("B_REC_F", "REB_FROM_EXPORT"), ".2f"), "results")
    add(st, "vStackSyn", format(d.e("B_REC_F", "SYNGAS_RECOVERY"), ".2f"), "results",
        "term T05, modelled on the Headline")
    add(st, "vStackSynAnch", format(R("TERM_T05"), ".2f"), "results", "term T05, anchored value")
    add(st, "vErowRef", format(d.e("B_REC_F", "REFORMING_POWER"), ".2f"), "results")
    add(st, "vErowPsa", format(d.e("B_REC_F", "PURIFICATION_POWER"), ".3f"), "results")
    add(st, "vErowSteam", format(d.e("B_REC_F", "STEAM_SYSTEM_POWER"), ".2f"), "results")
    add(st, "vErowCool", format(P["E_COOLING"] * s, ".3f"), "derived")
    add(st, "vErowWater", format(d.e("B_REC_F", "WATER_POWER"), ".2f"), "results")
    add(st, "vEpump", format(R("TERM_T07"), ".2f"), "results")
    add(st, "vEfans", format(R("TERM_T08"), ".2f"), "results")
    add(st, "vEcomp", format(R("TERM_T09"), ".2f"), "results")
    add(st, "vEpurge", format(P["E_PURGE"] * s * (1 + slip / P["PSA_REJECT_KMOL_H"]), ".2f"), "derived")
    add(st, "vEpurgeNo", format(P["E_PURGE"] * s, ".2f"), "derived")
    add(st, "cDiceCompPct", format(100 * ((7838 / tph) / (6400 / (P["CO2_CAPTURED_C6"] * ST / 24)) - 1), ".0f"))
    add(st, "vRbest", f1(R("RES_REACTOR_BEST_MWTH")), "results")
    add(st, "vRhigh", f1(R("RES_REACTOR_HIGH_MWTH")), "results")
    add(st, "cRlowScreen", "163.0", "screening", "printed only in the screening table, no results row")
    add(st, "vRahn", f1(d.sweep("HEAT_BASIS", "R_NEEDED", 143.4)), "results")
    add(st, "vRahnHeld", f1(d.sweep("HEAT_BASIS_HE_HELD", "R_NEEDED", 143.4)), "results")
    add(st, "vImpB", format(R("RES_GRID_IMPORT_AT_RATED_MWE"), ".1f"), "results")
    add(st, "vImpA", format(R("RES_GRID_IMPORT_AT_RATED_MWE", "case_a_mea"), ".1f"), "results")
    add(st, "cCoolKO", format(19.7 * MMBTU / 3.6, ".1f"))
    add(st, "cCoolHW", format(74.1 * MMBTU / 3.6, ".1f"))
    lhv_h2 = R("RES_H2_KG_H") / 3600 * 241.8 / MW_H2
    hhv_mw = R("RES_NG_MMSCFD") * 1e6 * P["NG_HHV"] * MMBTU * 1e-6 / 24 / 3.6
    add(st, "cEtaTh", format(lhv_h2 / (hhv_mw * 0.90 + R("RES_REACTOR_NEEDED_MWTH")), ".2f"), "derived")
    add(st, "vPlantCapRes", format(R("RES_PLANT_CAPTURE"), ".4f"), "results")
    ng = {"CH4": (93.571, MW_CH4), "C2": (3.749, 30.07), "C3": (0.920, 44.097), "C4": (0.260, 58.123),
          "C5": (0.040, 72.15), "C6": (0.010, 86.18), "N2": (1.190, 28.013), "O2": (0.010, 31.999),
          "CO2": (0.250, MW_CO2)}
    mng = sum(a * b for a, b in ng.values()) / 100
    add(st, "cMng", format(mng, ".2f"))
    add(st, "cYv", format(130 / 34.0, ".2f"))
    add(st, "cYm", format(34e6 / 379.49 / 24 * LB * mng / (P["H2_A"] * LB), ".2f"))
    for i, name in enumerate(("Zero", "One", "Two", "Three", "Four", "Five", "Six")):
        add(st, "vWalk" + name, f1(float(d.val[f"SIZING_WALK_STEP_{i}"]["simulated"])), "results")
    walk = {}
    for text in d.docs.splitlines():
        if text.startswith("|") and text.count("|") == 4:
            cells = [x.strip() for x in text.strip("|").split("|")]
            walk[cells[0]] = cells
    for name, step in (("Ab", "Restore purge load"), ("Ac", "Add vessel, IHX and duct losses"),
                       ("Ad", "Add helium circulators"), ("Ae", "Swap Selexol for MEA, zero reboiler heat")):
        add(st, "cWalk" + name, walk[step][2], "docs", "Case A walk, design decisions table")
    add(st, "cWalkAf", [c for k, c in walk.items() if k.startswith("Add reboiler gap")][0][2], "docs",
        "Case A walk, design decisions table")
    add(st, "vTfiveMod", format(R("TERM_T05_MODELLED"), ".2f"), "results")
    add(st, "vRanch", format(d.e("B_REC_F_ANCHORED", "HTGR_HEAT"), ".2f"), "results", "Anchored preset")

    # Section K.
    add(st, "cIeaBase", format(P["BASELINE_IEAGHG_KG_NM3"] / P["BASELINE_IEAGHG_RHO"], ".3f"))
    add(st, "cIeaFuel", format(100 * 4.332 / (26.231 + 4.332), ".1f"))
    add(st, "cEst", format(MW_CO2 / (4 * MW_H2), ".2f"))
    efeed = R("RES_FEED_C_KMOL_H") * MW_CO2 / R("RES_H2_KG_H")
    add(st, "cEfeed", format(efeed, ".2f"), "derived")
    add(st, "cChoProc", format(0.65 * P["BASELINE_CHO"], ".2f"))
    add(st, "cChoPct", format(100 * (0.65 * P["BASELINE_CHO"] / efeed - 1), ".1f"), "derived")
    add(st, "vEcapB", fmt_int(R("RES_CAPTURED_T_Y")), "results")
    add(st, "vAbCho", format(R("RES_ABATEMENT_CHO_T_Y") / 1e6, ".3f"), "results")
    add(st, "vAbIea", format(R("RES_ABATEMENT_IEAGHG_T_Y") / 1e6, ".3f"), "results")
    add(st, "vIntensity", format(R("RES_INTENSITY_KG_KG"), ".3f"), "results")
    add(st, "vReduction", format(100 * R("RES_REDUCTION"), ".1f"), "results")
    add(st, "vAbCapLow", format(d.sweep("CAPTURE", "A_T_Y", 0.85) / 1e6, ".3f"), "results")
    add(st, "vAbCapHigh", format(d.sweep("CAPTURE", "A_T_Y", 0.95) / 1e6, ".3f"), "results")
    add(st, "vAbCapEight", format(d.sweep("CAPTURE", "A_T_Y", 0.80) / 1e6, ".3f"), "results")
    for name, av in (("A", 0.65), ("B", 0.75), ("C", 0.85), ("D", 0.90)):
        add(st, "vAbAv" + name, format(d.sweep("AVAILABILITY", "A_T_Y", av) / 1e6, ".3f"), "results")
    add(st, "vAbateMt", format(abate / 1e6, ".3f"), "results")
    add(st, "vAbCombH", format(d.sweep("TAIL_GAS", "A_T_Y", run="B_COMB_H") / 1e6, ".3f"), "results")
    add(st, "vAbExport", format(d.sweep("TAIL_GAS", "A_T_Y", run="B_EXPORT") / 1e6, ".3f"), "results")
    add(st, "vAbCombN", format(d.sweep("TAIL_GAS", "A_T_Y", run="B_COMB_N") / 1e6, ".3f"), "results")
    em = {(r["baseline_id"], r["view"], r["line_id"]): float(r["t_per_y"]) for r in d.emis if r["run_id"] == "B_REC_F"}
    add(st, "vAbSystem", format(em[("INL_9p21", "system", "ABATEMENT")] / 1e6, ".3f"), "results")
    add(st, "vAbLife", format(em[("INL_9p21", "lifecycle_upstream", "ABATEMENT")] / 1e6, ".3f"), "results")
    add(st, "vAbBurden", format(em[("INL_9p21", "lifecycle_burdens", "ABATEMENT")] / 1e6, ".3f"), "results")
    fired = R("RES_PROCESS_HEAT_MW") * 3.6 * P["EF_NG_LHV"] / 0.85 / R("RES_H2_KG_H")
    add(st, "cBkTerm", format(fired, ".2f"), "derived")
    e_bk = R("RES_INTENSITY_KG_KG") + fired
    add(st, "cBkE", format(e_bk, ".2f"), "derived")
    out_h = 8760 - hours
    bk_h2 = R("RES_H2_KG_H") * out_h / 1000
    add(st, "cBkAnnual", fmt_int(bk_h2 * e_bk), "derived")
    add(st, "vBkHours", fmt_int(out_h), "derived")
    for name, run in (("No", "B_REC_F_CONV_UNABATED"), ("Cc", "B_REC_F_CONV_CAPTURE"), ("Heat", "B_REC_F_HEATER_K20")):
        add(st, "vBkEmit" + name, fmt_int(d.sweep("BACKUP_FIRING", "EMIT_BACKUP_T_Y", run=run)), "results")
        add(st, "vBkAdd" + name, fmt_int(d.sweep("BACKUP_FIRING", "A_EXTRA_T_Y", run=run)), "results")
    add(st, "vBkHtwo", fmt_int(d.sweep("BACKUP_FIRING", "H2_EXTRA_T_Y", run="B_REC_F_HEATER_K20")), "results")
    vented = d.sweep("CONDENSATE_CO2", "A_T_Y", 1.0, "B_REC_F_VENTED")
    add(st, "vCondVent", fmt_int(abate - vented), "derived", "abatement lost when condensate CO2 is vented")
    add(st, "vCondVentCost", format(d.sweep("CONDENSATE_CO2", "AC_L5_25", 1.0, "B_REC_F_VENTED")
                                    - d.sweep("CONDENSATE_CO2", "AC_L5_25", 0.0), ".2f"), "derived")
    add(st, "cInlCcCap", format(100 * P["CO2_CAPTURED_CONVCC"] / (P["CO2_CAPTURED_CONVCC"] + P["CO2_EMITTED_CONVCC"]), ".1f"))
    avoid = P["BASELINE_IEAGHG_KG_NM3"] - P["COMPARATOR_EMIT_CASE3_KG_NM3"]
    add(st, "cCmpAvoid", format(avoid, ".4f"))
    add(st, "cCmpAbate", format(avoid * P["H2_B"] * hours / 1000 / 1e6, ".3f"), "derived")
    add(st, "vCmpAbate", format(R("RES_COMPARATOR_ABATEMENT_T_Y") / 1e6, ".3f"), "results")
    add(st, "vAbAbove", fmt_int(abate - P["TARGET_ABATEMENT"]), "derived")

    # Section L inputs.
    i = P["DISCOUNT"]
    crf = lambda r, n: r / (1 - (1 + r) ** (-n))  # noqa: E731
    sff = lambda r, n: r / ((1 + r) ** n - 1)  # noqa: E731
    fidc_f = lambda r, t: ((1 + r) ** t - 1) / (r * t)  # noqa: E731
    tc = P["CONSTRUCTION_YEARS"]
    for name, n in (("cCrfT", 25), ("cCrfF", 40), ("cCrfS", 60)):
        add(st, name, format(crf(i, n), ".5f"))
    add(st, "cSffT", format(sff(i, 25), ".5f"))
    add(st, "cSffS", format(sff(i, 60), ".5f"))
    fidc = fidc_f(i, tc)
    add(st, "cFidc", format(fidc, ".3f"))
    F19, F14, FX = P["F_2019USD_TO_2025SGD"], P["F_2014EUR_TO_2025SGD"], P["FX"]
    lf = (1 - P["LEARNING"]) ** math.log2(P["LEARNING_UNITS"])
    add(st, "cLearn", format(lf, ".4f"))
    add(st, "cLearnF", format(1.6 * lf, ".3f"))
    add(st, "cOccPsix", fmt_int(P["OCC_CENTRAL"] * lf))
    add(st, "cOccPseven", fmt_int(P["OCC_CENTRAL"] * 1.6 * lf))
    add(st, "cTenHigh", fmt_int(5350 * lf))
    add(st, "cTenLow", fmt_int(1892 * lf))
    add(st, "cNgnp", format(1 + math.log(26553 / 18313) / math.log(350 / 600), ".4f"))
    K = P["CAPTURE_K_1A_DTCR_EURM"] * F14 / P["CAPTURE_K_1A_T_H"]
    add(st, "cKcap", format(K, ".2f"))
    add(st, "cKcapLow", format(K * P["CAPTURE_K_LOW_FACTOR"], ".2f"))
    add(st, "cKcapHigh", format(P["CAPTURE_K_3_DTCR_EURM"] * F14 / P["CAPTURE_K_3_T_H"], ".2f"))
    Cc = K * m_cap_th
    add(st, "cCapCap", f1(Cc), "derived")
    add(st, "cChemTerm", format(0.1 * F14 * 1e6 / abate, ".2f"), "derived")
    add(st, "vNgSaved", d.t("NG_SAVED_B_MMSCFD"), "results")
    ihx_a = P["IHX_LOOP_COST"] * P["DEFLATOR_2012_2025_NUM"] / P["DEFLATOR_2012_2025_DEN"] * FX
    Cihx = ihx_a * (R("RES_PROCESS_HEAT_MW") + R("RES_SLIP_HEAT_MW")) / P["IHX_REF_MW"]
    add(st, "cIhxAnchor", f1(ihx_a))
    add(st, "cIhxCap", f1(Cihx), "derived")
    Dec = P["DECOM_CENTRAL"] * P["REACTOR_RATED_MWTH"] * P["DECOM_KWE_PER_KWT"] * 1e3 / 1e6 * FX
    add(st, "cDecTotal", f1(Dec))
    Cpi = P["POWER_ISLAND_ADDBACK"] * R("RES_E_TOTAL_MWE") * 1e3 / 1e6 * F19
    add(st, "cPiCap", f1(Cpi), "derived")
    comp_k = P["RECYCLE_TRAIN_K_TPC_EURM"] / (P["CAPTURE_K_1A_T_H"] * 24 / ST * P["RECYCLE_TRAIN_K_KW_PER_STD"])
    Crec_a = P["E_PURGE"] * comp_k * P["TCR_OVER_TPC"] * F14
    Crec = Crec_a * s
    add(st, "cRecA", f1(Crec_a))
    add(st, "cRecB", f1(Crec), "derived")
    for k, n in (("", "N1P00"), ("m", "N0P60"), ("l", "N0P31")):
        add(st, "vRxB" + k, d.t(f"REACTOR_CAPITAL_BOAK_{n}_SM"), "results")
    add(st, "vCeiling", d.t("CEILING_SM_Y"), "results")

    # Ledger tables.
    lines = {"Gas": "GAS", "Rx": "REACTOR_CAP", "Cap": "CAPTURE_CAP", "Fom": "CAPTURE_FOM", "Ts": "TS",
             "Om": "REACTOR_OM", "Ihx": "IHX", "Lic": "LICENCE", "Dec": "DECOM", "Idc": "IDC",
             "Pi": "POWER_ISLAND", "Rec": "RECYCLE_TRAIN", "Steam": "LOST_STEAM", "Pow": "BASE_POWER"}
    runs = {"BT": ("BOAK", 25), "BS": ("BOAK", 60), "FT": ("FOAK", 25), "FS": ("FOAK", 60)}
    for tag, (stage, life) in runs.items():
        key = f"{stage}_{life}Y_N1P00"
        for nm, lid in lines.items():
            add(st, f"vL{nm}{tag}", d.tt(f"LINE_{lid}_{key}"), "results")
        for nm, lv in (("Three", "L3"), ("Four", "L4"), ("Five", "L5")):
            add(st, f"vAc{nm}{tag}", d.t(f"AC_{lv}_{key}"), "results")
            add(st, f"vCost{nm}{tag}", d.tt(f"COST_{lv}_{key}"), "results")
        for sf, n in (("m", "N0P60"), ("l", "N0P31")):
            add(st, f"vAcThree{tag}{sf}", d.t(f"AC_L3_{stage}_{life}Y_{n}"), "results")
            add(st, f"vAcFive{tag}{sf}", d.t(f"AC_L5_{stage}_{life}Y_{n}"), "results")
        add(st, f"vLac{tag}", d.t(f"LAC_DCF_{key}"), "results")
        add(st, f"vLcohNus{tag}", d.tt(f"LCOH_NUSH2_{key}"), "results")
        add(st, f"vGapPct{tag}", d.tt(f"LAC_L5_GAP_PCT_{key}"), "results")
        add(st, f"vGap{tag}", d.tt(f"LAC_L5_GAP_{key}"), "results")
    add(st, "vLcohSmrT", d.tt("LCOH_SMR_BOAK_25Y_N1P00"), "results")
    add(st, "vLcohSmrS", d.tt("LCOH_SMR_BOAK_60Y_N1P00"), "results")
    for name, token in (("vAcThreeNeedBT", "AC_L3_BOAK_25Y_N1P00_NEED"), ("vAcThreeNeedBS", "AC_L3_BOAK_60Y_N1P00_NEED"),
                        ("vAcFiveNeedBT", "AC_L5_BOAK_25Y_N1P00_NEED"), ("vAcFiveNeedBS", "AC_L5_BOAK_60Y_N1P00_NEED"),
                        ("vExcessFT", "EXCESS_L5_BOAK_25Y_SM_Y"), ("vExcessTT", "EXCESS_L3_BOAK_25Y_SM_Y")):
        add(st, name, d.t(token), "results")
    add(st, "vGapMax", d.tt("DCF_TEST_MAX_GAP_PCT"), "results")
    add(st, "vTargetPass", format(float(d.tok["TARGET_COST_N_PASS"]["table"]), ".0f"), "results")
    add(st, "vTargetN", format(float(d.tok["TARGET_COST_N_CENTRAL"]["table"]), ".0f"), "results")
    for tag, lv, life in (("FT", "L5", 25), ("FS", "L5", 60), ("TT", "L3", 25)):
        add(st, "vLevGas" + tag, d.t(f"LEVER_GAS_{lv}_BOAK_{life}Y_S_GJ"), "results")
        add(st, "vLevOcc" + tag, d.t(f"LEVER_OCC_{lv}_BOAK_{life}Y_USD_KWT"), "results")
        add(st, "vLevTs" + tag, d.t(f"LEVER_TS_{lv}_BOAK_{life}Y_S_T"), "results")
    for nm, sw, test, run in (("GasLo", "GAS", 10.0, "B_REC_F"), ("GasHi", "GAS", 20.0, "B_REC_F"),
                              ("TsLo", "TS", 23.2, "B_REC_F"), ("TsHi", "TS", 112.0, "B_REC_F"),
                              ("AvLo", "AVAILABILITY", 0.70, "B_REC_F"), ("AvHi", "AVAILABILITY", 0.90, "B_REC_F"),
                              ("Cho", "BASELINE", None, "B_REC_F_BASE_CHO"), ("Iea", "BASELINE", None, "B_REC_F_BASE_IEAGHG"),
                              ("Heat", "HEAT_BASIS", 143.4, "B_REC_F"), ("Comb", "TAIL_GAS", None, "B_COMB_H")):
        add(st, f"vSn{nm}T", f1(d.sweep(sw, "AC_L3_25", test, run)), "results")
        add(st, f"vSn{nm}S", f1(d.sweep(sw, "AC_L3_60", test, run)), "results")
        add(st, f"vSf{nm}T", f1(d.sweep(sw, "AC_L5_25", test, run)), "results")
        add(st, f"vSf{nm}S", f1(d.sweep(sw, "AC_L5_60", test, run)), "results")
    # Stacked MATURE_60Y run and the co-funding sweep, both solved by the engine.
    add(st, "vMatureCost", f1(d.sweep("MATURE_60Y", "COST_S_PER_T", 15.0, "MATURE_60Y_L5")), "results")
    add(st, "vMatureGas", format(d.sweep("MATURE_60Y", "GAS_NEEDED", run="MATURE_60Y_L5"), ".2f"), "results")
    target = [float(r["test_value"]) for r in d.sens if r["sweep_id"] == "MATURE_60Y" and r["metric"] == "GAS_NEEDED"][0]
    add(st, "vMatureTarget", format(target, ".0f"), "results", "target the run solves for")
    add(st, "vCofundShareT", f1(100 * d.sweep("COFUNDING", "COFUND_SHARE", run="L5_BOAK_25Y")), "results")
    add(st, "vCofundT", f1(d.sweep("COFUNDING", "COFUND_S_M", run="L5_BOAK_25Y")), "results")
    # The DCF side of MATURE_60Y, its printed gas price and the cost there, the route 4
    # step-down family and the rest of the co-funding rows, each from its engine row.
    printed = d.sweep("MATURE_60Y", "GAS_PRINTED", run="MATURE_60Y")
    central = [float(r["base_value"]) for r in d.sens
               if r["sweep_id"] == "MATURE_60Y" and r["metric"] == "GAS_PRINTED"][0]
    add(st, "vGasCentral", format(central, ".0f"), "results", "central gas price of the MATURE_60Y rows")
    add(st, "vMatureGasPrinted", format(printed, ".1f"), "results", "larger gas solve rounded up to S$0.1/GJ")
    add(st, "vMatureCostDcf", f1(d.sweep("MATURE_60Y", "COST_S_PER_T", central, "MATURE_60Y_DCF")), "results")
    add(st, "vMatureGasDcf", format(d.sweep("MATURE_60Y", "GAS_NEEDED", run="MATURE_60Y_DCF"), ".2f"), "results")
    for nm, run in (("Five", "MATURE_60Y_L5"), ("Dcf", "MATURE_60Y_DCF")):
        add(st, f"vMaturePrinted{nm}", f1(d.sweep("MATURE_60Y", "COST_S_PER_T", printed, run)), "results")
    steps = sorted({r["run_id"] for r in d.sens if r["sweep_id"] == "STEPDOWN"})
    for k, run in enumerate(steps):
        for nm, metric in (("Five", "AC_L5"), ("Dcf", "LAC_DCF")):
            add(st, f"vStep{WORDS[k + 1]}{nm}", f1(d.sweep("STEPDOWN", metric, run=run)), "results", run)
    add(st, "vReactorCapital", f1(d.sweep("COFUNDING", "REACTOR_CAPITAL_S_M", run="L5_BOAK_25Y")), "results")
    for tag, run in (("S", "L5_BOAK_60Y"), ("DcfT", "DCF_BOAK_25Y"), ("DcfS", "DCF_BOAK_60Y")):
        add(st, f"vCofundShare{tag}", f1(100 * d.sweep("COFUNDING", "COFUND_SHARE", run=run)), "results")
        add(st, f"vCofund{tag}", f1(d.sweep("COFUNDING", "COFUND_S_M", run=run)), "results")
    # Shift heat release of the Case B Level 3 flowsheet, the energy row of card E.6.
    add(st, "vWgsReleaseB", format(d.e("B_REC_F", "WGS_RELEASE"), ".2f"), "results")
    # Reformer exit through the flowsheet at 871, 825 and 800 C, and the assumed tube.
    for nm, test, run in (("A", 871.0, "FS_B_REC_F"), ("B", 825.0, "FS_B_REC_F_T825"),
                          ("C", 800.0, "FS_B_REC_F_T800")):
        add(st, f"vRefLfive{nm}", f1(d.sweep("REFORMER_T", "AC_L5_25", test, run)), "results")
    for nm, n in (("N", "N199"), ("M", "N5000")):
        add(st, f"vTubeExit{nm}", f1(d.r(f"TUBE_EXIT_C_{n}")), "results")
        add(st, f"vTubeShort{nm}", f1(d.r(f"TUBE_SHORTFALL_871C_K_{n}")), "results")
    # Capital rows of the tornado: INL's low and high OCC, and the capture capital range,
    # whose high end is IEAGHG Case 3.
    kcap = sorted({float(r["test_value"]) for r in d.sens if r["sweep_id"] == "CAPTURE_CAPITAL"})
    for nm, sw, test in (("OccLo", "REACTOR_CAPITAL", P["OCC_LOW"]), ("OccHi", "REACTOR_CAPITAL", P["OCC_HIGH"]),
                         ("KcLo", "CAPTURE_CAPITAL", kcap[0]), ("KcHi", "CAPTURE_CAPITAL", kcap[-1])):
        add(st, f"v{nm}", fmt_int(test) if test > 100 else format(test, ".2f"), "published" if nm.startswith("Occ") else "results", "sweep test value")
        for metric, suffix in (("AC_L3_25", "SnT"), ("AC_L3_60", "SnS"), ("AC_L5_25", "SfT"), ("AC_L5_60", "SfS")):
            add(st, f"v{suffix[:2]}{nm}{suffix[2]}", f1(d.sweep(sw, metric, test)), "results")
    for nm, rate in (("Five", 0.05), ("Eight", 0.08), ("Ten", 0.10)):
        add(st, f"vSwp{nm}T", f1(d.sweep("DISCOUNT", "AC_L5_25", rate)), "results")
        add(st, f"vSwp{nm}S", f1(d.sweep("DISCOUNT", "AC_L5_60", rate)), "results")
    add(st, "cCmpCac", f1(P["COMPARATOR_CAC_EUR"] * F14))
    adj = P["COMPARATOR_CAPTURED_CASE3_KG_NM3"] / avoid
    add(st, "cCmpT", f1(P["COMPARATOR_CAC_EUR"] * F14 + (P["TS_CENTRAL"] - P["TS_LOW"]) * adj))
    add(st, "cCmpHigh", f1(P["COMPARATOR_CAC_EUR"] * F14 + (P["TS_HIGH"] - P["TS_LOW"]) * adj))
    add(st, "vIslCap", d.t("SMR_ISLAND_CAPITAL_SM"), "results")
    add(st, "vIslFom", format(R("RES_SMR_ISLAND_FOM_SM_Y"), ".2f"), "results")
    add(st, "vIslVom", format(R("RES_SMR_ISLAND_VOM_SM_Y"), ".2f"), "results")

    # Discount sweep rebuilt here from the ledger lines, a check on the engine sweep above.
    def l5_at(rate: float, life: int) -> float:
        base = {k: d.line("BOAK", life, v) for k, v in lines.items()}
        crl0 = crf(i, life)
        crl, cr25 = crf(rate, life), crf(rate, 25)
        cr = base["Rx"] / crl0
        out = base["Gas"] + base["Fom"] + base["Ts"] + base["Om"] + base["Steam"] + base["Pow"]
        out += cr * crl + cr * (fidc_f(rate, tc) - 1) * crl
        out += Cc * cr25 + Cihx * cr25 + Crec * cr25 + Crec * P["CAPTURE_FOM"]
        out += base["Lic"] / crl0 * crl + base["Pi"] / crl0 * crl
        out += Dec * sff(rate, life)
        return out * 1e6 / abate

    for nm, rate in (("Five", 0.05), ("Eight", 0.08), ("Ten", 0.10)):
        add(st, f"cSwp{nm}T", f1(l5_at(rate, 25)), "derived")
        add(st, f"cSwp{nm}S", f1(l5_at(rate, 60)), "derived")

    # Discounted cash flow from the printed equations, independent of plant/dcf.rs.
    def pv(flows: list) -> float:
        return sum(x * (1 + i) ** (-y) for y, x in flows)

    def annual(life: int, x: float) -> list:
        return [(y, x) for y in range(1, life + 1)]

    def spend(total: float, years: float) -> list:
        whole = math.floor(years)
        part = years - whole
        out = [(-whole, total * part / years)] if part > 0 else []
        return out + [(k - whole + 1, total / years) for k in range(whole)]

    def replaced(cap: float, horizon: int, item_life: int = 25) -> list:
        out, b = [], 0
        while b < horizon:
            out.append((b, cap))
            b += item_life
        if b - horizon > 0:
            out.append((horizon, -cap * (b - horizon) / item_life))
        return out

    isl_cap = P["SMR_ISLAND_TCR_EURM"] * F14
    isl_fom = P["SMR_ISLAND_FOM_EUR_Y"] / 1e6 * F14
    isl_vom = ((P["SMR_ISLAND_RAW_WATER_EUR_Y"] + P["SMR_ISLAND_CHEMICALS_EUR_Y"]) / 1e6 * F14
               * P["AVAILABILITY"] / P["SMR_ISLAND_AVAILABILITY"])
    gas_per = P["NG_HHV"] * MMBTU * days * P["GAS_CENTRAL"] / 1e6
    ng_smr, ng_nus = P["NG_CONV_NOCC"] * s, P["NG_CASE6"] * s
    steam_per = MMBTU * hours * P["GAS_CENTRAL"] / 1e6
    steam_smr = P["EXPORT_STEAM_CONV"] * s
    power_smr = P["E_CONV_NOCC"] * s * hours * P["ELECTRICITY_PRICE"] / 1e3
    ts = P["TS_CENTRAL"] * R("RES_CAPTURED_T_Y") / 1e6
    fom = P["CAPTURE_FOM"]
    chk = {}
    for tag, (stage, life) in runs.items():
        stage_m = {"BOAK": 1.0, "FOAK": P["FOAK_CENTRAL"]}[stage]
        cr = P["OCC_CENTRAL"] * P["REACTOR_RATED_MWTH"] * 1e3 / 1e6 * F19 * stage_m
        rom = d.line(stage, life, "REACTOR_OM")
        smr = (pv(replaced(isl_cap, life)) + pv(annual(life, isl_fom + isl_vom)) + pv(annual(life, ng_smr * gas_per))
               + pv(annual(life, -steam_smr * steam_per)) + pv(annual(life, power_smr)))
        parts = {
            "Isl": pv(replaced(isl_cap, life)) + pv(annual(life, isl_fom + isl_vom)),
            "Gas": pv(annual(life, ng_nus * gas_per)), "Rx": pv(spend(cr, tc)),
            "Cap": pv(replaced(Cc, life)) + pv(annual(life, Cc * fom)), "Ts": pv(annual(life, ts)),
            "Om": pv(annual(life, rom)), "Ihx": pv(replaced(Cihx, life)),
            "Once": P["LICENCE_CENTRAL"] * FX + Cpi * stage_m, "Dec": Dec * (1 + i) ** (-life),
            "Rec": pv(replaced(Crec, life)) + pv(annual(life, Crec * fom)),
        }
        nus = sum(parts.values())
        ann = sum((1 + i) ** (-y) for y in range(1, life + 1))
        lac = (nus - smr) * 1e6 / (abate * ann)
        ln, ls = nus * 1e6 / (h2_t_y * 1000 * ann), smr * 1e6 / (h2_t_y * 1000 * ann)
        chk[tag] = {"lac": lac, "t_lac": d.line(stage, life, "LAC_DCF", "dcf_value"), "smr": smr, "nus": nus,
                    "ann": ann, "ident": (ln - ls) * 1000 * h2_t_y / abate, "parts": parts, "cr": cr}
        add(st, f"cChkLac{tag}", f1(lac), "derived")
    worst = max(abs(c["lac"] - c["t_lac"]) for c in chk.values())
    add(st, "cChkLacMax", format(max(worst, 0.005) + 0.005, ".2f"), "derived")
    add(st, "cChkIdent", f1(chk["BT"]["ident"]), "derived")
    for tag, sfx in (("BT", "T"), ("BS", "S")):
        c = chk[tag]
        add(st, "cNpvSmr" + sfx, fmt_int(c["smr"]), "derived")
        add(st, "cNpvNus" + sfx, fmt_int(c["nus"]), "derived")
        for k, v in c["parts"].items():
            add(st, f"cPv{k}{sfx}", f1(v), "derived")
    add(st, "cAnnT", format(chk["BT"]["ann"], ".3f"))
    add(st, "cAnnS", format(chk["BS"]["ann"], ".3f"))
    fs = (0.5 * (1 + i) ** 5 + sum((1 + i) ** k for k in range(5))) / tc
    theta = sum((1 + i) ** (-25 * m) for m in range(3)) - 0.6 * (1 + i) ** (-60)
    add(st, "cFs", format(fs, ".4f"))
    add(st, "cTheta", format(theta, ".4f"))
    cr = chk["BT"]["cr"]
    g_rx_t = cr * (fs - fidc) * crf(i, 25) * 1e6 / abate
    g_rx_s = cr * (fs - fidc) * crf(i, 60) * 1e6 / abate
    g_rep_s = (Cc + Cihx + Crec) * (theta * crf(i, 60) - crf(i, 25)) * 1e6 / abate
    add(st, "cGapRxT", format(g_rx_t, ".3f"), "derived")
    add(st, "cGapRxS", format(g_rx_s, ".3f"), "derived")
    add(st, "cGapRepS", format(g_rep_s, ".3f"), "derived")
    add(st, "cGapCloS", format(g_rx_s + g_rep_s, ".3f"), "derived")


# ----------------------------------------------------------------------------------------
# Validation status per card, written from results/Validation_Results.csv. A card prints the
# macro, so its text follows the results on every build and no outcome is typed by hand.
# ----------------------------------------------------------------------------------------
VALIDATION_LINKS = {
    "A.5": r"^FS_ENVELOPE_ELEMENTS_", "A.8": r"^FS_ENVELOPE_ENERGY_", "A.9": r"^PROPS-SHOMATE-(CP|H|S)$",
    "A.10": r"^XF-3-", "A.13": r"^XF-4$",
    "B.1": r"^PROPS-HE-PETERSEN$", "B.3": r"^(PROPS-HE-PETERSEN|HE-2-MU)$",
    "B.4": r"^(PROPS-HE-PETERSEN|HE-2-K)$", "B.8": r"^(T961-1-ROT\d+C|T953-3)$", "B.23": r"^T961-2-ROT\d+C$",
    "C.3": r"^IHX-HTTR-\d+-CLOSURE$", "C.11": r"^(IHX-INL-|IHX-HTTR-\d+-T[PS]$)",
    "D.7": r"^PROPS-K-(ZHANG-NIST|NIST-JANAF)$", "D.8": r"^PROPS-K(-ZHANG-NIST|2-ZHANG-LTS)$",
    "D.11": r"^(IA-[123]-|PROPS-TE1236-|PROPS-INL-A40-|T953-1-|T961-3-)", "D.13": r"^XF-[12]-",
    "D.16": r"^X-1$", "D.21": r"^JA-A-\d-X(-UNCAL)?$", "D.22": r"^JA-A-\d-X$",
    "D.25": r"^JA-A-\d-(S\d-(process|return)|Qproc|Qret)$", "D.26": r"^JA-A-\d-dP$",
    "D.28": r"^JA-A-\d-S\d-helium$", "D.33": r"^(T953-2|IA-4|IA-5)$", "D.34": r"^T961-4$",
    "E.4": r"^T961-[56]-CO(-NIST)?$", "E.5": r"^T961-[56]-T$", "E.7": r"^(CH-[123]|W-2)$", "E.9": r"^CH-[12]$",
    "E.10": r"^T961-5-GHSV-(CO|T)$", "E.15": r"^(T961-6-(T|CO|CO-NIST)|W-1)$", "E.16": r"^KR-1-",
    "F.1": r"^T961-7-(HPS-GEN1|WGS-EX[12])$", "F.2": r"^T961-7-WGS-EX[34]$", "F.4": r"^PROPS-IF97-VECTORS$",
    "F.9": r"^T961-7-KO-DRM-1$", "F.10": r"^T961-8-LIQ$", "F.11": r"^SCREENING_EX4_ABOVE_130_MW$",
    "F.13": r"^T961-8-CO2$",
    "G.13": r"^AR-1$", "G.14": r"^AR-1$", "G.15": r"^AR-1$", "G.16": r"^AR-1$", "G.17": r"^AR-1$",
    "G.19": r"^AR-(1|1-FIXTURE|2|3)$", "G.20": r"^AR-4$", "G.22": r"^VER-X$", "G.26": r"^HA-1$",
    "G.27": r"^HA-2$",
    "H.4": r"^SO-R\d+-CAP$", "H.5": r"^VER-[123]-", "H.8": r"^KH-[12]-", "H.9": r"^HA-3$",
    "H.10": r"^SO-R\d+-CAP$", "H.11": r"^SO-R\d+-CAP$", "H.12": r"^I-2$",
    "H.14": r"^SO-R\d+-(CAP|V\d+)$", "H.19": r"^SO-R\d+-(TI\d+|T-RMS)$",
    "H.27": r"^(SA-1|TAN-P[0-9.]+|TAN-TREND)$", "H.29": r"^ABS-CASEB-(DIAMETER|LOAD|HEIGHT|SPEC|RICH)$",
    "I.3": r"^(MO-K1[456]-(LEAN|TOUT)|VI-3-SS\d)$", "I.4": r"^DICE-RATIO$",
    "I.5": r"^(DICE-TREB|KW-TREB|DICE-TTOP)$", "I.7": r"^DICE-QCOND$",
    "I.12": r"^(DICE-SRD|KW-SRD.*|VI-[12])$",
    "J.3": r"^HEADLINE_H2_T_Y$", "J.4": r"^(HEADLINE_NG_MMSCFD|FS_T13_NG_FOR_CASE_B_H2)$",
    "J.5": r"^FS_CARBON_VS_ANCHOR_FEED$", "J.6": r"^FS_CARBON_VS_ANCHOR_CO2_ABS$",
    "J.8": r"^FS_CARBON_VS_ANCHOR_TUBE$", "J.9": r"^FS_CARBON_VS_ANCHOR_CAPTURED$",
    "J.10": r"^FS_CARBON_VS_ANCHOR_(TAIL_GAS|RECYCLED|BURNED)$",
    "J.11": r"^(CARBON_CLOSURE_ALL_RUNS|FS_ENVELOPE_ELEMENTS_.*|FS_CARBON_VS_ANCHOR_PRODUCT)$",
    "J.13": r"^FS_H2_RETURN_CLOSURE$", "J.14": r"^HEADLINE_PROCESS_HEAT$",
    "J.26": r"^(HEADLINE_REACTOR_NEED|FS_TEAR_THREE_STARTS)$", "J.35": r"^SIZING_WALK_STEP_\d$",
    "J.36": r"^HEADLINE_SPARE$", "K.11": r"^HEADLINE_ABATEMENT$", "L.25": r"^GOLD_LEDGER",
    "L.26": r"^HEADLINE_L3_", "L.37": r"^LAC_DCF_VS_L5_",
    "GATE.M6": r"^GATE_T0[2678]$", "GATE.M7": r"^(FS_GATE_M7|GATE_T(04|09|13))$",
}
WORDS = ("Zero One Two Three Four Five Six Seven Eight Nine Ten Eleven Twelve Thirteen Fourteen Fifteen "
         "Sixteen Seventeen Eighteen Nineteen Twenty").split()


def status_macro(card: str) -> str:
    """\\vVal plus the section letter plus the number in words, e.g. vValHfive for H.5."""
    sec, num = card.split(".")
    if sec == "GATE":
        return "vValGate" + {"M6": "Msix", "M7": "Mseven"}[num]
    n = int(num)
    word = WORDS[n] if n <= 20 else ("Twenty" + WORDS[n - 20] if n < 30 else "Thirty" + (WORDS[n - 30] if n > 30 else ""))
    return "vVal" + sec + word.lower()


def tex_plain(text: str) -> str:
    return text.replace("\\", "/").replace("_", "\\_").replace("%", "\\%").replace("&", "\\&").replace("#", "\\#")


def sig(text: str) -> str:
    try:
        v = float(text)
    except ValueError:
        return tex_plain(text)
    if v == 0:
        return "0"
    out = f"{abs(v):,.0f}" if abs(v) >= 1000 else f"{abs(v):.4g}"
    return ("$-$" if v < 0 else "") + out.replace(",", "{,}")


def status_text(rows: list[dict]) -> str:
    if not rows:
        return "No row in the validation results."
    counts: dict[str, int] = {}
    for row in rows:
        counts[row["status"]] = counts.get(row["status"], 0) + 1
    order = [("pass", "pass"), ("fail", "fail"), ("reported", "reported without a limit"), ("not_run", "not run")]
    parts = [f"{counts[k]} {w}" for k, w in order if counts.get(k)]
    ids = [r["test_id"] for r in rows]
    span = tex_plain(ids[0]) if len(ids) == 1 else f"{tex_plain(ids[0])} to {tex_plain(ids[-1])}"
    out = f"Validation results, {len(rows)} row{'s' if len(rows) > 1 else ''} ({span}): {', '.join(parts)}."

    def unit(r: dict) -> str:
        bare = not r["unit"] or r["unit"] == "-" or r["unit"].startswith("1 =")
        return "" if bare else " " + tex_plain(r["unit"])

    def limit(r: dict) -> str:
        # Flowsheet and cash-flow rows judge a relative error, as does a row whose explanation
        # names a relative limit; the others an absolute one.
        if (r["test_id"].startswith(("FS_", "LAC_DCF")) or r["unit"] == "relative"
                or "relative limit" in r["explanation"]):
            return f"{float(r['tolerance']) * 100:g}\\% relative"
        return f"{sig(r['tolerance'])}{unit(r)}"

    def against(r: dict) -> str:
        return f" against {sig(r['literature'])}" if r["literature"] else ""

    fails = [r for r in rows if r["status"] == "fail"]
    if fails:
        bits = [f"{tex_plain(r['test_id'])} gives {sig(r['simulated'])}{against(r)}{unit(r)}, limit {limit(r)}"
                for r in fails[:4]]
        out += " Failed: " + "; ".join(bits) + (f"; and {len(fails) - 4} more" if len(fails) > 4 else "") + "."
    same = [r for r in rows if r["data_use"] == "same_dataset_disclosed"]
    calib = [r for r in rows if r["data_use"] == "calibration"]
    indep = [r for r in rows if r["data_use"] == "independent"]
    if same and len(same) == len(rows):
        out += (" The results mark these rows as using the same data as the model or as a check of the"
                " code, so they are not validation evidence.")
    elif same:
        out += (f" The results mark {len(same)} of these rows as using the same data as the model, and"
                f" {len(indep)} as independent.")
    # A calibration row tests the fit only, whatever its status.
    if len(calib) == 1:
        out += f" {tex_plain(calib[0]['test_id'])} is a calibration row, so it tests the fit only."
    elif 1 < len(calib) <= 3:
        out += (f" {', '.join(tex_plain(r['test_id']) for r in calib)} are calibration rows, so they test"
                " the fit only.")
    elif calib:
        out += f" {len(calib)} of these rows are calibration rows, so they test the fit only."
    reported = [r for r in rows if r["status"] == "reported" and r["simulated"] and r["literature"]]
    if reported and len(reported) <= 4:
        bits = [f"{tex_plain(r['test_id'])} gives {sig(r['simulated'])} against {sig(r['literature'])}{unit(r)}"
                for r in reported]
        out += " Reported: " + "; ".join(bits) + "."
    elif reported:
        by_unit: dict[str, list[float]] = {}
        for r in reported:
            by_unit.setdefault(r["unit"], []).append(float(r["simulated"]) - float(r["literature"]))
        spans = [f"{sig(str(min(g)))} to {sig(str(max(g)))}{' ' + tex_plain(u) if u else ''} over {len(g)} rows"
                 for u, g in by_unit.items()]
        out += " Simulated less published: " + "; ".join(spans) + "."
    return out


def gate_text(d: Inputs, gate: str) -> str:
    """A clause naming why a gate failed or passed, from its results row."""
    if gate == "M6":
        row = d.val["GATE_T02"]
        judged = re.search(r"(\d+) rows judged", row["conditions"])
        failed = row["explanation"].split("Failed rows:", 1)[-1].split()
        n = int(float(row["simulated"] or 0))
        if row["status"] == "pass":
            return f"with all {judged.group(1) if judged else ''} judged rows inside their limits."
        return (f"with {n} of {judged.group(1) if judged else 'its'} judged rows outside their limits "
                f"({', '.join(tex_plain(x) for x in failed)}).")
    row = d.val["FS_GATE_M7"]
    if row["status"] == "pass":
        return "with every carbon line inside its limit."
    lines = row["explanation"].split(":", 1)[-1].split(".")[0].split()
    limit = re.search(r"outside (\S+)%", row["explanation"])
    return (f"with the carbon lines {', '.join(tex_plain(x) for x in lines)} outside the "
            f"{limit.group(1) if limit else ''}\\% limit against the anchored ledger.")


def values_status(d: Inputs, st: list) -> None:
    for card, pattern in VALIDATION_LINKS.items():
        if card.startswith("GATE."):
            add(st, status_macro(card), gate_text(d, card.split(".")[1]), "results", "gate status")
            continue
        rows = [r for r in d.val_rows if re.match(pattern, r["test_id"])]
        add(st, status_macro(card), status_text(rows), "results", "validation status")


def parse_test_log(path: str) -> dict[str, str]:
    """Test name to ok, FAILED or ignored, from the output of cargo test."""
    out: dict[str, str] = {}
    if not path:
        return out
    for line in Path(path).read_text(encoding="utf-8", errors="replace").splitlines():
        m = re.match(r"^test (\S+) \.\.\. (ok|FAILED|ignored)\s*$", line)
        if m:
            out[m.group(1).rsplit("::", 1)[-1]] = m.group(2)
    return out


# ----------------------------------------------------------------------------------------
# Engine links. A card is linked when a doc line "eq: X.n" sits on an item of the engine and
# the register names that item. Tags are matched whole, so A.1 never matches A.10.
# ----------------------------------------------------------------------------------------
ENGINE = ROOT / "engine"
NOT_IMPLEMENTED = "not implemented"
TAG_LINE = re.compile(r"^\s*//[/!]?\s*eq:\s*([A-L]\.\d+)\s*$")
DEF_LINE = re.compile(r"^(\s*)(?:pub(?:\([^)]*\))?\s+)?(?:(?:const|async|unsafe)\s+)*"
                      r"(fn|struct|enum|const|static|type|trait|mod)\s+([A-Za-z_]\w*)")
IMPL_LINE = re.compile(r"^impl\b(?:<[^>]*>)?\s+(?:[\w:]+(?:<[^>]*>)?\s+for\s+)?([A-Za-z_]\w*)")
MOD_LINE = re.compile(r"^(?:pub\s+)?mod\s+([A-Za-z_]\w*)\s*\{")


def module_of(path: Path) -> str:
    rel = path.relative_to(ENGINE).parts
    crate = rel[0]
    if rel[1] != "src":
        return "::".join([crate, rel[1]] + [p[:-3] if p.endswith(".rs") else p for p in rel[2:]])
    parts = list(rel[2:])
    if parts and parts[-1] in ("lib.rs", "mod.rs", "main.rs"):
        parts = parts[:-1]
    elif parts:
        parts[-1] = parts[-1][:-3]
    return "::".join([crate] + parts)


def engine_index() -> dict:
    """Items, eq tags and eq tests of the engine source, by module path."""
    items: set[str] = set()
    tags: dict[str, list[str]] = {}
    tests: dict[str, str] = {}
    for path in sorted(ENGINE.rglob("*.rs")):
        if "target" in path.parts:
            continue
        lines = path.read_text(encoding="utf-8").splitlines()
        mod = module_of(path)
        scope: list[str | None] = []
        owner: str | None = None
        inner: str | None = None
        for text in lines:
            m_impl, m_mod = IMPL_LINE.match(text), MOD_LINE.match(text)
            if m_impl:
                owner = m_impl.group(1)
            elif m_mod:
                inner = m_mod.group(1)
            scope.append(owner or inner)
            if text.startswith("}") or ((m_impl or m_mod) and text.rstrip().endswith("}")):
                owner = inner = None

        def path_of(j: int) -> str | None:
            m = DEF_LINE.match(lines[j])
            if not m:
                return None
            return "::".join(x for x in (mod, scope[j], m.group(3)) if x)

        for j, text in enumerate(lines):
            p = path_of(j)
            if p:
                items.add(p)
                name = DEF_LINE.match(text).group(3)
                if DEF_LINE.match(text).group(2) == "fn" and re.fullmatch(r"eq_[a-l]_\d+\w*", name):
                    tests[name] = p
        for i, text in enumerate(lines):
            m = TAG_LINE.match(text)
            if not m:
                continue
            target = None
            for j in range(i + 1, min(i + 40, len(lines))):
                s = lines[j].strip()
                if not s or s.startswith("//") or s.startswith("#["):
                    continue
                target = path_of(j)
                break
            if target is None:
                indent = len(text) - len(text.lstrip())
                for j in range(i - 1, -1, -1):
                    dm = DEF_LINE.match(lines[j])
                    if dm and dm.group(2) == "fn" and len(dm.group(1)) < indent:
                        target = path_of(j)
                        break
            if target:
                tags.setdefault(m.group(1), [])
                if target not in tags[m.group(1)]:
                    tags[m.group(1)].append(target)
    return {"items": items, "tags": tags, "tests": tests}


def tests_for(test_id: str, tests: dict) -> list[str]:
    """Engine tests named for a card: eq_x_nn, eq_x_nn_what, or a second card in the name."""
    tail = test_id[3:]
    out = []
    for name in tests:
        if name == test_id or name.startswith(test_id + "_"):
            out.append(name)
        elif re.search(r"_and_(eq_)?" + re.escape(tail) + r"(_|$)", name):
            out.append(name)
    return sorted(out)


def link_states(register: list[dict], index: dict, log: dict) -> dict:
    """Engine and test link state of every card."""
    eng: dict[str, list[str]] = {k: [] for k in ("linked", "tag_unnamed", "untagged_found", "untagged_missing",
                                                 "not_implemented", "theory", "none")}
    tst: dict[str, list[str]] = {k: [] for k in ("passed", "failed", "not_in_log", "no_test", "benchmark_none")}
    for row in register:
        fn = row["engine_fn"].replace(" (disabled)", "")
        names = [n.strip() for n in fn.split(";") if n.strip()]
        if fn == "theory":
            eng["theory"].append(row["id"])
        elif fn.lower().startswith("none"):
            eng["none"].append(row["id"])
        elif fn.lower().startswith(NOT_IMPLEMENTED):
            eng["not_implemented"].append(row["id"])
        else:
            found = all(n in index["items"] for n in names)
            tagged = row["id"] in index["tags"]
            if tagged and found and set(index["tags"][row["id"]]) <= set(names):
                eng["linked"].append(row["id"])
            elif tagged:
                eng["tag_unnamed"].append(row["id"])
            elif found:
                eng["untagged_found"].append(row["id"])
            else:
                eng["untagged_missing"].append(row["id"])
        if "NONE" in row["validation"]:
            tst["benchmark_none"].append(row["id"])
            continue
        names = tests_for(row["test_id"], index["tests"])
        if not names:
            tst["no_test"].append(row["id"])
        elif not log or any(n not in log for n in names):
            tst["not_in_log"].append(row["id"])
        elif all(log[n] == "ok" for n in names):
            tst["passed"].append(row["id"])
        else:
            tst["failed"].append(row["id"])
    return {"engine": eng, "tests": tst}


def link_sentence(links: dict) -> str:
    """The link counts in words. A state with no card is left out, so no zero is printed."""
    e, t = links["engine"], links["tests"]
    total = sum(len(v) for v in e.values())

    def join(bits: list[str]) -> str:
        return bits[0] if len(bits) == 1 else ", ".join(bits[:-1]) + " and " + bits[-1]

    rest = [f"{len(e[k])} {w}" for k, w in (
        ("untagged_found", "name an engine item that carries no tag yet"),
        ("untagged_missing", "name an item that the engine does not hold under that name"),
        ("tag_unnamed", "carry a tag on an item the card does not name"),
        ("not_implemented", "are stated as not implemented"), ("theory", "are theory"),
        ("none", "are listed only")) if e[k]]
    tests = [f"{len(t[k])} {w}" for k, w in (
        ("passed", "have an engine test named for them that passed in the recorded run"),
        ("failed", "have one that failed"), ("not_in_log", "have one that the recorded run does not show"),
        ("no_test", "have none")) if t[k]]
    out = (f"Of the {total} cards, {len(e['linked'])} are linked to the engine: a doc line names the card on "
           f"an engine item and the implementation line names that item.")
    if rest:
        out += f" Of the rest, {join(rest)}."
    if tests:
        out += (f" Of the {total - len(t['benchmark_none'])} cards with a benchmark code other than NONE, "
                f"{join(tests)}.")
    return out


def apply_links(rows: list[dict], index: dict) -> int:
    """Write the tagged items, or the present code a card names, into engine_fn."""
    cards = {c["id"]: c for c in all_cards()}
    changed = 0
    for row in rows:
        if row["engine_fn"] in ("theory",) or row["engine_fn"].lower().startswith("none"):
            continue
        new = None
        impl = cards[row["id"]]["fields"].get("Impl", "").replace("\\_", "_") if row["id"] in cards else ""
        if row["id"] in index["tags"]:
            new = "; ".join(index["tags"][row["id"]])
        elif impl.lower().startswith(NOT_IMPLEMENTED):
            new = NOT_IMPLEMENTED
        elif impl:
            named = [n for n in re.findall(r"\\fn\{([^}]+)\}", impl) if n in index["items"]]
            if named:
                new = "; ".join(dict.fromkeys(named))
        if new and new != row["engine_fn"]:
            row["engine_fn"] = new
            changed += 1
    return changed


def cmd_links(_: argparse.Namespace) -> int:
    rows = list(csv.DictReader(REGISTER.open(encoding="utf-8")))
    changed = apply_links(rows, engine_index())
    with REGISTER.open("w", newline="\n", encoding="utf-8") as fh:
        writer = csv.DictWriter(fh, REGISTER_FIELDS, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    print(f"links: engine_fn updated on {changed} rows of {REGISTER.relative_to(ROOT)}")
    return 0


def cmd_values(args: argparse.Namespace) -> int:
    # Pass 2: Case B is the Headline column of the results (case_b) and Case A is the INL
    # reference product with MEA and recycle (case_a_mea), as the engine exports them.
    d = Inputs()
    bb = fb = eb = d.basis("case_b", "B_REC_F")
    ba = fa = ea = d.basis("case_a_mea", "A_REC_F")
    carbon = {k: d.r(v) for k, v in (
        ("F", "RES_FEED_C_KMOL_H"), ("cap", "RES_CAPTURED_KMOL_H"), ("burn", "RES_BURNED_C_KMOL_H"),
        ("P", "RES_PRODUCT_C_KMOL_H"), ("T", "RES_TUBE_C_KMOL_H"), ("R", "RES_RECYCLED_C_KMOL_H"),
        ("G", "RES_TAILGAS_C_KMOL_H"), ("co2_abs", "RES_CO2_ABS_KMOL_H"))}
    st: list = []

    def pair(name: str, vb: float, va: float, spec: str, note: str = "", cls: str = "results") -> None:
        add(st, name + "B", format(vb, spec), cls, note)
        add(st, name + "A", format(va, spec), cls, note)

    def pair_int(name: str, vb: float, va: float, note: str = "", cls: str = "results") -> None:
        add(st, name + "B", fmt_int(vb), cls, note)
        add(st, name + "A", fmt_int(va), cls, note)

    s_b = bb["h2_kg_h"] / ba["h2_kg_h"]
    add(st, "vScaleB", format(s_b, ".4f"), "derived")
    add(st, "vScaleA", format(1.0, ".4f"), "assumed", "Case A is the reference")
    pair("vMdotHtwo", fb["h2_kg_h"] / 3600, fa["h2_kg_h"] / 3600, ".3f", cls="derived")
    pair_int("vMHtwo", fb["h2_t_y"], fa["h2_t_y"])
    add(st, "vHours", fmt_int(fb["hours"]), "results")
    pair("vNG", fb["ng_mmscfd"], fa["ng_mmscfd"], ".1f")
    pair("vQR", bb["R"], ba["R"], ".1f")
    pair("vQRx", bb["R"], ba["R"], ".2f")
    add(st, "vSpareB", format(d.r("RES_MARGIN_210_MWTH"), ".1f"), "results")
    add(st, "vSpareA", format(d.r("RES_MARGIN_210_MWTH", "case_a_mea"), ".1f"), "results")
    add(st, "vMsB", format(78.49 * s_b, ".1f"), "derived", "78.49 kg/s scaled by s")
    add(st, "vMsA", "78.49", "published")
    pair("vMsc", fb["he_sec_kg_s"], fa["he_sec_kg_s"], ".1f", "carries the slip heat too")
    pair("vMp", fb["he_prim_kg_s_at_R"], fa["he_prim_kg_s_at_R"], ".1f")
    add(st, "vMpAr", format(fa["he_prim_kg_s_at_R"], ".0f"), "results")
    pair("vQproc", bb["q_proc"], ba["q_proc"], ".1f")
    pair("vQprocx", bb["q_proc"], ba["q_proc"], ".2f")
    pair("vQrec", bb["q_recyc"], ba["q_recyc"], ".2f")
    pair("vWs", bb["w_s"], ba["w_s"], ".2f", "electric draw times the motor efficiency", "derived")
    pair("vWp", bb["w_p"], ba["w_p"], ".2f", "electric draw times the motor efficiency", "derived")
    pair("vWsum", bb["w_s"] + bb["w_p"], ba["w_s"] + ba["w_p"], ".2f", cls="derived")
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
    add(st, "vEtaPl", format(carbon["cap"] / carbon["F"], ".4f"), "derived")
    add(st, "vPhiC", format(carbon["co2_abs"] / carbon["T"], ".3f"), "derived")
    add(st, "vFr", format(carbon["R"] / carbon["G"], ".3f"), "derived")
    pair("vMcotwo", bb["cap_t"], ba["cap_t"], ".1f", cls="derived")
    pair_int("vMsolv", fb["solvent_t_h"], fa["solvent_t_h"])
    add(st, "vEbaseInt", format(eb["baseline"] / eb["h2_t_y"], ".2f"), "derived")
    pair_int("vEbase", eb["baseline"], ea["baseline"])
    pair_int("vEatm", eb["atm"], ea["atm"])
    pair_int("vAbate", eb["A_atm"], ea["A_atm"])
    add(st, "vACTwentyFive", d.t("AC_L3_BOAK_25Y_N1P00"), "results")
    add(st, "vACSixty", d.t("AC_L3_BOAK_60Y_N1P00"), "results")

    # Section A worked example, carbon closure, kmol/h at Case B.
    add(st, "cFeedC", format(carbon["F"], ".2f"), "results")
    add(st, "cCapC", format(carbon["cap"], ".2f"), "results")
    add(st, "cBurnC", format(carbon["burn"], ".2f"), "results")
    add(st, "cProdC", format(carbon["P"], ".2f"), "results")
    add(st, "cTubeC", format(carbon["T"], ".2f"), "results")
    add(st, "cRecyC", format(carbon["R"], ".2f"), "results")

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
    add(st, "cDutySel", format(scoped_tube_duty(d.r("RES_TUBE_C_KMOL_H", "case_a_inl")), ".1f"), "derived",
        "Case A, INL selexol basis")
    add(st, "cDutyRec", format(scoped_tube_duty(d.r("RES_TUBE_C_KMOL_H", "case_a_mea")), ".1f"), "derived",
        "Case A, recycle basis")
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
    # Worked-example arithmetic in sections B to D that takes a results value as input.
    uses_results = {"cRise", "cTRI", "cRITgap", "cCoreLHS", "cCoreRHS", "cCoreGap", "cMpExact", "cQav",
                    "cExtractW", "cDuctRoot", "cVesFrac", "cUAb", "cAreab", "cAreabHigh", "cUAratio",
                    "cUAmodule", "cUAmoduleRatio", "cUAfifty", "cImbalMpB", "cImbalUAB", "cImbalRatioB",
                    "cTubesB", "cVesHigh", "cWsHigh", "cTubesBLow", "cTubesBHigh"}
    st[:] = [(n, t, "derived" if n in uses_results and c == "calculated" else c, note) for n, t, c, note in st]

    # Sections I to L, the validation status of each linked card, and the test run.
    values_i_to_l(d, st)
    values_status(d, st)
    index = engine_index()
    log = parse_test_log(args.test_log)
    commit = subprocess.run(["git", "log", "-1", "--format=%h", "--", "engine"], cwd=ROOT,
                            capture_output=True, text=True).stdout.strip() or "unknown"
    if log:
        raw = Path(args.test_log).read_text(encoding="utf-8", errors="replace")
        counts = {k: len(re.findall(r"^test \S+ \.\.\. " + k + r"\s*$", raw, flags=re.M))
                  for k in ("ok", "FAILED", "ignored")}
        eq_tests = [n for n in index["tests"] if n in log]
        eq_ok = sum(1 for n in eq_tests if log[n] == "ok")
        add(st, "vTestRun", f"The engine test suite ran on the engine at commit {commit}: {counts['ok']} tests passed, "
            f"{counts['FAILED']} failed and {counts['ignored']} were ignored. Of these, {len(eq_tests)} carry an "
            f"equation name and {eq_ok} of them passed.", "results", "cargo test log")
    else:
        add(st, "vTestRun", "The engine test suite was not run for this build of the document.", "results")
    register = list(csv.DictReader(REGISTER.open(encoding="utf-8")))
    links = link_states(register, index, log)
    add(st, "vLinkSummary", link_sentence(links), "results", "engine and test links")

    # The basis statement on the cover and in the frame, from the classes above.
    numeric = [(n, t, c) for n, t, c, _ in st if re.fullmatch(r"-?[\d.,]+", t.replace("{,}", ",").replace("{}", ""))]
    counts: dict[str, int] = {}
    for _, _, cls in numeric:
        counts[cls] = counts.get(cls, 0) + 1
    parts = [f"{counts[c]} are {CLASS_WORDS[c]}" for c in ("results", "derived", "calculated", "published", "assumed")
             if counts.get(c)]
    basis = (f"Every number on these pages comes from the tools script. Of its {len(numeric)} numbers, "
             + ", ".join(parts[:-1]) + f" and {parts[-1]}.")
    docs_n = counts.get("docs", 0)
    scr = [t for _, t, c in numeric if c == "screening"]
    words = dict(enumerate("no one two three four five six seven eight nine".split()))
    if docs_n or scr:
        bits = []
        if docs_n:
            bits.append(f"{words.get(docs_n, str(docs_n))} values of the Case A sizing walk are read from the design "
                        "decisions table")
        if scr:
            bits.append(f"the low reactor corner of {', '.join(scr)} is taken from the screening table")
        text = " and ".join(bits) + ", because no results row holds them."
        basis += " " + text[0].upper() + text[1:]
    n_status = sum(1 for n, _, _, _ in st if n.startswith("vVal"))
    basis += f" The {n_status} validation status lines are written from the validation results file."
    add(st, "vNumberBasis", basis, "results", "class counts of this file")

    GENERATED.mkdir(parents=True, exist_ok=True)
    lines = [
        "% Generated by scripts/derivations_tools.py values. Never edit by hand.",
        "% Pass 2: the final results files, the build tokens, the parameter registry and the",
        "% small calculations in the tools script. The class of each value follows it.",
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
# Published numbers that happen to equal a generated value. Each is printed by its source.
PUBLISHED_LITERALS = {
    ("D_smr.tex", "1.198"): "Zhang et al. Table 1 prefactor of K1, printed",
    ("M_missing_sources.tex", "1.198"): "Zhang et al. Table 1 prefactor of K1, printed",
    ("L_economics.tex", "1,500"): "INL rounded OCC low value, printed",
}

BANNED = [r"Group\s*7", r"\btime\s*slot\b", r"\bdeadline\b", r"\bspeaker\b", r"AI[- ]declaration",
          r"AI[- ]use", r"\bA\d{7}[A-Z]\b", r"\bCN\s?\d{4}\b", r"\bAY\s?\d{4}", r"\bsemester\b",
          r"\bmodule code\b"]


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
        if reg["engine_fn"].lower().startswith(NOT_IMPLEMENTED):
            if not impl.lower().startswith(NOT_IMPLEMENTED):
                problems.append(f"{c['id']} register says not implemented and the card does not")
            continue
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
        if m and m.group(3) in CHECKED_CLASSES:
            value = m.group(2).replace("{,}", ",")
            if re.fullmatch(r"-?[\d,.]+", value) and len(re.sub(r"\D", "", value)) >= 4:
                literals[value] = m.group(1)
    hand_written = [p for p in SECTIONS.glob("*.tex") if p.name != "00_reference_frame.tex"]
    for path in hand_written:
        text = strip_comments(path.read_text(encoding="utf-8")).replace("{,}", ",")
        for value, name in literals.items():
            if (path.name, value) in PUBLISHED_LITERALS:
                continue
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
        index = engine_index()
        log = parse_test_log(args.test_log)
        rows = [r for r in register.values() if r["section"] in scope]
        links = link_states(rows, index, log)
        eng, tst = links["engine"], links["tests"]
        problems = ([f"{i} names an engine item with no eq tag" for i in eng["untagged_found"]]
                    + [f"{i} names an item the engine does not hold" for i in eng["untagged_missing"]]
                    + [f"{i} has a tag on an item it does not name" for i in eng["tag_unnamed"]])
        ok &= report("Engine links, pass 2", problems, limit=8)
        print("      counts: " + ", ".join(f"{k} {len(v)}" for k, v in eng.items()))
        problems = ([f"{i} has no engine test named eq_x_nn" for i in tst["no_test"]]
                    + [f"{i} test failed" for i in tst["failed"]]
                    + [f"{i} test not in the recorded run" for i in tst["not_in_log"]])
        ok &= report("Test links, pass 2", problems, limit=8)
        print("      counts: " + ", ".join(f"{k} {len(v)}" for k, v in tst.items()))
        print("      " + link_sentence(links))

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
    val.add_argument("--test-log", default="", help="output of cargo test, for the test run sentence")
    val.set_defaults(func=cmd_values)
    lnk = sub.add_parser("links")
    lnk.set_defaults(func=cmd_links)
    fr = sub.add_parser("frame")
    fr.set_defaults(func=cmd_frame)
    chk = sub.add_parser("check")
    chk.add_argument("--sections", default="")
    chk.add_argument("--pass2", action="store_true")
    chk.add_argument("--test-log", default="", help="output of cargo test, to mark linked tests as run")
    chk.add_argument("--log", default="")
    chk.add_argument("--aux", default="")
    chk.set_defaults(func=cmd_check)
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
