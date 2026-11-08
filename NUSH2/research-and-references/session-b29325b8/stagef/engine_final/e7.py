import sys
sys.path.insert(0, r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/engine_final")
from e1lib import rep

H = "engine/vv/src/harness.rs"
rep(H, '''        source: "screening.py via design calc goldens".to_string(),
        locator: "engine_goldens.json, freeze_values.json".to_string(),''',
    '''        source: "the author's screening calculation, frozen as goldens".to_string(),
        locator: "engine/vv/data/engine_goldens.json, engine/vv/data/freeze_values.json"
            .to_string(),''')
rep(H, '''        source: "screening.py, printed in the freeze".to_string(),
        locator: "freeze sections 1, 5, 6".to_string(),''',
    '''        source: "the author's screening calculation, frozen as goldens".to_string(),
        locator: "engine/vv/data/freeze_values.json".to_string(),''')
rep(H, '''            locator: "build order section 2".to_string(),''',
    '''            locator: "decision D08".to_string(),''')

C = "engine/units/src/bench/columns.rs"
rep(C, '''        source: "Plan section 7 limits; USER_ANSWERS absorber 40 C, 26.8 bar, rich cap 0.50".into(),''',
    '''        source: "Author limits: absorber 40 C, 26.8 bar and rich cap 0.50 (decision D28), packed \\
                 height at most 40 m"
            .into(),''')

S = "engine/vv/data/screening_checks.csv"
rep(S, "100000 Nm3/h at 0.08988 kg/Nm3 and 85% availability,freeze section 4.1",
    "100000 Nm3/h at 0.08988 kg/Nm3 and 85% availability,author calculation")
rep(S, "Mass ratio of Case B to the INL product,freeze section 4",
    "Mass ratio of Case B to the INL product,author calculation")
rep(S, "Process heat plus electricity without purge at 39.7%,freeze section 5",
    "TEV-961 process heat plus electricity without purge at 39.7%,Table 2 PDF p17 to p18")
rep(S, "Process heat plus full Case 6 electricity at 39.7%,freeze section 5",
    "TEV-961 process heat plus full Case 6 electricity at 39.7%,Table 2 PDF p17 to p18")
rep(S, ",screening report\n", ",author calculation\n", n=5)
rep(S, "Reader sum at 7 MPa with circulators and losses,htgr_ihx.md s8",
    "Author sum at 7 MPa with circulators and losses,author calculation")
rep(S, "Reader sum at 70.4 t/h and SRD 3.8,mea_calc2 output",
    "Author sum at 70.4 t/h and SRD 3.8,author calculation")
rep(S, "Independent cooling curve against the published chain scale,inl_calcs_output",
    "Independent cooling curve against the published chain scale,TEV-961 Case 5 A-55 to A-59")
print("ok")
