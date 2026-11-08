
// ------------------------------------------------------- the exported table

type Row = std::collections::BTreeMap<String, String>;

fn table() -> Vec<Row> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../results/Subsidy_Scenarios.csv"
    );
    let text = std::fs::read_to_string(path).unwrap();
    let mut lines = text.lines();
    let header = split_csv(lines.next().unwrap());
    lines
        .map(|l| {
            let cells = split_csv(l);
            assert_eq!(cells.len(), header.len(), "{l}");
            header.iter().cloned().zip(cells).collect()
        })
        .collect()
}

#[test]
fn eq_s_8_the_committed_table_matches_the_engine() {
    let c = ctx();
    let rows = table();
    // 4 stages, 2 lives, 2 eligible sets, 2 timings, each with 4 grid shares for
    // 2 methods and 3 break-even rows.
    assert_eq!(rows.len(), 4 * 2 * 2 * 2 * (4 * 2 + 3));
    let f = |r: &Row, k: &str| -> f64 { r[k].parse().unwrap() };
    let (mut grid, mut breakeven) = (0, 0);
    for r in &rows {
        let stage = Stage::ALL
            .into_iter()
            .find(|s| s.id() == r["stage"])
            .unwrap();
        let life = f(r, "life_y");
        let d = if r["definition"] == "A" {
            Definition::A
        } else {
            Definition::B
        };
        let t = if r["timing"] == "PROGRESSIVE" {
            Timing::Progressive
        } else {
            Timing::AtClose
        };
        assert_eq!(r["run_id"], "B_REC_F");
        assert_eq!(r["n"], "1.0000");
        assert_eq!(r["reactor_charged_mwth"], "210.00");
        assert_eq!(r["class"], "calculated");
        assert!(r["status"] == "anchored_port" || r["status"].starts_with("gated_"));
        assert_eq!(r["default_case"], (d == Definition::A && t == Timing::Progressive).to_string());
        let q = query(c, stage, life);
        let share = f(r, "share");
        let e = c.m.subsidy_eval(&c.b, &q, gq(share, d, t), c.a).unwrap();
        let cells = match r["method"].as_str() {
            "L5" => Some(&e.l5),
            "DCF" => Some(&e.dcf),
            "BOTH" => None,
            other => panic!("method {other}"),
        };
        let tol = 1.0e-6;
        close(f(r, "total_capital_sgdm"), e.plan.total_capital, tol, "total capital");
        close(f(r, "eligible_capital_sgdm"), e.plan.eligible, tol, "eligible capital");
        close(f(r, "grant_sgdm"), e.plan.grant, tol, "grant");
        close(f(r, "private_capital_sgdm"), e.plan.private_capital, tol, "private capital");
        close(f(r, "abatement_t_y"), c.a, tol, "abatement");
        close(f(r, "target_s_per_t"), c.m.target_c, tol, "target");
        match cells {
            Some(m) => {
                close(f(r, "cost_net_s_per_t"), m.lac_net, tol, "net cost");
                close(f(r, "cost_gross_s_per_t"), m.lac_gross, tol, "gross cost");
                close(f(r, "annual_cost_net_sgdm_y"), m.cost_net_sm_y, tol, "net annual cost");
                close(f(r, "annual_cost_gross_sgdm_y"), m.cost_gross_sm_y, tol, "gross annual cost");
                close(f(r, "interest_gross_sgdm"), m.interest_gross_sm, tol, "gross interest");
                close(f(r, "interest_private_sgdm"), m.interest_private_sm, tol, "private interest");
                close(f(r, "interest_saved_sgdm"), m.interest_saved_sm, tol, "interest saved");
                close(f(r, "public_outlay_s_per_t"), m.public_outlay_s_per_t, tol, "outlay");
                close(f(r, "state_pv_sgdm"), m.state_pv_sm, tol, "state PV");
                assert_eq!(r["target_met"], m.target_met.to_string());
                assert_eq!(r["gap_to_l5_s_per_t"].is_empty(), r["method"] == "L5");
                if r["method"] == "DCF" {
                    close(f(r, "gap_to_l5_s_per_t"), e.gap_s_per_t, tol, "gap");
                }
            }
            None => {
                assert_eq!(r["target_met"], e.target_met.to_string());
                for k in ["cost_net_s_per_t", "annual_cost_net_sgdm_y", "public_outlay_s_per_t"] {
                    assert!(r[k].is_empty(), "{k} on a row for both methods");
                }
            }
        }
        match r["row_kind"].as_str() {
            "grid" => {
                grid += 1;
                assert!(c.m.subsidy.grid.contains(&share));
                assert!(r["breakeven_case"].is_empty());
                assert_ne!(r["method"], "BOTH");
            }
            "breakeven" => {
                breakeven += 1;
                let be = c.m.breakeven(&c.b, &q, d, t, c.a).unwrap();
                let result = match r["method"].as_str() {
                    "L5" => be.l5.result,
                    "DCF" => be.dcf.result,
                    _ => be.both,
                };
                match (r["breakeven_case"].as_str(), result) {
                    ("share", BreakEven::Share(s)) => {
                        // Rounded up in the sixth decimal, and passing.
                        assert!(share >= s - 1e-12 && share - s < 1e-5, "{share} against {s}");
                        assert_eq!(r["target_met"], "true");
                    }
                    ("met_without_grant", BreakEven::MetWithoutGrant) => {
                        assert_eq!(share, 0.0);
                        assert_eq!(r["target_met"], "true");
                    }
                    ("unreachable", BreakEven::Unreachable { .. }) => {
                        assert_eq!(share, 1.0);
                        assert_eq!(r["target_met"], "false");
                    }
                    (case, other) => panic!("case {case} against {other:?}"),
                }
            }
            other => panic!("row kind {other}"),
        }
    }
    assert_eq!((grid, breakeven), (4 * 2 * 2 * 2 * 8, 4 * 2 * 2 * 2 * 3));
}

#[test]
fn eq_s_8_the_committed_table_reproduces_the_economic_ledger_at_a_zero_share() {
    let rows = table();
    let ledger = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../results/Economic_Ledger.csv"
    ))
    .unwrap();
    let ledger: Vec<Vec<String>> = ledger.lines().skip(1).map(split_csv).collect();
    let mut n = 0;
    for r in rows.iter().filter(|r| r["row_kind"] == "grid" && r["share"] == "0.000000") {
        let want = if r["method"] == "L5" {
            ledger
                .iter()
                .filter(|l| {
                    l[0] == "B_REC_F"
                        && l[1] == r["stage"]
                        && l[2] == r["life_y"]
                        && l[3] == "1.0000"
                        && l[4] == "210.00"
                        && l[5] == "L5"
                })
                .next_back()
                .unwrap()[12]
                .parse::<f64>()
                .unwrap()
        } else {
            ledger
                .iter()
                .find(|l| {
                    l[0] == "B_REC_F"
                        && l[1] == r["stage"]
                        && l[2] == r["life_y"]
                        && l[3] == "1.0000"
                        && l[4] == "210.00"
                        && l[6] == "LAC_DCF"
                })
                .unwrap()[18]
                .parse::<f64>()
                .unwrap()
        };
        for k in ["cost_gross_s_per_t", "cost_net_s_per_t"] {
            let v: f64 = r[k].parse().unwrap();
            close(v, want, 1e-6, "zero-share cost against the economic ledger");
        }
        assert_eq!(r["grant_sgdm"], "0.000000");
        n += 1;
    }
    assert_eq!(n, 4 * 2 * 2 * 2 * 2);
}
