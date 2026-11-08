//! Benchmarks of `docs/VALIDATION_MATRIX.md` that do not run, each written as a
//! `not_run` row with its reason. Where the matrix names one printed target, the
//! row carries it, read from the fixture.

use crate::fixture::Fixture;
use crate::report::Row;
use crate::rowkit::{Meta, INDEPENDENT, SAME};
use nush2_numerics::Result;

const NO_ROUTE: &str = "no harness route was built for this benchmark in this pass";

/// Matrix rows with a fixture behind them that do not run.
pub fn matrix_rows(out: &mut Vec<Row>) -> Result<()> {
    let khan = "Khan, Krishnamoorthi and Mahmud 2011, Chem. Eng. Res. Des. 89, 1600-1608";
    let onda = "Khan prints no specific area or critical surface tension for the Berl saddles \
                of run T22 (Table 1 and Section 3, PDF p5) and no local source holds them, so \
                Onda's Eqs 5a to 5c cannot run (derivation note M17)";
    out.push(
        Meta::new("KH-3", 2, "Gas CO2 profile, run T22", khan, "Fig 3a PDF p5")
            .unit("mol%")
            .use_(SAME)
            .cond("7 heights, RMS limit 2.0 mol%")
            .not_run(None, onda),
    );
    out.push(
        Meta::new(
            "KH-4",
            2,
            "Liquid MEA profile, run T22",
            khan,
            "Fig 3b PDF p5",
        )
        .unit("mol/m3")
        .use_(SAME)
        .cond("7 heights, RMS limit 300 mol/m3")
        .not_run(None, onda),
    );

    let isa = Fixture::open("isa_2017.csv")?;
    let y = isa.get("ytop_conclusion", "Y_CO2_top")?;
    out.push(
        Meta::at(y, "IS-1", 2, "Y CO2 at the column top", "mol CO2/mol CH4")
            .cond("Pressures up to 50 bar, 1.505 m, 0.145 m, Flexipac 1Y, limit 0.05 absolute")
            .not_run(
                Some(y.value),
                &format!(
                    "{NO_ROUTE}. The absorber would need the Table 3 inlet state at each pressure"
                ),
            ),
    );

    let ah = Fixture::open("abdul_halim_2015.csv")?;
    let rise = ah.get("fig4_text", "CO2_removal_increase")?;
    let gauze = "Sulzer metal gauze geometry is not in the packing set of nush2_units, which \
                 holds RSP-250 and Flexipac 1Y, and the paper prints no lean loading";
    out.push(
        Meta::at(rise, "AH-1", 2, "Column temperature profile", "C")
            .locator("Table 3 PDF p5")
            .cond("Eight conditions, 6 heights, RMS limit 5 K")
            .not_run(None, gauze),
    );
    out.push(
        Meta::at(rise, "AH-2", 2, "Rise in CO2 removal with liquid load", "%")
            .cond("1.0 kmol/m3, 19.72 kmol/m2/h, 27 C, 1.81 to 4.51 m3/m2/h; sign, then 20 points")
            .not_run(Some(rise.value), gauze),
    );

    let br = Fixture::open("brigman_2014.csv")?;
    let coupled = "needs the absorber and stripper coupled on flue gas at about 1 bar with a \
                   lean loading sweep, and no harness route was built for it";
    let srd30 = br.get("min_30wt", "SRD_min")?;
    out.push(
        Meta::at(srd30, "BR-1-SRD", 2, "Minimum SRD, 30 wt%", "MJ/kg CO2")
            .cond("24 m, 1.9 bara, 47000 Sm3/h, 25 C, capture 85%, limit 10%")
            .not_run(Some(srd30.value), coupled),
    );
    let lean = br.get("min_30wt", "lean_loading_at_min")?;
    out.push(
        Meta::at(
            lean,
            "BR-1-LEAN",
            2,
            "Lean loading at the SRD minimum, 30 wt%",
            "mol/mol",
        )
        .cond("As BR-1-SRD, limit 0.05")
        .not_run(Some(lean.value), coupled),
    );
    let srd40 = br.get("min_40wt", "SRD_min")?;
    out.push(
        Meta::at(srd40, "BR-2", 2, "Minimum SRD, 40 wt%", "MJ/kg CO2")
            .cond("59000 Sm3/h, limit 10%")
            .not_run(Some(srd40.value), coupled),
    );
    out.push(
        Meta::at(
            srd30,
            "BR-3",
            3,
            "SRD and rich loading, 18 m against 24 m packing",
            "MJ/kg CO2",
        )
        .locator("Section 4.3 text PDF p7")
        .cond("Sign, then 0.03 on loading")
        .not_run(None, coupled),
    );
    out.push(
        Meta::at(
            srd30,
            "BR-4",
            3,
            "SRD, flue gas 50 C against 25 C",
            "MJ/kg CO2",
        )
        .locator("Section 4.5 text PDF p9")
        .cond("Sign only")
        .not_run(None, coupled),
    );

    let ch = "Chinchen et al. 1984, Appl. Catal. 12, 69-88";
    out.push(
        Meta::new("CH-3", 2, "Microreactor CO conversion", ch, "Table 2 PDF p7")
            .unit("%")
            .cond("31.6 bar, GHSV 5000 1/h STP, crushed 15-5, limit 10 points")
            .not_run(
                None,
                &format!("{NO_ROUTE}. wgs::hts sizes adiabatic beds, and the microreactor bed of PDF p5 was not set up"),
            ),
    );

    let kr = Fixture::open("wgs_kraussler_2017.csv")?;
    for op in ["OP1", "OP2", "OP3", "OP4"] {
        let fix = kr
            .select(|r| r.series == op && r.quantity == "outlet_CO" && r.data_kind == "measured")
            .into_iter()
            .next()
            .ok_or_else(|| nush2_numerics::EngineError::Parse(format!("Kraussler {op}")))?;
        out.push(
            Meta::at(fix, format!("KR-1-{op}"), 2, "Outlet CO, dry", "vol%")
                .cond("Ambient pressure, limit 2.0 vol%")
                .not_run(
                    Some(fix.value),
                    &format!("{NO_ROUTE}. The catalyst differs from Chinchen's and the gas carries H2S and tar"),
                ),
        );
    }
    Ok(())
}

/// Section 18: benchmarks that cannot run from local sources.
pub fn section18_rows(out: &mut Vec<Row>) {
    let loc = "docs/VALIDATION_MATRIX.md section 18";
    let rows: [(&str, u8, &str, &str, &str, &'static str); 20] = [
        ("C-1", 3, "TEV-961 Case 6 streams", "INL TEV-961 Rev. 0 2010",
         "The local PDF ends at A-59, the end of Case 5. The Case 6 calculator pages and stream appendices are absent", SAME),
        ("W-1", 2, "LTS kinetics", "Library search",
         "No Cu/Zn kinetic source is in the library. WGS-LTS is a 10 K approach assumption", INDEPENDENT),
        ("W-2", 2, "Chinchen effectiveness, Tables 3 and 5", "Chinchen et al. 1984, Appl. Catal. 12, 69-88",
         "The engine has no pore diffusion model and none is planned. Chinchen Part III is not local", INDEPENDENT),
        ("Z-1", 1, "Zhang e-NRTL reproduction", "Zhang, Que and Chen 2011, Fluid Phase Equilib. 311, 67-75",
         "Parameters depend on the Aspen Databank and the activity equations are unprinted. The paper gives no numerical model output to compare with", SAME),
        ("Z-2", 1, "High pressure VLE from Zhang Figs 9 and 10", "Zhang, Que and Chen 2011, Fluid Phase Equilib. 311, 67-75",
         "Those figures are raster, so the points cannot be read reliably, and Jou's primary paper is not local", INDEPENDENT),
        ("X-1", 2, "Xu-Froment rates against their experiments", "Xu and Froment 1989 Part I",
         "The papers show rates only in figures, and K1(T), K2(T), K3(T) appear in neither paper", INDEPENDENT),
        ("H-1", 1, "Hartono correlations", "Hartono et al. 2014, J. Chem. Eng. Data 59, 1808-1816",
         "Parameters k1 to k4 are in Hartono 2013, which is not local. Pure component properties are unprinted, and supplement Tables S1 and S2 are not local", INDEPENDENT),
        ("H-2", 1, "Properties above loading 0.50", "Hartono et al. 2014, J. Chem. Eng. Data 59, 1808-1816",
         "Hartono stops at 0.50. No local source reaches the 0.50 to 0.75 region", INDEPENDENT),
        ("H-3", 1, "H2, CH4, CO solubility in MEA", "Library search",
         "No local source covers them. FLASH treats them as insoluble", INDEPENDENT),
        ("VER-X", 1, "Versteeg Henry constants", "Versteeg and van Swaaij 1988, J. Chem. Eng. Data 33, 29-34",
         "That paper is not local. The local paper is the kinetics paper", INDEPENDENT),
        ("I-1", 2, "Isa rich loading", "Isa et al. 2017, Int. J. Greenh. Gas Control 56, 173-186",
         "Table 9 lists 0.056 to 0.195, while Table 3 flows and about 95% removal imply about 0.31. The table fails its own mass balance", INDEPENDENT),
        ("I-2", 2, "Isa Fig 10 and Table 8 areas", "Isa et al. 2017, Int. J. Greenh. Gas Control 56, 173-186",
         "Fig 10 is raster. Table 8 prints no operating condition for its single area value", INDEPENDENT),
        ("A-1", 2, "Abdul Halim removal and KGav", "Abdul Halim et al. 2015, Ind. Eng. Chem. Res. 54, 1675-1680",
         "Figures 3 to 6 are raster", INDEPENDENT),
        ("S-1", 2, "SRD at high pressure", "Library search",
         "Isa, Abdul Halim and Salvinder all run once through. None measured regeneration", INDEPENDENT),
        ("S-2", 2, "30 wt% MEA at high pressure", "Library search",
         "Isa and Salvinder use 20 wt%, and Abdul Halim stops at 4 kmol/m3", INDEPENDENT),
        ("S-3", 2, "MEA on H2-rich syngas", "Library search",
         "No local absorber study uses H2-rich gas. The absorber carries ReducedOrder status at plant conditions", INDEPENDENT),
        ("V-1", 2, "Split flow and interheating", "Vinjarapu et al. 2024 and 2025",
         "Both need a stripper configuration the plant flowsheet does not contain", INDEPENDENT),
        ("HT-1", 2, "HTTR heat utilisation performance", "IAEA-TECDOC-1236, 2001",
         "TECDOC-1236 gives design intent for the coupled reformer and prints no measured data from it", INDEPENDENT),
        ("HE-2-MU", 1, "Helium viscosity at 700, 900, 1100 and 1200 K and 50 bar against an independent source", "Arp, McCarty and Friend 1998, NIST Technical Note 1334",
         "TN-1334 is not in the local library and CoolProp is not installed, so no independent value is held. Limit frozen at twice Petersen's stated standard deviation, 0.003 T percent (3.5% at 1173 K)", INDEPENDENT),
        ("HE-2-K", 1, "Helium conductivity at 700, 900, 1100 and 1200 K and 50 bar against an independent source", "Arp, McCarty and Friend 1998, NIST Technical Note 1334",
         "TN-1334 is not in the local library and CoolProp is not installed, so no independent value is held. Limit frozen at twice Petersen's stated standard deviation, 0.007 T percent (8.2% at 1173 K)", INDEPENDENT),
    ];
    for (id, level, subject, source, reason, data_use) in rows {
        out.push(
            Meta::new(id, level, subject, source, loc)
                .use_(data_use)
                .not_run(None, reason),
        );
    }
}
