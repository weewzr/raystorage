import sys
sys.path.insert(0, r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/engine_final")
from e1lib import rep
F = "engine/plant/src/sens.rs"
old_start = "/// MATURE_60Y, the author's stacked scenario: tenth of a kind on INL's BOAK"
old_end = "/// Gas price of the second MATURE_60Y point, S$/GJ, the author's chosen value.\npub const MATURE_GAS_POINT: f64 = 20.0;\n"
from pathlib import Path
p = Path(F)
s = p.read_bytes().decode("utf-8")
assert "\r\n" not in s
i = s.index(old_start)
j = s.index(old_end) + len(old_end)
new = '''/// Gas price of the second MATURE_60Y point, S$/GJ, the author's chosen value.
pub const MATURE_GAS_POINT: f64 = 20.0;

/// Gas price the documents print for the mature scenario: the larger of the
/// L5 and DCF gas solves, rounded up to S$0.1/GJ, so both methods sit at or
/// below the target there.
pub fn printed_gas(l5: f64, dcf: f64) -> f64 {
    (l5.max(dcf) * 10.0).ceil() / 10.0
}

/// The plant, its abatement and the cost of one method at one query, for the
/// MATURE_60Y and step-down rows.
struct CostRun {
    m: Model,
    b: crate::anchored::Balance,
    a: f64,
}

impl CostRun {
    fn new(base: &ParamSet) -> Result<Self> {
        let m = Model::new(base)?;
        let b = m.balance(Scenario::RecF, &m.opts(m.s_b))?;
        let a = m.emissions(&b.c, m.s_b).a_atm;
        Ok(Self { m, b, a })
    }

    fn cost(&self, method: Method, q: CostQuery) -> Result<f64> {
        Ok(match method {
            Method::L5 => self.m.ledger(&self.b, &q)?.l5() * 1e6 / self.a,
            Method::Dcf => self.m.dcf(&self.b, &q)?.lac,
        })
    }

    /// The MATURE_60Y query: 10OAK_P46, 60 y, n = 1.0, the rated reactor.
    fn mature(&self) -> CostQuery {
        let m = &self.m;
        CostQuery::central(Stage::Oak10FromBoak, m.lives[2], m.n_linear, m.r_rated)
    }

    /// Gas price that brings a method to the mature-scenario target.
    fn gas_needed(&self, method: Method) -> Result<f64> {
        let q0 = self.mature();
        solve_for(
            |g| self.cost(method, CostQuery { gas: Some(g), ..q0 }),
            self.m.gas.low,
            self.m.gas.high,
            self.m.target_mature_c,
        )
    }

    /// The printed gas price of `printed_gas` on the two solves.
    fn gas_printed(&self) -> Result<f64> {
        Ok(printed_gas(
            self.gas_needed(Method::L5)?,
            self.gas_needed(Method::Dcf)?,
        ))
    }
}

/// MATURE_60Y, the author's stacked scenario: tenth of a kind on INL's BOAK
/// learning route (INL PDF p46), a 60 y life, n = 1.0 and the rated reactor
/// charged. Each method gives its cost at the central gas price, at S$20/GJ
/// and at the printed price, and the gas price that brings it to the
/// mature-scenario target. Run MATURE_60Y carries the printed price itself.
///
/// eq: L.31
pub fn mature_rows(base: &ParamSet) -> Result<Vec<SensRow>> {
    let c = CostRun::new(base)?;
    let m = &c.m;
    let target = m.target_mature_c;
    let q0 = c.mature();
    let x = c.gas_printed()?;
    let note = "stacked scenario MATURE_60Y: 10OAK_P46 stage, 60 y life, n = 1.0, reactor \\
                charged at the 210 MWth rating, Case B REC_F; the base result stays BOAK 25 y. \\
                Higher gas lowers the abatement cost, because NUS-H2 burns less gas than the \\
                unabated plant of equal H2 output. The printed gas price is the larger of the \\
                L5 and DCF solves rounded up to S$0.1/GJ";
    let mut rows = Vec::new();
    let mut push = |run: &str, parameter, unit, base_value: f64, test_value, metric, v: f64| {
        rows.push(SensRow {
            sweep_id: "MATURE_60Y",
            run_id: run.to_string(),
            parameter,
            unit,
            base_value,
            test_value,
            metric,
            result: v,
            delta: v - base_value,
            note: note.to_string(),
            status: None,
        });
    };
    for (method, mid) in [(Method::L5, "L5"), (Method::Dcf, "DCF")] {
        let run = format!("MATURE_60Y_{mid}");
        let at = |g: f64| c.cost(method, CostQuery { gas: Some(g), ..q0 });
        push(
            &run,
            "Cost at the central gas price, against the mature-scenario target",
            "S$/t",
            target,
            m.gas.central,
            "COST_S_PER_T",
            c.cost(method, q0)?,
        );
        push(
            &run,
            "Cost with gas at S$20/GJ, against the mature-scenario target",
            "S$/t",
            target,
            MATURE_GAS_POINT,
            "COST_S_PER_T",
            at(MATURE_GAS_POINT)?,
        );
        push(
            &run,
            "Cost with gas at the printed price, the larger gas solve rounded up to S$0.1/GJ, \\
             against the mature-scenario target",
            "S$/t",
            target,
            x,
            "COST_S_PER_T",
            at(x)?,
        );
        push(
            &run,
            "Gas price that brings the cost to the mature-scenario target",
            "S$/GJ",
            m.gas.central,
            target,
            "GAS_NEEDED",
            c.gas_needed(method)?,
        );
    }
    push(
        "MATURE_60Y",
        "Printed gas price: the larger of the L5 and DCF solves rounded up to S$0.1/GJ",
        "S$/GJ",
        m.gas.central,
        target,
        "GAS_PRINTED",
        x,
    );
    Ok(rows)
}

/// The route 4 step-down from the base full cost to the mature scenario, one
/// bar per step on each method: BOAK 25 y at the central gas price, then tenth
/// of a kind (10OAK_P46) 25 y, then 10OAK_P46 60 y, then gas at the printed
/// price. n = 1.0 and the rated reactor charged throughout. Each bar's base
/// value is the bar before it, so the delta is the step.
///
/// eq: L.31
pub fn stepdown_rows(base: &ParamSet) -> Result<Vec<SensRow>> {
    let c = CostRun::new(base)?;
    let m = &c.m;
    let x = c.gas_printed()?;
    let steps: [(&str, &'static str, Stage, f64, Option<f64>); 4] = [
        (
            "STEPDOWN_1_BOAK_25Y",
            "Step 1, base full cost: BOAK, 25 y, gas at the central price",
            Stage::Boak,
            m.lives[0],
            None,
        ),
        (
            "STEPDOWN_2_10OAK_P46_25Y",
            "Step 2, tenth of a kind on INL's BOAK learning route (PDF p46), 25 y",
            Stage::Oak10FromBoak,
            m.lives[0],
            None,
        ),
        (
            "STEPDOWN_3_10OAK_P46_60Y",
            "Step 3, tenth of a kind, 60 y life",
            Stage::Oak10FromBoak,
            m.lives[2],
            None,
        ),
        (
            "STEPDOWN_4_GAS_PRINTED",
            "Step 4, tenth of a kind, 60 y, gas at the printed MATURE_60Y price",
            Stage::Oak10FromBoak,
            m.lives[2],
            Some(x),
        ),
    ];
    let note = "route 4 step-down from the base full cost to the MATURE_60Y point, Case B \\
                REC_F, n = 1.0, reactor charged at the 210 MWth rating; base_value is the bar \\
                before, test_value the gas price of the bar, S$/GJ";
    let mut rows = Vec::new();
    for (method, metric) in [(Method::L5, "AC_L5"), (Method::Dcf, "LAC_DCF")] {
        let mut prev: Option<f64> = None;
        for (run, parameter, stage, life, gas) in steps {
            let q = CostQuery {
                gas,
                ..CostQuery::central(stage, life, m.n_linear, m.r_rated)
            };
            let v = c.cost(method, q)?;
            let before = prev.unwrap_or(v);
            rows.push(SensRow {
                sweep_id: "STEPDOWN",
                run_id: run.to_string(),
                parameter,
                unit: "S$/t",
                base_value: before,
                test_value: gas.unwrap_or(m.gas.central),
                metric,
                result: v,
                delta: v - before,
                note: note.to_string(),
                status: None,
            });
            prev = Some(v);
        }
    }
    Ok(rows)
}
'''
s = s[:i] + new + s[j:]
s = s.replace("""    rows.extend(mature_rows(base)?);
    rows.extend(cofunding_rows(base)?);""", """    rows.extend(mature_rows(base)?);
    rows.extend(stepdown_rows(base)?);
    rows.extend(cofunding_rows(base)?);""", 1)
p.write_bytes(s.encode("utf-8"))
print("ok")
