"""Freeze values for NUS-H2 from the screening model. Arithmetic only, read-only on project files.

Runs screening.py as a library, on the answered basis, and writes design/calc/freeze_values.json.
Set NUSH2_AVAIL to change availability; the driver at the bottom runs the cases it needs.
"""
import importlib.util
import json
import os
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
S = HERE.parent.parent
spec = importlib.util.spec_from_file_location("screening", S / "findings/scripts/screening.py")
m = importlib.util.module_from_spec(spec)
sys.modules["screening"] = m
spec.loader.exec_module(m)

SB = m.S_B
L3_KEYS = ["gas", "reactor_capital", "capture_capital", "capture_fom", "capture_allin", "ts", "reactor_om", "ihx", "licence", "decom"]
L4_KEYS = L3_KEYS + ["idc", "power_island", "recycle_train"]
L5_KEYS = L4_KEYS + ["lost_steam", "baseline_power", "export_revenue"]
L0_KEYS = L3_KEYS[:6]


def total(rows, keys):
    return sum(rows.get(k, 0.0) for k in keys)


def stage_rows(b, s, A, R, n, stage, life, **kw):
    r = m.ledger(b, s, stage=stage, life=life, n=n, R=R, **kw)
    return r, {lvl: total(r, keys) * 1e6 / A for lvl, keys in (("L0", L0_KEYS), ("L3", L3_KEYS), ("L4", L4_KEYS), ("L5", L5_KEYS))}


def case_block(s, label):
    b = m.balance("REC_F", s=s)
    c = b["c"]
    e = m.emissions(c, s)
    out = {"label": label, "s": s}
    h2_kgh = m.H2_A_KGH * s
    out["h2_kg_h"] = h2_kgh
    out["h2_t_y"] = h2_kgh * m.HOURS / 1000
    out["h2_nm3_h"] = h2_kgh / m.RHO_H2N
    out["hours"] = m.HOURS
    out["days"] = m.DAYS
    out["ng_mmscfd"] = m.NG_C6 * s
    out["ng_unabated_mmscfd"] = m.NG_CONV * s
    out["ng_norecycle_mmscfd"] = m.NG_NOREC * s
    out["ng_saved_mmscfd"] = (m.NG_CONV - m.NG_C6) * s
    out["ng_gj_d_saved"] = (m.NG_CONV - m.NG_C6) * s * m.NG_HHV * m.MMBTU_GJ
    out["gas_saving_SM_y"] = -m.ledger(b, s)["gas"]
    out["feed_C_kmol_h"] = m.F_C6 * s
    out["tube_C_kmol_h"] = c["T"] * s
    out["co2_abs_kmol_h"] = c["co2_abs"] * s
    out["cap_kmol_h"] = c["cap"] * s
    out["slip_kmol_h"] = c["slip"] * s
    out["tailgas_C_kmol_h"] = c["G"] * s
    out["recycled_C_kmol_h"] = c["R"] * s
    out["burned_C_kmol_h"] = c["burn"] * s
    out["product_C_kmol_h"] = c["P"] * s
    out["cap_t_h"] = b["cap_t"]
    out["baseline_t_y"] = e["baseline"]
    out["captured_t_y"] = e["captured"]
    out["atm_t_y"] = e["atm"]
    out["slip_t_y"] = m.tpy(c["slip"], s)
    out["tailgas_t_y"] = m.tpy(c["G"], s)
    out["recycled_t_y"] = m.tpy(c["R"], s)
    out["A_t_y"] = e["A_atm"]
    out["plant_capture"] = e["captured"] / (e["captured"] + e["atm"])
    out["reduction"] = e["A_atm"] / e["baseline"]
    out["ci_kg_kg"] = e["atm"] / out["h2_t_y"]
    out["baseline_kg_kg"] = e["baseline"] / out["h2_t_y"]
    out["never_generated_t_y"] = e["baseline"] - e["captured"] - e["atm"]
    out["A_cho"] = m.BASE_CHO * out["h2_t_y"] - e["atm"]
    out["A_ieaghg"] = m.BASE_IEAGHG * out["h2_t_y"] - e["atm"]
    out["A_900"] = 9.00 * out["h2_t_y"] - e["atm"]
    out["base_cho"] = m.BASE_CHO * out["h2_t_y"]
    out["base_ieaghg"] = m.BASE_IEAGHG * out["h2_t_y"]
    # energy
    for k in ("q_proc", "q_recyc", "q_reb", "avail", "gap", "q_ihx", "q_duct", "qv", "w_s", "w_p", "e_proc", "e_circ", "e_total", "branch", "R"):
        out[k] = b[k]
    out["stack"] = b["stack"]
    out["e_rows"] = b["e_rows"]
    out["circ_sec_MWe"] = b["w_s"] / m.ETA_MOTOR
    out["circ_prim_MWe"] = b["w_p"] / m.ETA_MOTOR
    out["he_sec_kg_s"] = m.HE_FLOW * (b["q_proc"] + b["q_recyc"]) / m.Q_PROC
    out["he_prim_kg_s_at_R"] = b["R"] * 1e6 / m.CP_HE / (m.ROT - (m.T_HE_R + 25.0))
    out["he_prim_kg_s_at_210"] = 210.0e6 / m.CP_HE / (m.ROT - (m.T_HE_R + 25.0))
    out["solvent_t_h"] = c["cap"] * s / (m.RICH - m.LEAN) * 61.08 / m.W_MEA / 1000
    out["water_gpm"] = 743.0 * s
    out["export_steam_used_all"] = b["export_kept"]
    out["margin_210"] = 210.0 - b["R"]
    # ranges from section 7.2 of screening
    out["R_best"] = m.balance("REC_F", s=s, srd=3.4, circ="low", loss="low", r_work=m.R_WORK["low_R"], use_zld=True, eta_p=m.ETA_PWR_ALT)["R"]
    out["R_high"] = m.balance("REC_F", s=s, srd=4.2, circ="high", loss="high", route="direct")["R"]
    # H2 streams, with PSA as an assumed external recovery
    h2_prod_kmol = h2_kgh / m.MW_H2
    out["psa_h2_feed_kg_h"] = h2_kgh / m.PSA_REC
    out["psa_h2_reject_kg_h"] = h2_kgh / m.PSA_REC - h2_kgh
    out["psa_h2_feed_kmol_h"] = h2_prod_kmol / m.PSA_REC
    return out, b, c, e


def main():
    res = {}
    res["avail"] = m.AVAIL
    bk, bb, cb, eb = case_block(SB, "B")
    ak, ba, ca, ea = case_block(1.0, "A")
    res["B"], res["A"] = bk, ak
    Ab, Aa = eb["A_atm"], ea["A_atm"]

    # INL Case 6 as published, Selexol, with the project's circulators and losses added
    bi = m.balance("INL_SEL")
    ci = bi["c"]
    ei = m.emissions(ci, 1.0)
    ri, lvi = stage_rows(bi, 1.0, ei["A_atm"], bi["R"], 1.0, "BOAK", 25)
    ri60, lvi60 = stage_rows(bi, 1.0, ei["A_atm"], bi["R"], 1.0, "BOAK", 60)
    res["INL"] = {"R": bi["R"], "e_proc": bi["e_proc"], "e_total": bi["e_total"], "circ_sec": bi["w_s"] / m.ETA_MOTOR, "circ_prim": bi["w_p"] / m.ETA_MOTOR,
                  "q_ihx": bi["q_ihx"], "q_duct": bi["q_duct"], "qv": bi["qv"], "branch": bi["branch"], "cap_t_y": ei["captured"], "atm_t_y": ei["atm"],
                  "A": ei["A_atm"], "plant": ei["captured"] / (ei["captured"] + ei["atm"]), "L3_25": lvi["L3"], "L3_60": lvi60["L3"],
                  "complete_220": m.Q_PROC + 17.3 / m.ETA_PWR, "margin_210": 210.0 - bi["R"]}

    # Cost grids
    grid = {}
    for nm, n in (("1.0", 1.0), ("0.6", 0.6), ("0.31", m.N_SCALE["NGNP"])):
        for R in (210.0, bb["R"]):
            for stage in ("FOAK", "BOAK", "10-OAK v1_7", "10-OAK INL"):
                for life in (25, 40, 60):
                    r, lv = stage_rows(bb, SB, Ab, R, n, stage, life)
                    grid[f"B|{R:.1f}|n{nm}|{stage}|{life}"] = lv
    for nm, n in (("1.0", 1.0), ("0.6", 0.6), ("0.31", m.N_SCALE["NGNP"])):
        for stage in ("FOAK", "BOAK", "10-OAK v1_7", "10-OAK INL"):
            for life in (25, 40, 60):
                r, lv = stage_rows(ba, 1.0, Aa, ba["R"], n, stage, life)
                grid[f"A|{ba['R']:.1f}|n{nm}|{stage}|{life}"] = lv
    res["grid"] = grid

    # Central lines, S$M/y
    lines = {}
    for life in (25, 40, 60):
        for stage in ("BOAK", "FOAK"):
            r, lv = stage_rows(bb, SB, Ab, 210.0, 1.0, stage, life)
            lines[f"B210|{stage}|{life}"] = {**r, **lv, "dC_L3": total(r, L3_KEYS)}
            r, lv = stage_rows(bb, SB, Ab, bb["R"], 1.0, stage, life)
            lines[f"B179|{stage}|{life}"] = {**r, **lv, "dC_L3": total(r, L3_KEYS)}
    res["lines"] = lines
    res["capex"] = {
        "BOAK_210_n1": m.reactor_capex(210.0, "central", "BOAK", 1.0),
        "FOAK_210_n1": m.reactor_capex(210.0, "central", "FOAK", 1.0),
        "BOAK_210_n06": m.reactor_capex(210.0, "central", "BOAK", 0.6),
        "BOAK_210_n031": m.reactor_capex(210.0, "central", "BOAK", m.N_SCALE["NGNP"]),
        "BOAK_179_n1": m.reactor_capex(bb["R"], "central", "BOAK", 1.0),
        "USD_BOAK_210": 2184.0 * 210e3 / 1e6,
        "USD_FOAK_210": 2184.0 * 210e3 / 1e6 * 1.6,
        "F2019": m.F2019,
    }
    res["capture"] = {"K": m.CAPT_K["central"], "capex": m.CAPT_K["central"] * bb["cap_t"], "K_high": m.CAPT_K["high"]}
    res["crf"] = {str(l): m.crf(m.RATE, l) for l in (25, 40, 60)}

    # T&S and other cost sensitivities at the central ledger
    base = lines["B210|BOAK|25"]
    def l3(**kw):
        b_ = kw.pop("b", bb)
        A_ = kw.pop("A", Ab)
        R_ = kw.pop("R", 210.0)
        n_ = kw.pop("n", 1.0)
        life_ = kw.pop("life", 25)
        stage_ = kw.pop("stage", "BOAK")
        r = m.ledger(b_, SB, stage=stage_, life=life_, n=n_, R=R_, **kw)
        return total(r, L3_KEYS) * 1e6 / A_
    sens = {
        "central": l3(),
        "ts_low": l3(ts=23.2), "ts_high": l3(ts=112.0),
        "ts_low_60y": l3(ts=23.2, life=60), "ts_high_60y": l3(ts=112.0, life=60),
        "gas_10": l3(gas=10.0), "gas_20": l3(gas=20.0),
        "n06": l3(n=0.6), "n031": l3(n=m.N_SCALE["NGNP"]),
        "srd34": l3(b=m.balance("REC_F", s=SB, srd=3.4)), "srd42": l3(b=m.balance("REC_F", s=SB, srd=4.2)),
        "capt_case3": l3(capt_k=m.CAPT_K["high"]), "fom6": l3(fom=m.FOM["high"]),
        "rx_p30": l3(rx_mult=1.3), "rx_m30": l3(rx_mult=0.7),
        "ahn_gas": l3(ng_override=m.NG_CONV * (24816.0 / 30563.0)),
        "cho": l3(A=Ab + (m.BASE_CHO - m.BASE_INL) * bk["h2_t_y"]),
        "ieaghg": l3(A=Ab + (m.BASE_IEAGHG - m.BASE_INL) * bk["h2_t_y"]),
        "R179": l3(R=bb["R"]),
        "R179_40": l3(R=bb["R"], life=40),
        "R179_60": l3(R=bb["R"], life=60),
    }
    res["sens_cost"] = sens
    ceiling = Ab * 100 / 1e6
    gas_per_sgj = bk["gas_saving_SM_y"] / 15.0
    excess = base["dC_L3"] - ceiling
    res["levers"] = {
        "ceiling_SM_y": ceiling,
        "dC_L3": base["dC_L3"],
        "excess": excess,
        "budget_after_capture_and_gas": ceiling - (base["capture_capital"] + base["capture_fom"] + base["ts"]) + bk["gas_saving_SM_y"],
        "nuclear_items_L3": base["reactor_capital"] + base["reactor_om"] + base["ihx"] + base["licence"] + base["decom"],
        "gas_price_needed": 15.0 + excess / gas_per_sgj,
        "occ_needed_USD_kWt": 2184.0 * (1 - excess / base["reactor_capital"]),
        "ts_needed": 60.0 - excess * 1e6 / bk["captured_t_y"],
    }

    # Emissions sensitivities at Case B
    es = {}
    for eta in (0.80, 0.85, 0.90, 0.95):
        c = m.carbon("REC_F", eta=eta)
        e = m.emissions(c, SB)
        es[f"eta{int(eta*100)}"] = {"A": e["A_atm"], "cap": e["captured"], "atm": e["atm"], "plant": e["captured"] / (e["captured"] + e["atm"])}
    for scn in ("INL_SEL", "REC_FIX", "COMB_H", "COMB_N", "EXPORT"):
        c = m.carbon(scn)
        e = m.emissions(c, SB)
        es[scn] = {"A": e["A_atm"], "cap": e["captured"], "atm": e["atm"], "plant": e["captured"] / (e["captured"] + e["atm"]), "NG": c["NG"] * SB}
    es["heat_alone_no_recycle"] = (m.CO2_CONV - m.CO2_HTGR_NOCC) * m.ST * m.DAYS * SB
    es["heat_plus_recycle_no_capture"] = bk["never_generated_t_y"]
    lc = m.lifecycle_adjust(bb, SB, bb["R"])
    gj = m.NG_HHV * m.MMBTU_GJ * m.DAYS
    up = (m.NG_CONV - m.NG_C6) * SB * gj * m.EF_UP / 1000.0
    nuc = bb["R"] * m.ETA_PWR * m.HOURS * m.EF_NUC / 1000.0
    ts_b = bk["captured_t_y"] * m.TS_BURDEN
    es["lifecycle"] = {"up_credit": up, "nuclear": nuc, "ts_burden": ts_b, "net_adjust": lc, "A_burdens_only": Ab - nuc - ts_b, "A_with_upstream": Ab + lc, "A_ts_inside": Ab - ts_b}
    es["system_adjust"] = m.system_adjust(bb, SB)
    es["system_view"] = Ab + es["system_adjust"]
    # Backup firing: outage hours run on a gas-fired conventional train with capture (TEV-953 Conv CC), H2 sold as extra
    out_h = 8760.0 - m.HOURS
    h2_extra = bk["h2_kg_h"] * out_h / 1000.0
    conv_cc_emit = 855.0 * m.ST / 24.0 * SB  # t/h
    conv_nocc_emit = 3205.0 * m.ST / 24.0 * SB
    es["backup"] = {"hours": out_h, "h2_extra_t_y": h2_extra, "emit_cc_t_y": conv_cc_emit * out_h, "emit_nocc_t_y": conv_nocc_emit * out_h,
                    "A_extra_cc_vs_new_smr": (conv_nocc_emit - conv_cc_emit) * out_h, "gas_MMSCFD_cc": 49.2 * SB, "gas_MMSCFD_nocc": 52.5 * SB}
    res["es"] = es

    # Walk at Case B
    q = m.Q_PROC * SB
    w = [("Slide 6 at 130 MMSCFD", m.Q_PROC + 11.6 / m.ETA_PWR)]
    w.append(("Same arithmetic scaled to Case B", q + 11.6 * SB / m.ETA_PWR))
    w.append(("Add 5.7 MWe x s", q + 17.3 * SB / m.ETA_PWR))
    w.append(("Add vessel, IHX and duct losses", m.balance("INL_SEL", s=SB, circ="none")["R"]))
    w.append(("Add helium circulators", m.balance("INL_SEL", s=SB)["R"]))
    w.append(("Swap Selexol for MEA with zero reboiler heat", m.balance("REC_F", s=SB, srd=0.0001)["R"]))
    w.append(("Add MEA reboiler gap at 3.8 GJ/t", bb["R"]))
    res["walk_B"] = w
    w2 = [("Slide 6", m.Q_PROC + 11.6 / m.ETA_PWR), ("Add 5.7 MWe", m.Q_PROC + 17.3 / m.ETA_PWR),
          ("Losses", m.balance("INL_SEL", circ="none")["R"]), ("Circulators", m.balance("INL_SEL")["R"]),
          ("MEA zero reboiler", m.balance("REC_F", srd=0.0001)["R"]), ("Gap", ba["R"])]
    res["walk_A"] = w2

    # Comparator: IEAGHG 2017-02 Case 3 at its own 100,000 Nm3/h, availability of this study
    avoided_kg_nm3 = 0.8091 - 0.0888
    capt_kg_nm3 = 0.8004
    cac_eur = 69.8
    cac_sgd = cac_eur * m.F2014
    ts_adj = (60.0 - 23.2) * capt_kg_nm3 / avoided_kg_nm3
    res["comparator"] = {"A_t_y": avoided_kg_nm3 * 100000 * m.HOURS / 1000, "cac_published": cac_sgd, "ts_adj": ts_adj, "cac_T60": cac_sgd + ts_adj,
                         "cac_T232": cac_sgd, "cac_T112": cac_sgd + (112.0 - 23.2) * capt_kg_nm3 / avoided_kg_nm3, "overall_capture": 0.90}

    # User Q8 line in one unit
    A_d, A_n = 0.585, Ab / 1e6
    nonrx = 38.09 - 31.5
    res["q8"] = {f"{life}y": {"S_M_y": nonrx + rx + 7.2, "at_0585": (nonrx + rx + 7.2) / A_d, "at_0578": (nonrx + rx + 7.2) / A_n,
                              "no72_at_0578": (nonrx + rx) / A_n} for life, rx in (("25", 51.2), ("60", 44.1))}
    print(json.dumps(res))


if __name__ == "__main__":
    if os.environ.get("NUSH2_CHILD"):
        main()
    else:
        all_res = {}
        for av in ("0.85", "0.70", "0.75", "0.90", "0.65"):
            env = dict(os.environ, NUSH2_AVAIL=av, NUSH2_CHILD="1", PYTHONIOENCODING="utf-8")
            p = subprocess.run([sys.executable, __file__], env=env, capture_output=True, text=True, encoding="utf-8")
            if p.returncode:
                print(p.stderr)
                raise SystemExit(1)
            all_res[av] = json.loads(p.stdout.strip().splitlines()[-1])
        (HERE / "freeze_values.json").write_text(json.dumps(all_res, indent=1), encoding="utf-8")
        r = all_res["0.85"]
        B, A = r["B"], r["A"]
        for k in ("h2_kg_h", "h2_t_y", "h2_nm3_h", "ng_mmscfd", "ng_unabated_mmscfd", "ng_norecycle_mmscfd", "ng_saved_mmscfd", "ng_gj_d_saved", "gas_saving_SM_y", "feed_C_kmol_h", "tube_C_kmol_h", "co2_abs_kmol_h", "cap_kmol_h", "slip_kmol_h", "tailgas_C_kmol_h", "recycled_C_kmol_h", "burned_C_kmol_h", "product_C_kmol_h", "cap_t_h", "baseline_t_y", "captured_t_y", "atm_t_y", "slip_t_y", "tailgas_t_y", "recycled_t_y", "A_t_y", "plant_capture", "reduction", "ci_kg_kg", "baseline_kg_kg", "never_generated_t_y", "A_cho", "A_ieaghg", "A_900", "base_cho", "base_ieaghg", "q_proc", "q_recyc", "q_reb", "avail", "gap", "q_ihx", "q_duct", "qv", "w_s", "w_p", "e_proc", "e_circ", "e_total", "branch", "R", "circ_sec_MWe", "circ_prim_MWe", "he_sec_kg_s", "he_prim_kg_s_at_R", "he_prim_kg_s_at_210", "solvent_t_h", "water_gpm", "export_steam_used_all", "margin_210", "R_best", "R_high", "psa_h2_feed_kg_h", "psa_h2_reject_kg_h"):
            print(f"{k:28s} B {B[k]:14,.4f}   A {A[k]:14,.4f}")
        print("stack B", B["stack"], "\nstack A", A["stack"])
        print("e_rows B", {k: round(v, 3) for k, v in B["e_rows"].items()})
        print("e_rows A", {k: round(v, 3) for k, v in A["e_rows"].items()})
        print("walk B", [(a, round(b, 1)) for a, b in r["walk_B"]])
        print("walk A", [(a, round(b, 1)) for a, b in r["walk_A"]])
        print("capex", r["capex"], "\ncapture", r["capture"], "\ncrf", r["crf"])
        print("levers", r["levers"])
        print("sens", {k: round(v, 1) for k, v in r["sens_cost"].items()})
        print("es", json.dumps(r["es"], indent=0)[:2400])
        print("comparator", r["comparator"], "\nq8", r["q8"])
        for key in ("B|210.0|n1.0|BOAK|25", "B|210.0|n1.0|BOAK|40", "B|210.0|n1.0|BOAK|60", "B|210.0|n1.0|FOAK|25", "B|210.0|n1.0|FOAK|60", "B|210.0|n1.0|10-OAK v1_7|25", "B|210.0|n1.0|10-OAK INL|25", "B|210.0|n0.6|BOAK|25", "B|210.0|n0.6|BOAK|60", "B|210.0|n0.31|BOAK|25", "B|210.0|n0.31|BOAK|60"):
            print(key, {k: round(v, 1) for k, v in r["grid"][key].items()})
        for k, v in r["grid"].items():
            if k.startswith("A|") and "n1.0" in k and ("BOAK|25" in k or "BOAK|60" in k or "FOAK|25" in k or "FOAK|60" in k):
                print(k, {a: round(b, 1) for a, b in v.items()})
        print("lines B210 BOAK 25", {k: round(v, 2) for k, v in r["lines"]["B210|BOAK|25"].items()})
        print("lines B210 BOAK 60", {k: round(v, 2) for k, v in r["lines"]["B210|BOAK|60"].items()})
        for av in ("0.65", "0.70", "0.75", "0.90"):
            x = all_res[av]
            print("avail", av, "A", round(x["B"]["A_t_y"]), "AC L3 25", round(x["grid"]["B|210.0|n1.0|BOAK|25"]["L3"], 1), "H2", round(x["B"]["h2_t_y"]))
