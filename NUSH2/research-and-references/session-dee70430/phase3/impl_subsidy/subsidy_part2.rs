
// ------------------------------------------------------------ closed forms

/// Present-value factor of a payment dated `year`, negative before year 0.
fn growth(rate: f64, year: i32) -> f64 {
    (1.0 + rate).powi(-year)
}

/// DCF bucket factor: the year-0 value of the reactor spend per unit of
/// overnight capital when the spend sits in the year-end buckets of
/// `dcf::uniform_spend`, a part year first.
///
/// eq: S.0.1
pub fn bucket_factor(rate: f64, years: f64) -> Result<f64> {
    if !(rate > 0.0 && years > 0.0) {
        return Err(EngineError::Range {
            param: "bucket_factor".into(),
            msg: format!("rate {rate} and construction time {years} must both be positive"),
        });
    }
    let whole = years.floor();
    let part = years - whole;
    let g = (1.0 + rate).powf(whole);
    Ok((part * g + (g - 1.0) / rate) / years)
}

/// The ledger's private present-value factor of a grant paid at close, per unit
/// of overnight capital, when the grant covers the fraction `u` of the reactor
/// spend.
///
/// eq: S.3c
pub fn private_factor_at_close(rate: f64, years: f64, u: f64) -> f64 {
    ((1.0 + rate).powf(years * (1.0 - u)) - 1.0) / (rate * years)
}

/// One bucket of the funded-balance recursion of S.3a.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FundedRow {
    pub year: i32,
    /// Gross reactor spend of the bucket, S$M.
    pub bucket: f64,
    /// Paid by the grant, S$M.
    pub grant: f64,
    /// Paid by the owner, S$M.
    pub private: f64,
    /// Owner's funded balance after the bucket: cash paid in plus the interest
    /// accrued on it, S$M.
    pub balance: f64,
}

/// The owner's account when a grant of `grant` S$M arrives at financial close.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Funded {
    pub rows: Vec<FundedRow>,
    /// Balance carried to year 0: the present value of the private payments.
    pub balance_at_0: f64,
}

/// Apply a grant paid at close to the dated reactor buckets, earliest first,
/// and run the owner's balance forward. Idle grant cash earns nothing.
///
/// eq: S.3a
pub fn funded_balance(rate: f64, buckets: &[(i32, f64)], grant: f64) -> Funded {
    let mut rows = Vec::with_capacity(buckets.len());
    let mut spent = 0.0;
    let mut balance = 0.0;
    let mut last_year: Option<i32> = None;
    for &(year, bucket) in buckets {
        if let Some(y) = last_year {
            balance *= (1.0 + rate).powi(year - y);
        }
        let paid_by_grant = bucket.min((grant - spent).max(0.0));
        let private = bucket - paid_by_grant;
        balance += private;
        spent += bucket;
        last_year = Some(year);
        rows.push(FundedRow {
            year,
            bucket,
            grant: paid_by_grant,
            private,
            balance,
        });
    }
    let balance_at_0 = match last_year {
        Some(y) => balance * (1.0 + rate).powi(-y),
        None => 0.0,
    };
    Funded {
        rows,
        balance_at_0,
    }
}

/// Closed form of the owner's present value of the reactor spend after a grant
/// of `grant` S$M at close, without a loop.
///
/// eq: S.3b
pub fn private_pv_closed_form(rate: f64, years: f64, reactor: f64, grant: f64) -> f64 {
    let whole = years.floor();
    let part = years - whole;
    let gamma = grant * years / reactor;
    if gamma >= years {
        return 0.0;
    }
    let (n_r, kept) = if gamma <= part {
        (whole, part - gamma)
    } else {
        let j = (gamma - part).floor().min(whole - 1.0);
        (whole - 1.0 - j, 1.0 - (gamma - part - j))
    };
    let g = (1.0 + rate).powf(n_r);
    reactor / years * (kept * g + (g - 1.0) / rate)
}

// ------------------------------------------------------------- grant plan

/// A grant worked out against one set of capital items. Amounts are S$M at
/// year 0 unless stated. The ledger side and the DCF side differ only in how
/// interest during construction is measured (S.0.2).
///
/// eq: S.8
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GrantPlan {
    pub share: f64,
    pub definition: Definition,
    pub timing: Timing,
    /// Reactor overnight capital, `CostItems::reactor_capital`.
    pub reactor_capital: f64,
    /// Eligible capital other than the reactor.
    pub other_eligible: f64,
    /// Eligible capital, S.1.
    pub eligible: f64,
    /// The grant, `share * eligible`, S.2.
    pub grant: f64,
    /// First-unit capital of the incremental project, S.8.
    pub total_capital: f64,
    /// `total_capital - grant`.
    pub private_capital: f64,
    /// Part of the grant that pays reactor spend.
    pub grant_reactor: f64,
    /// Part of the grant that pays items bought at year 0.
    pub grant_year0: f64,
    /// Ledger interest factor `f_IDC(t_c)`.
    pub ledger_factor: f64,
    /// DCF bucket factor, the gross reactor value per unit of capital.
    pub dcf_factor: f64,
    /// Interest during construction on the gross spend, ledger form.
    pub ledger_interest_gross: f64,
    /// Interest on the private balance, ledger form.
    pub ledger_interest_private: f64,
    /// Interest the grant saves, ledger form.
    pub ledger_interest_saved: f64,
    /// Year-0 value of the grant to the state, ledger form.
    pub ledger_state_pv: f64,
    pub dcf_interest_gross: f64,
    pub dcf_interest_private: f64,
    pub dcf_interest_saved: f64,
    /// Present value of the private reactor payments in the DCF.
    pub dcf_private_reactor_pv: f64,
    /// Present value of the GRANT line of the DCF, as a positive amount.
    pub dcf_grant_pv: f64,
    pub dcf_state_pv: f64,
    /// Negative flows of the GRANT line at whole years.
    pub dcf_flows: Vec<(i32, f64)>,
}

impl Model {
    /// Work out the grant against the capital items of one query.
    ///
    /// eq: S.1
    /// eq: S.2
    /// eq: S.3a
    /// eq: S.3c
    /// eq: S.4a
    /// eq: S.4b
    /// eq: S.5a
    /// eq: S.8
    pub fn grant_plan(&self, k: &CostItems, g: &GrantQuery) -> Result<GrantPlan> {
        let sp = &self.subsidy;
        if !(0.0..=sp.share_max).contains(&g.share) {
            return Err(EngineError::Range {
                param: "SUBSIDY_SHARE".into(),
                msg: format!("{} lies outside 0 to {}", g.share, sp.share_max),
            });
        }
        let c_r = k.reactor_capital;
        if !(c_r > 0.0 && c_r.is_finite()) {
            return Err(EngineError::Range {
                param: "reactor_capital".into(),
                msg: format!("{c_r} must be positive"),
            });
        }
        let el = sp.eligibility(g.definition);
        let other_eligible: f64 = [
            (el.ihx, k.ihx_capital),
            (el.licence, k.licence_capital),
            (el.power_island, k.power_island_capital),
            (el.capture, k.capture_capital),
            (el.recycle, k.recycle_capital),
        ]
        .iter()
        .filter(|(on, _)| *on)
        .map(|(_, v)| *v)
        .sum();
        let eligible = c_r + other_eligible;
        let s = g.share;
        let grant = s * eligible;
        let total_capital = c_r
            + k.capture_capital
            + k.ihx_capital
            + k.licence_capital
            + k.power_island_capital
            + k.recycle_capital;
        let rate = self.rate;
        let tc = self.idc_years;
        let f_idc = self.idc_factor();
        let buckets = uniform_spend(c_r, tc)?;
        let gross_pv: f64 = buckets.iter().map(|(y, v)| v * growth(rate, *y)).sum();
        let ledger_interest_gross = c_r * (f_idc - 1.0);
        let dcf_interest_gross = gross_pv - c_r;
        let (grant_reactor, grant_year0) = match g.timing {
            Timing::Progressive => (s * c_r, s * other_eligible),
            Timing::AtClose => (grant.min(c_r), (grant - c_r).max(0.0)),
        };
        // Ledger side: S.4b for a progressive grant, S.3c at close.
        let (ledger_interest_private, ledger_state_pv) = match g.timing {
            Timing::Progressive => {
                let private = (1.0 - s) * ledger_interest_gross;
                (private, grant + (ledger_interest_gross - private))
            }
            Timing::AtClose => {
                let u = grant_reactor / c_r;
                let h = private_factor_at_close(rate, tc, u);
                (c_r * (h - (1.0 - u)), grant * (1.0 + rate).powf(tc))
            }
        };
        // DCF side: S.4a for a progressive grant, S.3a at close.
        let (mut flows, dcf_private_reactor_pv) = match g.timing {
            Timing::Progressive => (
                buckets.iter().map(|(y, b)| (*y, -s * b)).collect::<Vec<_>>(),
                buckets
                    .iter()
                    .map(|(y, b)| (1.0 - s) * b * growth(rate, *y))
                    .sum::<f64>(),
            ),
            Timing::AtClose => {
                let f = funded_balance(rate, &buckets, grant_reactor);
                (
                    f.rows.iter().map(|r| (r.year, -r.grant)).collect(),
                    f.balance_at_0,
                )
            }
        };
        if grant_year0 > 0.0 {
            match flows.last_mut() {
                Some(last) if last.0 == 0 => last.1 -= grant_year0,
                _ => flows.push((0, -grant_year0)),
            }
        }
        let dcf_grant_pv: f64 = flows.iter().map(|(y, v)| -v * growth(rate, *y)).sum();
        let dcf_interest_private = dcf_private_reactor_pv - (c_r - grant_reactor);
        let dcf_state_pv = match g.timing {
            Timing::Progressive => dcf_grant_pv,
            Timing::AtClose => grant * (1.0 + rate).powf(tc),
        };
        Ok(GrantPlan {
            share: s,
            definition: g.definition,
            timing: g.timing,
            reactor_capital: c_r,
            other_eligible,
            eligible,
            grant,
            total_capital,
            private_capital: total_capital - grant,
            grant_reactor,
            grant_year0,
            ledger_factor: f_idc,
            dcf_factor: gross_pv / c_r,
            ledger_interest_gross,
            ledger_interest_private,
            ledger_interest_saved: ledger_interest_gross - ledger_interest_private,
            ledger_state_pv,
            dcf_interest_gross,
            dcf_interest_private,
            dcf_interest_saved: dcf_interest_gross - dcf_interest_private,
            dcf_private_reactor_pv,
            dcf_grant_pv,
            dcf_state_pv,
            dcf_flows: flows,
        })
    }
}

// ------------------------------------------------------- reported quantities

/// The quantities of S.8 for one method at one grant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct MethodEval {
    /// Annual incremental cost with the grant ignored, S$M/y. For the DCF this
    /// is the levelised cost per tonne times the abatement.
    pub cost_gross_sm_y: f64,
    /// The same net of the grant.
    pub cost_net_sm_y: f64,
    pub lac_gross: f64,
    pub lac_net: f64,
    pub interest_gross_sm: f64,
    pub interest_private_sm: f64,
    pub interest_saved_sm: f64,
    /// Year-0 value of the grant to the state, S$M.
    pub state_pv_sm: f64,
    /// The state's outlay levelised over the abated tonnes, S$/t.
    pub public_outlay_s_per_t: f64,
    /// Strict: the net cost is below the target.
    pub target_met: bool,
}

/// Both methods at one grant, with the plan behind them.
///
/// eq: S.8
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SubsidyEval {
    pub plan: GrantPlan,
    pub abatement_t_y: f64,
    pub life: f64,
    pub target: f64,
    pub l5: MethodEval,
    pub dcf: MethodEval,
    /// Both methods meet the target (decision D3).
    pub target_met: bool,
    /// DCF less L5 at the net cost, S$/t.
    pub gap_s_per_t: f64,
    /// That gap over the L5 net cost.
    pub gap_rel: f64,
}

impl SubsidyEval {
    /// The ledger-to-cash-flow gap stays within the registry limit (decision D11).
    pub fn gap_within(&self, limit: f64) -> bool {
        self.gap_rel.abs() <= limit
    }
}

/// Where a break-even share stands against the target.
///
/// eq: S.7e
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum BreakEven {
    /// The cost with no grant is already below the target. The share is 0.
    MetWithoutGrant,
    /// The infimum of the shares that meet the target. The cost at this share
    /// equals the target, so only larger shares pass.
    Share(f64),
    /// Even the largest share leaves the cost at or above the target.
    Unreachable { lac_at_max: f64 },
}

impl BreakEven {
    /// The share to print, `None` when no share in range meets the target.
    pub fn share(&self) -> Option<f64> {
        match self {
            BreakEven::MetWithoutGrant => Some(0.0),
            BreakEven::Share(s) => Some(*s),
            BreakEven::Unreachable { .. } => None,
        }
    }
}

/// How a break-even share was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SolveRoute {
    /// No root was needed: met without a grant, or unreachable.
    NotNeeded,
    /// The closed form of S.7b to S.7d passed the residual check.
    ClosedForm,
    /// The closed form failed the residual check and the bracketed bisection
    /// of S.7d ran.
    Bisection,
}

/// A break-even share with the check that stands behind it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BreakEvenSolve {
    pub result: BreakEven,
    pub route: SolveRoute,
    /// Cost at the returned share less the target, S$/t. Within the registry
    /// tolerance on the closed-form route.
    pub residual: f64,
}

/// The break-even shares of one scenario, per method and for both together.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BreakEvenSet {
    pub l5: BreakEvenSolve,
    pub dcf: BreakEvenSolve,
    /// Both methods meet the target: the larger share.
    pub both: BreakEven,
}

/// The larger of two break-even results, or the unreachable one.
///
/// eq: S.7e
pub fn combine(a: BreakEven, b: BreakEven) -> BreakEven {
    use BreakEven::{MetWithoutGrant, Share, Unreachable};
    match (a, b) {
        (Unreachable { lac_at_max: x }, Unreachable { lac_at_max: y }) => Unreachable {
            lac_at_max: x.max(y),
        },
        (Unreachable { lac_at_max: x }, _) | (_, Unreachable { lac_at_max: x }) => {
            Unreachable { lac_at_max: x }
        }
        (MetWithoutGrant, MetWithoutGrant) => MetWithoutGrant,
        (MetWithoutGrant, Share(s)) | (Share(s), MetWithoutGrant) => Share(s),
        (Share(x), Share(y)) => Share(x.max(y)),
    }
}

/// Round a share up in the last of `decimals` printed digits, so the printed
/// share lies on the passing side of the infimum.
///
/// eq: S.7e
pub fn round_up_share(s: f64, decimals: i32) -> f64 {
    let scale = 10f64.powi(decimals);
    (s * scale).ceil() / scale
}

/// Bracketed bisection for the share at which a falling cost crosses the
/// target. `lac(lo) >= target > lac(hi)` must hold. Returns the upper end of
/// the final bracket, which always passes the strict test.
///
/// eq: S.7d
pub fn bisect_share<F>(mut lac: F, target: f64, lo: f64, hi: f64, tol: f64) -> Result<f64>
where
    F: FnMut(f64) -> Result<f64>,
{
    let (mut lo, mut hi) = (lo, hi);
    if !(lac(lo)? >= target && lac(hi)? < target) {
        return Err(EngineError::Range {
            param: "bisect_share".into(),
            msg: format!("the target {target} is not bracketed by shares {lo} and {hi}"),
        });
    }
    while hi - lo > tol {
        let mid = 0.5 * (lo + hi);
        if lac(mid)? < target {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Ok(hi)
}

/// What the closed forms of S.7b to S.7d need about one scenario.
struct Curve {
    rate: f64,
    years: f64,
    /// Levelising factor, S$/t per S$M.
    kappa: f64,
    plan: GrantPlan,
    target: f64,
}

impl Curve {
    /// Break-even share of the ledger cost. Negative when the target is already
    /// met.
    ///
    /// eq: S.7b
    /// eq: S.7c
    fn ledger(&self, lac0: f64, timing: Timing) -> f64 {
        let p = &self.plan;
        match timing {
            Timing::Progressive => {
                let slope = self.kappa * (p.reactor_capital * p.ledger_factor + p.other_eligible);
                (lac0 - self.target) / slope
            }
            Timing::AtClose => {
                let d0 = (lac0 - self.target) / self.kappa;
                let h_star = p.ledger_factor - d0 / p.reactor_capital;
                if h_star >= 0.0 {
                    let ln1 = (1.0 + self.rate).ln();
                    let u_star = 1.0
                        - (1.0 + self.rate * self.years * h_star).ln() / (self.years * ln1);
                    u_star * p.reactor_capital / p.eligible
                } else {
                    (d0 - p.reactor_capital * p.ledger_factor + p.reactor_capital) / p.eligible
                }
            }
        }
    }

    /// Cost of the DCF at close as a function of the share, by S.6b with the
    /// closed form of S.3b.
    ///
    /// eq: S.6b
    fn dcf_close(&self, lac0: f64, s: f64) -> f64 {
        let p = &self.plan;
        let grant = s * p.eligible;
        let reactor = grant.min(p.reactor_capital);
        let pd = private_pv_closed_form(self.rate, self.years, p.reactor_capital, reactor);
        let removed = p.reactor_capital * p.dcf_factor - pd + (grant - p.reactor_capital).max(0.0);
        lac0 - self.kappa * removed
    }

    /// Break-even share of the DCF cost. Progressive: S.7b. At close: the exact
    /// route of S.7d, a straight line between the shares at which the grant
    /// clears a bucket.
    ///
    /// eq: S.7b
    /// eq: S.7d
    fn dcf(&self, lac0: f64, timing: Timing, share_max: f64) -> Result<f64> {
        let p = &self.plan;
        match timing {
            Timing::Progressive => {
                let slope = self.kappa * (p.reactor_capital * p.dcf_factor + p.other_eligible);
                Ok((lac0 - self.target) / slope)
            }
            Timing::AtClose => {
                let buckets = uniform_spend(p.reactor_capital, self.years)?;
                let mut knots = vec![0.0];
                let mut cum = 0.0;
                for (_, b) in &buckets {
                    cum += b;
                    knots.push(cum / p.eligible);
                }
                knots.push(share_max);
                knots.retain(|s| *s <= share_max);
                knots.sort_by(f64::total_cmp);
                knots.dedup_by(|a, b| (*a - *b).abs() < 1e-15);
                let mut prev = (knots[0], self.dcf_close(lac0, knots[0]));
                if prev.1 < self.target {
                    return Ok(-1.0);
                }
                for &s in &knots[1..] {
                    let v = self.dcf_close(lac0, s);
                    if v < self.target {
                        return Ok(prev.0 + (prev.1 - self.target) / (prev.1 - v) * (s - prev.0));
                    }
                    prev = (s, v);
                }
                Ok(f64::INFINITY)
            }
        }
    }
}

impl Model {
    /// Evaluate both methods at one grant against the registry target.
    pub fn subsidy_eval(
        &self,
        b: &Balance,
        q: &CostQuery,
        grant: GrantQuery,
        a: f64,
    ) -> Result<SubsidyEval> {
        self.subsidy_eval_at(b, q, grant, a, self.target_c)
    }

    /// Evaluate both methods at one grant. `a` is the annual net abatement,
    /// t/y, and `target` the cost to undercut, S$/t. The `grant` field of `q`
    /// is replaced.
    ///
    /// eq: S.6a
    /// eq: S.6b
    /// eq: S.7e
    /// eq: S.8
    pub fn subsidy_eval_at(
        &self,
        b: &Balance,
        q: &CostQuery,
        grant: GrantQuery,
        a: f64,
        target: f64,
    ) -> Result<SubsidyEval> {
        if !(a > 0.0 && a.is_finite()) {
            return Err(EngineError::Range {
                param: "abatement".into(),
                msg: format!("{a} t/y must be positive"),
            });
        }
        let q0 = CostQuery { grant: None, ..*q };
        let qg = CostQuery {
            grant: Some(grant),
            ..*q
        };
        let k = self.cost_items(b, &q0)?;
        let plan = self.grant_plan(&k, &grant)?;
        let led0 = self.ledger(b, &q0)?;
        let ledg = self.ledger(b, &qg)?;
        let d0 = self.dcf_at(b, &q0, a)?;
        let dg = self.dcf_at(b, &qg, a)?;
        let per_t = 1e6 / a;
        let outlay = crf(self.rate, q.life)? * per_t;
        let lac5_gross = led0.l5() * per_t;
        let lac5_net = ledg.l5_net() * per_t;
        let l5 = MethodEval {
            cost_gross_sm_y: led0.l5(),
            cost_net_sm_y: ledg.l5_net(),
            lac_gross: lac5_gross,
            lac_net: lac5_net,
            interest_gross_sm: plan.ledger_interest_gross,
            interest_private_sm: plan.ledger_interest_private,
            interest_saved_sm: plan.ledger_interest_saved,
            state_pv_sm: plan.ledger_state_pv,
            public_outlay_s_per_t: plan.ledger_state_pv * outlay,
            target_met: lac5_net < target,
        };
        let dcf = MethodEval {
            cost_gross_sm_y: d0.lac * a * 1e-6,
            cost_net_sm_y: dg.lac * a * 1e-6,
            lac_gross: d0.lac,
            lac_net: dg.lac,
            interest_gross_sm: plan.dcf_interest_gross,
            interest_private_sm: plan.dcf_interest_private,
            interest_saved_sm: plan.dcf_interest_saved,
            state_pv_sm: plan.dcf_state_pv,
            public_outlay_s_per_t: plan.dcf_state_pv * outlay,
            target_met: dg.lac < target,
        };
        Ok(SubsidyEval {
            plan,
            abatement_t_y: a,
            life: q.life,
            target,
            target_met: l5.target_met && dcf.target_met,
            gap_s_per_t: dcf.lac_net - l5.lac_net,
            gap_rel: (dcf.lac_net - l5.lac_net) / l5.lac_net,
            l5,
            dcf,
        })
    }

    /// The break-even shares of one scenario against the registry target.
    pub fn breakeven(
        &self,
        b: &Balance,
        q: &CostQuery,
        definition: Definition,
        timing: Timing,
        a: f64,
    ) -> Result<BreakEvenSet> {
        self.breakeven_at(b, q, definition, timing, a, self.target_c)
    }

    /// Solve the continuous break-even share for the ledger and for the DCF.
    ///
    /// The closed forms of S.7b to S.7d give the share. Each is checked against
    /// the cost the model itself returns at that share, and a result more than
    /// `SUBSIDY_RESIDUAL_TOL` from the target is replaced by the bracketed
    /// bisection of S.7d. A cost already below the target at no grant returns
    /// `MetWithoutGrant`. A cost at or above the target at the largest share
    /// returns `Unreachable`.
    ///
    /// eq: S.7b
    /// eq: S.7c
    /// eq: S.7d
    /// eq: S.7e
    pub fn breakeven_at(
        &self,
        b: &Balance,
        q: &CostQuery,
        definition: Definition,
        timing: Timing,
        a: f64,
        target: f64,
    ) -> Result<BreakEvenSet> {
        if !(a > 0.0 && a.is_finite()) {
            return Err(EngineError::Range {
                param: "abatement".into(),
                msg: format!("{a} t/y must be positive"),
            });
        }
        let q0 = CostQuery { grant: None, ..*q };
        let k = self.cost_items(b, &q0)?;
        let plan = self.grant_plan(
            &k,
            &GrantQuery {
                share: 0.0,
                definition,
                timing,
            },
        )?;
        let kappa = crf(self.rate, q.life)? * 1e6 / a;
        let lac5_0 = self.ledger(b, &q0)?.l5() * 1e6 / a;
        let lacd_0 = self.dcf_at(b, &q0, a)?.lac;
        let curve = Curve {
            rate: self.rate,
            years: self.idc_years,
            kappa,
            plan,
            target,
        };
        let with = |s: f64| CostQuery {
            grant: Some(GrantQuery {
                share: s,
                definition,
                timing,
            }),
            ..q0
        };
        let lac5 = |s: f64| -> Result<f64> { Ok(self.ledger(b, &with(s))?.l5_net() * 1e6 / a) };
        let lacd = |s: f64| -> Result<f64> { Ok(self.dcf_at(b, &with(s), a)?.lac) };
        let l5 = self.solve_share(target, lac5, curve.ledger(lac5_0, timing))?;
        let dcf = self.solve_share(target, lacd, curve.dcf(lacd_0, timing, self.subsidy.share_max)?)?;
        Ok(BreakEvenSet {
            both: combine(l5.result, dcf.result),
            l5,
            dcf,
        })
    }

    /// Classify one cost curve against the target and find its break-even
    /// share: the closed form `guess` if the model agrees with it, the
    /// bisection otherwise.
    fn solve_share<F>(&self, target: f64, mut lac: F, guess: f64) -> Result<BreakEvenSolve>
    where
        F: FnMut(f64) -> Result<f64>,
    {
        let max = self.subsidy.share_max;
        let at0 = lac(0.0)?;
        if at0 < target {
            return Ok(BreakEvenSolve {
                result: BreakEven::MetWithoutGrant,
                route: SolveRoute::NotNeeded,
                residual: at0 - target,
            });
        }
        let at_max = lac(max)?;
        if at_max >= target {
            return Ok(BreakEvenSolve {
                result: BreakEven::Unreachable { lac_at_max: at_max },
                route: SolveRoute::NotNeeded,
                residual: at_max - target,
            });
        }
        if guess.is_finite() && (0.0..=max).contains(&guess) {
            let residual = lac(guess)? - target;
            if residual.abs() <= self.subsidy.residual_tol {
                return Ok(BreakEvenSolve {
                    result: BreakEven::Share(guess),
                    route: SolveRoute::ClosedForm,
                    residual,
                });
            }
        }
        let s = bisect_share(&mut lac, target, 0.0, max, self.subsidy.solver_tol)?;
        Ok(BreakEvenSolve {
            result: BreakEven::Share(s),
            route: SolveRoute::Bisection,
            residual: lac(s)? - target,
        })
    }

    /// The share at which the ledger cost meets the target if idle grant cash
    /// earned the discount rate. It is a lower bound that the model does not
    /// use, because it fails the limit at a share of 1 (S.3d, S.9b). It can lie
    /// outside 0 to 1.
    ///
    /// eq: S.3d
    pub fn idle_cash_bound_share(
        &self,
        b: &Balance,
        q: &CostQuery,
        definition: Definition,
        a: f64,
        target: f64,
    ) -> Result<f64> {
        let q0 = CostQuery { grant: None, ..*q };
        let k = self.cost_items(b, &q0)?;
        let plan = self.grant_plan(
            &k,
            &GrantQuery {
                share: 0.0,
                definition,
                timing: Timing::AtClose,
            },
        )?;
        let kappa = crf(self.rate, q.life)? * 1e6 / a;
        let lac5_0 = self.ledger(b, &q0)?.l5() * 1e6 / a;
        let slope = kappa * plan.eligible * (1.0 + self.rate).powf(self.idc_years);
        Ok((lac5_0 - target) / slope)
    }
}
