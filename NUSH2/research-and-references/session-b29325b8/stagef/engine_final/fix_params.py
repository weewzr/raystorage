"""Replace internal working-note pointers in engine/data/parameters.toml with
primary citations, decision ids or 'not verified'. Only source, locator and
source_status lines of the named ids change."""
import json
import sys
from pathlib import Path

C5 = "TEV-961 Case 5 stream tables, A-53 to A-56 (PDF p77 to p80)"
C5L = "A-53 to A-56, PDF p77 to p80"
CIRC_DP = ("loop drops of INL/EXT-06-11109 PDF p11 to p12 (50 kPa per heat exchanger and per "
           "pipe leg) and the JAERI-Tech 2005-031 reformer mock-up, App. A-6 PDF p116")
EX4 = ("Independent check of the WGS-EX4 heat above 130 C on a syngas cooling curve; Case 5 "
       "dry gas reconstructed from " + C5)

NEW = {
    "AVAILABILITY": ("Author assumption (decision D05); TEV-953 and TEV-961 state no availability "
                     "factor, and IEAGHG 2017-02 Table 4 uses 95%", "D05", None),
    "RHO_H2_NORMAL": ("Normal density of H2 at 0 C and 1 atm, a handbook value, not verified; "
                      "IEAGHG 2017-02 Table 2 PDF p16 implies 0.08994 (8.994 t/h per 100,000 Nm3/h)",
                      "Table 2, PDF p16", "not_verified"),
    "H2_B": ("Air Liquide press releases of 15 Jan 2008 and 30 Nov 2011: the SOXAL hydrogen plant on "
             "Jurong Island, close to 100,000 Nm3/h (decision D03)",
             "press releases 15 Jan 2008, 30 Nov 2011", None),
    "NG_NORECYCLE": ("TEV-953 Table 2 doc p29 (PDF p30): HTGR case with no capture, all PSA reject "
                     "fired, no recycle; a linear fit over TEV-961 Cases 2 to 5 gives 44.3",
                     "Table 2, PDF p30, doc p29", None),
    "WATER_GPM_CASE_A": ("TEV-961 Table 2 doc p16 (PDF p17): Case 6 process water, which excludes "
                         "reactor heat rejection", "Table 2, PDF p17, doc p16", "primary"),
    "CO2_CAPTURED_C6": ("TEV-961 Table 2 doc p16 (PDF p17). INL's ton is the short ton: the Case 5 "
                        "and Case 2 CO2 mass flows of TEV-961 A-50 and A-51 give the published tons "
                        "per day only in short tons", "Table 2, PDF p17, doc p16, A-50, A-51", None),
    "ETA_MEA": ("Author design value of the MEA train, 90% per pass on absorber-inlet CO2 "
                "(decision D25); the CAPTURE sweep runs 80 to 95%", "D25", None),
    "PHI_RECYCLE": ("CO2/(CO2+CH4+CO) after the shift, 3754/(3754+1020+70) lbmol/h, reconstructed "
                    "from " + C5, C5L, None),
    "BASELINE_IEAGHG_ROUNDED": ("IEAGHG 2017-02 Table 2 PDF p16 bound rounded to 9.00 kg/kg; kept "
                                "only for the frozen screening key A_900 of "
                                "engine/vv/data/freeze_values.json, and no result uses it",
                                "Table 2, PDF p16", None),
    "GRID_EF": ("Energy Market Authority 2024 grid emission factor, not verified",
                "grid emission factor 2024", "not_verified"),
    "EF_NG_LHV": ("IPCC 2006 Guidelines, Volume 2, Chapter 2, Table 2.2, natural gas, not verified",
                  "Vol. 2, Ch. 2, Table 2.2", "not_verified"),
    "EF_UPSTREAM": ("IEA 2026, Assessing Emissions from LNG Supply and Abatement Options, not verified",
                    "whole report", "not_verified"),
    "TS_BURDEN": ("Author assumption for the ship transport and storage burden, not verified",
                  "author assumption", "not_verified"),
    "EF_NUCLEAR": ("UNECE 2022, Life Cycle Assessment of Electricity Generation Options, nuclear, "
                   "not verified", "whole report", "not_verified"),
    "IHX_APPROACH_K": ("Primary return 491 C against the secondary return of 466 C, TEV-961 Table 2 "
                       "cont. doc p17 (PDF p18), the 25 K approach INL assumes (TEV-961 printed p.10)",
                       "Table 2 cont., PDF p18; printed p.10", None),
    "LOADING_RICH": ("Author design cap (decision D28); no kinetic, density or viscosity data for MEA "
                     "above a loading of 0.50 were found", "D28", None),
    "LOADING_LEAN": ("Author design value of the MEA train (decision D25); Guo et al. 2018 Table 2, "
                     "not verified", "Table 2", "not_verified"),
    "P_STRIPPER": ("Author design value of the MEA train (decision D25), not verified against a "
                   "syngas plant", "D25", "not_verified"),
    "Q_EX4_ABOVE_130C": ("TEV-961 Case 5 WGS-EX4 cooler, 300 to 266 F, A-55 to A-59 (PDF p79 to p83), "
                         "heat above 130 C by the author's calculation; the EX4_CHECK inputs check it "
                         "on an independent cooling curve", "A-55 to A-59, PDF p79 to p83", None),
    "SRD_LOW": ("Author range (decision D26): the low end needs good heat recovery; Vinjarapu et al. "
                "2024 minimum 3.46 GJ/t, Appl. Energy 355 122193 Table D.19 PDF p38",
                "Table D.19, PDF p38", None),
    "SRD_CENTRAL": ("Author central value (decision D26): Vinjarapu et al. 2024 Table D.19 PDF p38, "
                    "median 3.75 GJ/t over the 88 to 92% runs; Kwak et al. 2012 3.92; Nexant and "
                    "Bechtel 2020 DICE design 3.64", "Table D.19, PDF p38", None),
    "SRD_HIGH": ("Author range (decision D26): Brigman et al. 2014, Mongstad runs at 3.95 to 4.25 "
                 "GJ/t, Fig. 3", "Fig. 3", None),
    "CP_HE": ("Petersen 1970, Riso Report 224, Eq. 4-4: cp 5195 J/kg/K; TEV-961 implies 5190",
              "Eq. 4-4", None),
    "SLIP_CP_KJ_KMOL_K": ("Author assumption for the CO2 slip sensible heat, RWGS excluded, not "
                          "verified", "author assumption", None),
    "SLIP_DT_K": ("Author assumption for the CO2 slip temperature rise to the reformer, not verified",
                  "author assumption", None),
    "T_CIRC_SEC_K": ("Secondary circulator inlet at the secondary helium return, 466 C, TEV-961 "
                     "Table 2 cont. doc p17 (PDF p18)", "Table 2 cont., PDF p18", None),
    "T_CIRC_PRIM_K": ("Primary circulator inlet at the primary return, 491 C: the 466 C secondary "
                      "return of TEV-961 Table 2 cont. (PDF p18) plus INL's 25 K approach (TEV-961 "
                      "printed p.10)", "Table 2 cont., PDF p18; printed p.10", None),
    "LOSS_LOW_IHX": ("90 m duct of INL/EXT-06-11109 Table 3 PDF p15 scaled on flow; vessel loss "
                     "scaled with P^(2/3) from IAEA-TECDOC-1382 Table 3-21 PDF p164, the author's "
                     "scaling", "Table 3, PDF p15; Table 3-21, PDF p164", None),
    "LOSS_LOW_DUCT": ("90 m duct of INL/EXT-06-11109 Table 3 PDF p15 scaled on flow; vessel loss "
                      "scaled with P^(2/3) from IAEA-TECDOC-1382 Table 3-21 PDF p164, the author's "
                      "scaling", "Table 3, PDF p15; Table 3-21, PDF p164", None),
    "LOSS_CENTRAL_IHX": ("IHX loss of 0.8% from the HTTR heat balance residual, IAEA-TECDOC-1382 "
                         "Table 3-23", "Table 3-23", None),
    "LOSS_CENTRAL_DUCT": ("IHX loss of 0.8% from the HTTR heat balance residual, IAEA-TECDOC-1382 "
                          "Table 3-23", "Table 3-23", None),
    "LOSS_HIGH_IHX_MWTH": ("IHX 8.1 MWth from an earlier project allowance, not verified; 500 m duct "
                           "of INL/EXT-06-11109 Table 3 PDF p15; vessel 2.7% of power from "
                           "IAEA-TECDOC-1382 Table 3-21 PDF p164",
                           "Table 3, PDF p15; Table 3-21, PDF p164", None),
    "LOSS_HIGH_DUCT": ("IHX 8.1 MWth from an earlier project allowance, not verified; 500 m duct of "
                       "INL/EXT-06-11109 Table 3 PDF p15; vessel 2.7% of power from IAEA-TECDOC-1382 "
                       "Table 3-21 PDF p164", "Table 3, PDF p15; Table 3-21, PDF p164", None),
    "LOSS_HIGH_VESSEL_FRACTION": ("IHX 8.1 MWth from an earlier project allowance, not verified; "
                                  "500 m duct of INL/EXT-06-11109 Table 3 PDF p15; vessel 2.7% of "
                                  "power from IAEA-TECDOC-1382 Table 3-21 PDF p164",
                                  "Table 3, PDF p15; Table 3-21, PDF p164", None),
    "R_WORK_HIGH": ("Author upper value of the back-pressure work ratio, not verified",
                    "author assumption", None),
    "R_WORK_CENTRAL": ("IEAGHG 2017-02 Case 1A back-pressure work per unit of condensing heat "
                       "(PDF p68, p75 to p76), the author's calculation",
                       "PDF p68, p75 to p76", None),
    "CO_SHARE_OF_UNCONVERTED_CO": ("CO and CH4 leaving the shift, 70 and 1020 lbmol/h, reconstructed "
                                   "from " + C5, C5L, None),
    "CO_SHARE_OF_UNCONVERTED_CH4": ("CO and CH4 leaving the shift, 70 and 1020 lbmol/h, "
                                    "reconstructed from " + C5, C5L, None),
    "EX4_CHECK_DRY_KMOL_H": (EX4, C5L, None),
    "EX4_CHECK_CP_DRY": (EX4 + "; heat capacity an author assumption", C5L, None),
    "EX4_CHECK_CP_VAP": (EX4 + "; heat capacity an author assumption", C5L, None),
    "EX4_CHECK_LATENT": (EX4 + "; latent heat an author assumption", C5L, None),
    "EX4_CHECK_P_BAR": (EX4 + "; pressure an author assumption", C5L, None),
    "EX4_CHECK_T_IN_C": (EX4, C5L, None),
    "EX4_CHECK_T_OUT_C": (EX4 + "; outlet an author assumption", C5L, None),
    "PSA_REJECT_KMOL_H": ("TEV-961 Case 5 PSA reject molar flow reconstructed from " + C5 +
                          "; it raises the purge load with the CO2 slip", C5L, None),
    "ETA_MOTOR": ("Author assumption for pump and circulator motors, not verified",
                  "author assumption", None),
    "LIVES_SHORT": ("Author choice of economic life; 25 y and 60 y form the printed pair "
                    "(decision D08)", "D08", None),
    "LIVES_MID": ("Author choice of economic life; 25 y and 60 y form the printed pair "
                  "(decision D08)", "D08", None),
    "LIVES_LONG": ("Author choice of economic life; 25 y and 60 y form the printed pair "
                   "(decision D08)", "D08", None),
    "FX": ("FRED series AEXSIUS, 2025 annual average, S$1.3065 per US$", "AEXSIUS 2025", None),
    "F_2019USD_TO_2025SGD": ("US GDP implicit price deflator (BEA, FRED series GDPDEF) 128.885 over "
                             "103.9745, times FRED AEXSIUS 2025, 1.3065; a GDP deflator understates "
                             "nuclear escalation", "GDPDEF 2019 and 2025; AEXSIUS 2025", None),
    "F_2014EUR_TO_2025SGD": ("FRED AEXUSEU 2014 annual average, US GDP implicit price deflator (FRED "
                             "GDPDEF) 2014 mean 96.417 and 2025 128.885, and FRED AEXSIUS 2025, 1.3065",
                             "AEXUSEU 2014; GDPDEF; AEXSIUS 2025", None),
    "GAS_LOW": ("Author sensitivity value, not verified", "author assumption", None),
    "GAS_CENTRAL": ("IMF global LNG price, Asia (FRED PNGASJPUSDM), 2025 mean US$12.09/MMBtu, times "
                    "FRED AEXSIUS 1.3065 and over 1.05506 GJ/MMBtu, S$14.97/GJ",
                    "PNGASJPUSDM 2025; AEXSIUS 2025", None),
    "GAS_HIGH": ("Author sensitivity value, not verified", "author assumption", None),
    "TS_CENTRAL": ("Author central tariff (decision D10); ERIA and GCCSI 2024 Fig. 2.13 (report PDF "
                   "p121) puts transport in a Singapore CCS network at about US$43/t, S$56/t, with "
                   "storage extra", "Fig. 2.13, PDF p121", None),
    "OCC_CENTRAL": ("INL/RPT-23-72972 Rev 3 Table 1 (PDF p5) central US$6,000/kWe through the PDF p47 "
                    "recipe: x 0.91 x 0.4", "Table 1, PDF p5; PDF p47", None),
    "OPEX_CENTRAL": ("INL/RPT-23-72972 Rev 3 Table 1 (PDF p5) central OPEX US$25/MWh at 40% "
                     "conversion", "Table 1, PDF p5", None),
    "REACTOR_COST_REF_SIZE": ("600 MWth GTHTR300C class, Nishihara, Mouri and Kunitomi 2007, ICONE15, "
                              "not verified; INL/RPT-23-72972 Table 1 ties its cost to no size",
                              "Table 1, PDF p5", "not_verified"),
    "N_TEAM": ("Size exponent 0.6, Carelli et al. 2010 Prog. Nucl. Energy 52 403 and Peters, "
               "Timmerhaus and West 2003, not verified (decision D09)", "D09", None),
    "REACTOR_RATED_MWTH": ("Author decision D03: headline reactor rating and the size charged",
                           "D03", None),
    "IHX_LOOP_COST": ("IAEA-TECDOC-1682 Table 3.17, not verified", "Table 3.17", None),
    "IHX_REF_MW": ("IAEA-TECDOC-1682 Table 3.17, not verified", "Table 3.17", None),
    "DEFLATOR_2012_2025_NUM": ("Proxy deflator of an earlier project cost model, not verified",
                               "author assumption", "not_verified"),
    "DEFLATOR_2012_2025_DEN": ("Proxy deflator of an earlier project cost model, not verified",
                               "author assumption", "not_verified"),
    "IHX_LOW_USD_KWTH": ("General Atomics GA-A24265, US$43/kWth for the loop only, not verified",
                         "whole report", None),
    "IHX_LOW_REF_MWTH": ("General Atomics GA-A24265, US$43/kWth for the loop only, not verified",
                         "whole report", None),
    "CAPTURE_FOM": ("2.7% of capture capital per year, derived from IEAGHG 2017-02 cases, not verified "
                    "(decision D10)", "D10", None),
    "CAPTURE_ALLIN": ("Energy Transitions Commission exhibit, US$10 plus US$10 per t, not verified at "
                      "source; includes compression", "ETC exhibit", None),
    "ELECTRICITY_PRICE": ("Author assumption, not verified", "author assumption", None),
    "AMDEA_ETA": ("IEAGHG 2017-02 Case 1A, aMDEA on shifted syngas, 98.7% absorber removal (PDF p75 to "
                  "p76), the aMDEA sensitivity of decision D25", "PDF p75 to p76", None),
    "AMDEA_SRD": ("IEAGHG 2017-02 Case 1A, aMDEA on shifted syngas, about 3.03 GJ/t from its LP steam "
                  "(PDF p68, p75 to p76), the author's estimate, not verified; the aMDEA sensitivity "
                  "of decision D25", "PDF p68, p75 to p76", None),
}

for _k in ("P_MPA", "DP_SEC_KPA", "DP_PRIM_KPA", "ETA"):
    NEW["CIRC_LOW_" + _k] = ("Author pressure-drop budget, low case, from the " + CIRC_DP,
                             "PDF p11 to p12; App. A-6, PDF p116", None)
    NEW["CIRC_CENTRAL_" + _k] = ("Author budget, central case at 7.0 MPa, from the " + CIRC_DP +
                                 "; INL/EXT-11-21537 Table 2 gives circulator efficiency 0.75",
                                 "Table 2; PDF p11 to p12", None)
    NEW["CIRC_CENTRAL_5P15_" + _k] = ("Author budget, central case at 5.15 MPa, from the " + CIRC_DP,
                                      "PDF p11 to p12; App. A-6, PDF p116", None)
    NEW["CIRC_HIGH_" + _k] = ("Author budget, high case, from the " + CIRC_DP +
                              "; HTTR circulator map peak efficiency about 0.65, IAEA-TECDOC-1382 "
                              "Fig. 3.35 PDF p198", "Fig. 3.35, PDF p198", None)


def main(path):
    p = Path(path)
    lines = p.read_text(encoding="utf-8").split("\n")
    cur = None
    seen = set()
    out = []
    for line in lines:
        s = line.strip()
        if s == "[[param]]":
            cur = None
        elif s.startswith("id = "):
            cur = json.loads(s.split("=", 1)[1].strip())
        if cur in NEW:
            src, loc, status = NEW[cur]
            if s.startswith("source = "):
                line = "source = " + json.dumps(src, ensure_ascii=False)
                seen.add(cur)
            elif s.startswith("locator = "):
                line = "locator = " + json.dumps(loc, ensure_ascii=False)
            elif s.startswith("source_status = ") and status:
                line = "source_status = " + json.dumps(status)
        out.append(line)
    lost = set(NEW) - seen
    if lost:
        sys.exit(f"ids not found: {sorted(lost)}")
    p.write_bytes("\n".join(out).encode("utf-8"))
    print(f"rewrote {len(seen)} entries")


main(sys.argv[1])
