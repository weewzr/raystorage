//! SMR-TUBE: one-dimensional pseudo-homogeneous reformer tube heated by helium in
//! counterflow.
//!
//! Class ReducedOrder. The rate law is `xf`, extrapolated about 300 K and a factor
//! of 2 in pressure, and the activity lumps catalyst state, effectiveness and bed
//! density. The bed sits at z from `cat_start_m` to `length_m`, process gas moves
//! towards larger z, and helium enters at z = L and leaves at z = 0.
//!
//! With `bayonet` set, gas turns at the tip and returns through a centre tube
//! towards z = 0, handing heat back to the bed, which is the JAERI arrangement.
//! Without it the gas leaves at z = L, which is the INL arrangement.
//!
//! State at each node: extent of CH4 consumed, extent of CO2 formed, bed gas T,
//! helium T, return gas T and pressure. Reactions I, II and III span only two
//! independent extents. Each step of the implicit trapezoid holds the discrete
//! enthalpy balance of each stream exactly, so energy closes to the Newton
//! tolerance and no truncation error enters the balance. A step is checked against
//! two half steps and halved until they agree.
//!
//! Helium and return gas are unknown at z = 0, so the outlet temperatures are
//! shot from z = 0 by Newton on the two end conditions at z = L, on a frozen grid.
//! C2+ has no rate data here, so it converts fully at the inlet at constant
//! enthalpy, as the equilibrium unit treats it.

use crate::gas::closure;
use crate::solve::lu_solve;
use crate::stream::{
    Boundary, Closure, EnergyKind, EnergyStream, ModelClass, Sp, Stream, UnitRecord, NSP,
};
use crate::tube_ht::{self, BedGas, BedHt, FinnedTube};
use crate::wgs::t_from_h;
use crate::xf::{Kinetics, Partial};
use nush2_numerics::{EngineError, Result};
use nush2_props::equilibrium::{self, KSource};
use nush2_props::units::{BAR_PA, R_GAS};
use nush2_props::{helium, mixture, species, transport};

const XA: usize = 0;
const XB: usize = 1;
const T: usize = 2;
const HE: usize = 3;
const RET: usize = 4;
const P: usize = 5;
const NS: usize = 6;

const H2: usize = Sp::H2 as usize;
const CO: usize = Sp::Co as usize;
const CO2: usize = Sp::Co2 as usize;
const CH4: usize = Sp::Ch4 as usize;
const H2O: usize = Sp::H2o as usize;

/// Local error limit of the step check, scaled.
const STEP_TOL: f64 = 2.0e-5;
/// Newton limit on the scaled step residual.
const NEWTON_TOL: f64 = 1.0e-10;
/// Shooting limit on the end conditions, K.
const SHOOT_TOL_K: f64 = 1.0e-6;
const SHOOT_ITERS: usize = 40;
/// Largest shooting move per iteration, K.
const SHOOT_MOVE_K: f64 = 60.0;
/// Conversion seeded at the head of the bed. The rate law diverges at zero H2 (XF1
/// PDF p8), so the first catalyst sees a trace of products, reached at constant
/// enthalpy. Half the seeded CH4 leaves as CO2.
const SEED_CONVERSION: f64 = 1.0e-4;
/// Extra halvings a frozen grid may use inside one of its intervals.
const SUBSTEP_LEVELS: u32 = 7;

/// Heat transfer and helium side data over one stretch of tube, ending at `end_m`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Section {
    pub end_m: f64,
    /// U times perimeter, helium to bed, W/m/K.
    pub up_he_w_m_k: f64,
    /// U times perimeter, return gas to bed, W/m/K.
    pub up_ret_w_m_k: f64,
    /// Net heat leaving the helium for the surroundings, W/m.
    pub q_he_w_m: f64,
}

/// Local overall coefficient from correlations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Correlated {
    pub tube: FinnedTube,
    /// Guide tube inner diameter, m.
    pub guide_id_m: f64,
    pub bed: BedHt,
}

/// Source of the helium side coefficient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeatModel {
    /// The `Section` values stand as given.
    Sections,
    /// `up_he_w_m_k` comes from correlations at the local state.
    Correlated(Correlated),
}

/// One reformer tube.
#[derive(Debug, Clone, PartialEq)]
pub struct TubeSpec {
    pub tag: String,
    pub length_m: f64,
    /// Where the catalyst starts, m.
    pub cat_start_m: f64,
    pub bed_area_m2: f64,
    pub void: f64,
    pub dp_m: f64,
    /// kg of catalyst per m3 of bed.
    pub bulk_density_kg_m3: f64,
    /// Ordered by `end_m`, the last at `length_m`.
    pub sections: Vec<Section>,
    pub heat: HeatModel,
    pub kinetics: Kinetics,
    pub k_source: KSource,
    pub bayonet: bool,
    /// Off holds pressure constant, for limit tests.
    pub ergun: bool,
    pub he_m_kg_s: f64,
    pub he_t_in_k: f64,
    pub he_p_pa: f64,
    /// Starting guess of helium outlet T, K.
    pub he_out_guess_k: Option<f64>,
    /// Starting guess of return gas outlet T, K.
    pub ret_out_guess_k: Option<f64>,
}

/// State at one axial node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Node {
    pub z_m: f64,
    /// Share of the tube-inlet CH4 converted.
    pub conversion: f64,
    pub t_k: f64,
    pub t_he_k: f64,
    /// Equal to `t_k` on a straight tube.
    pub t_ret_k: f64,
    pub p_pa: f64,
}

/// Outcome of a tube.
#[derive(Debug, Clone, PartialEq)]
pub struct TubeResult {
    pub nodes: Vec<Node>,
    pub outlet: Stream,
    pub conversion: f64,
    pub he_out_k: f64,
    /// Heat that left the helium, W.
    pub he_heat_w: f64,
    /// Net heat to the surroundings from the helium side, W.
    pub he_loss_w: f64,
    /// Heat taken up by the process gas, W.
    pub process_heat_w: f64,
    /// Heat handed back through the return tube, W.
    pub return_heat_w: f64,
    pub closure: Closure,
    pub record: UnitRecord,
}

impl TubeResult {
    /// Linear interpolation on the nodes. `z_m` outside the tube takes the end.
    pub fn at(&self, z_m: f64) -> Node {
        let n = &self.nodes;
        let mut i = n.partition_point(|a| a.z_m < z_m);
        if i == 0 {
            return n[0];
        }
        if i >= n.len() {
            i = n.len() - 1;
            return n[i];
        }
        let (a, b) = (n[i - 1], n[i]);
        let w = (z_m - a.z_m) / (b.z_m - a.z_m);
        let mix = |p: f64, q: f64| p + w * (q - p);
        Node {
            z_m,
            conversion: mix(a.conversion, b.conversion),
            t_k: mix(a.t_k, b.t_k),
            t_he_k: mix(a.t_he_k, b.t_he_k),
            t_ret_k: mix(a.t_ret_k, b.t_ret_k),
            p_pa: mix(a.p_pa, b.p_pa),
        }
    }
}

type State = [f64; NS];

/// Everything one node evaluation returns.
#[derive(Debug, Clone, Copy)]
struct Eval {
    ra: f64,
    rb: f64,
    q_bed: f64,
    q_he: f64,
    q_ret: f64,
    h_p: f64,
    h_r: f64,
    dpdz: f64,
}

/// Local data of one grid interval.
#[derive(Debug, Clone, Copy)]
struct Seg {
    sec: Section,
    w_kg_m: f64,
}

struct Path {
    z: Vec<f64>,
    y: Vec<State>,
}

struct Tube<'a> {
    s: &'a TubeSpec,
    n0: [f64; NSP],
    t_in: f64,
    p_in: f64,
    /// Inlet CH4 after the C2+ conversion, mol/s.
    ch4_in: f64,
    m_gas: f64,
    c_he: f64,
    h_scale: f64,
}

/// C_n H_(2n+2) + n H2O = n CO + (2n+1) H2 at the inlet.
fn pre_convert(n: &[f64; NSP]) -> Result<[f64; NSP]> {
    let mut out = *n;
    for sp in [
        species::Sp::C2h6,
        species::Sp::C3h8,
        species::Sp::Nc4h10,
        species::Sp::Nc5h12,
    ] {
        let v = out[sp.index()];
        let c = f64::from(sp.atoms()[0]);
        out[H2O] -= c * v;
        out[CO] += c * v;
        out[H2] += (2.0 * c + 1.0) * v;
        out[sp.index()] = 0.0;
    }
    if out[H2O] <= 0.0 {
        return Err(EngineError::Range {
            param: "feed H2O".into(),
            msg: "the steam fed cannot convert the C2+ species".into(),
        });
    }
    Ok(out)
}

fn positive(name: &str, v: f64) -> Result<()> {
    if v > 0.0 && v.is_finite() {
        Ok(())
    } else {
        Err(EngineError::Range {
            param: name.into(),
            msg: format!("{v} must be positive"),
        })
    }
}

impl<'a> Tube<'a> {
    fn new(s: &'a TubeSpec, feed: &Stream) -> Result<Self> {
        positive("length_m", s.length_m)?;
        positive("bed_area_m2", s.bed_area_m2)?;
        positive("bulk_density_kg_m3", s.bulk_density_kg_m3)?;
        positive("he_m_kg_s", s.he_m_kg_s)?;
        positive("dp_m", s.dp_m)?;
        let void_ok = s.void > 0.0 && s.void < 1.0;
        let start_ok = (0.0..s.length_m).contains(&s.cat_start_m);
        if !(void_ok && start_ok) {
            return Err(EngineError::Range {
                param: "bed".into(),
                msg: format!("void {} or start {} m is invalid", s.void, s.cat_start_m),
            });
        }
        let last = s.sections.last().map(|v| v.end_m);
        if last != Some(s.length_m) {
            return Err(EngineError::Range {
                param: "sections".into(),
                msg: "the last section must end at the tube length".into(),
            });
        }
        if feed.n_liq_h2o != 0.0 {
            return Err(EngineError::Range {
                param: "feed".into(),
                msg: format!("{} carries liquid water into the tube", feed.tag),
            });
        }
        if s.he_t_in_k <= feed.t_k {
            return Err(EngineError::Range {
                param: "he_t_in_k".into(),
                msg: "helium must enter above the feed temperature".into(),
            });
        }
        let n0 = pre_convert(&feed.n)?;
        let t_in = t_from_h(&n0, feed.h_w()?, feed.t_k)?;
        if n0[CH4] <= 0.0 {
            return Err(EngineError::Range {
                param: "feed CH4".into(),
                msg: "the tube needs methane".into(),
            });
        }
        let c_he = s.he_m_kg_s * helium::CP_J_KG_K;
        Ok(Tube {
            s,
            n0,
            t_in,
            p_in: feed.p_pa,
            ch4_in: n0[CH4],
            m_gas: mixture::mass_kg_s(&n0)?,
            c_he,
            h_scale: c_he * 100.0,
        })
    }

    fn flows(&self, xa: f64, xb: f64) -> [f64; NSP] {
        let mut n = self.n0;
        n[CH4] -= xa;
        n[CO2] += xb;
        n[CO] += xa - xb;
        n[H2O] -= xa + xb;
        n[H2] += 3.0 * xa + xb;
        n
    }

    /// Flows for a node evaluation. A difference step at the bed head can dip a
    /// zero flow just below zero, so a tiny negative reads as zero.
    fn flows_checked(&self, xa: f64, xb: f64) -> Result<[f64; NSP]> {
        let mut n = self.flows(xa, xb);
        let floor = -1.0e-5 * self.ch4_in;
        for v in n.iter_mut() {
            if *v < 0.0 {
                if *v < floor {
                    return Err(EngineError::Range {
                        param: "tube flows".into(),
                        msg: format!("flow {v} mol/s is negative"),
                    });
                }
                *v = 0.0;
            }
        }
        Ok(n)
    }

    fn seg(&self, z_mid: f64) -> Seg {
        let s = self.s;
        let sec = s
            .sections
            .iter()
            .find(|v| z_mid <= v.end_m)
            .or(s.sections.last())
            .copied();
        // The constructor proved the list is not empty.
        let sec = sec.unwrap_or(Section {
            end_m: s.length_m,
            up_he_w_m_k: 0.0,
            up_ret_w_m_k: 0.0,
            q_he_w_m: 0.0,
        });
        let w = if z_mid >= s.cat_start_m {
            s.bed_area_m2 * s.bulk_density_kg_m3
        } else {
            0.0
        };
        Seg { sec, w_kg_m: w }
    }

    fn up_he(&self, y: &State, n: &[f64; NSP], seg: &Seg) -> Result<f64> {
        let HeatModel::Correlated(c) = &self.s.heat else {
            return Ok(seg.sec.up_he_w_m_k);
        };
        let s = self.s;
        let he_t = y[HE];
        let mu_he = helium::viscosity_pa_s(he_t)?;
        let k_he = helium::conductivity_w_m_k(he_t, s.he_p_pa)?;
        let pr_he = helium::prandtl(he_t, s.he_p_pa)?;
        let ao = tube_ht::alpha_o_knudsen(&c.tube, c.guide_id_m, s.he_m_kg_s, mu_he, k_he, pr_he)?;
        let tr = transport::syngas(y[T])?;
        let rho = mixture::density_ideal_kg_m3(n, y[T], y[P])?;
        let cp = mixture::cp_j_mol_k(n, y[T])? / mixture::mean_mw_kg_mol(n)?;
        let gas = BedGas {
            t_c: y[T] - 273.15,
            lambda_g: tr.k_w_m_k,
            nu_m2_s: tr.mu_pa_s / rho,
            prandtl: cp * tr.mu_pa_s / tr.k_w_m_k,
            u0_m_s: self.m_gas / (rho * s.bed_area_m2),
        };
        let ai = tube_ht::alpha_i(&c.bed, &gas)?;
        Ok(c.tube.u_outer(ao, ai)? * c.tube.a_outer_per_m())
    }

    fn eval(&self, y: &State, seg: &Seg, n_ret: &[f64; NSP]) -> Result<Eval> {
        let s = self.s;
        let n = self.flows_checked(y[XA], y[XB])?;
        let h_p = mixture::h_flow_w(&n, 0.0, y[T], BAR_PA)?;
        let total: f64 = n.iter().sum();
        let (ra, rb) = if seg.w_kg_m > 0.0 {
            let (k1, k2) = s.k_source.pair(y[T])?;
            let f = y[P] / BAR_PA / total;
            let pp = Partial {
                ch4: n[CH4] * f,
                h2o: n[H2O] * f,
                h2: n[H2] * f,
                co: n[CO] * f,
                co2: n[CO2] * f,
            };
            let r = s.kinetics.rates(y[T], &pp, k1, k2)?;
            (seg.w_kg_m * r.ch4(), seg.w_kg_m * r.co2())
        } else {
            (0.0, 0.0)
        };
        let up_he = self.up_he(y, &n, seg)?;
        let (q_ret, h_r) = if s.bayonet {
            (
                seg.sec.up_ret_w_m_k * (y[RET] - y[T]),
                mixture::h_flow_w(n_ret, 0.0, y[RET], BAR_PA)?,
            )
        } else {
            (0.0, 0.0)
        };
        let q_wall = up_he * (y[HE] - y[T]);
        let dpdz = if s.ergun {
            let mu = transport::syngas(y[T])?.mu_pa_s;
            let rho = mixture::density_ideal_kg_m3(&n, y[T], y[P])?;
            let u = self.m_gas / (rho * s.bed_area_m2);
            let e = s.void;
            let e3 = e * e * e;
            150.0 * (1.0 - e) * (1.0 - e) * mu * u / (e3 * s.dp_m * s.dp_m)
                + 1.75 * (1.0 - e) * rho * u * u / (e3 * s.dp_m)
        } else {
            0.0
        };
        Ok(Eval {
            ra,
            rb,
            q_bed: q_wall + q_ret,
            q_he: q_wall + seg.sec.q_he_w_m,
            q_ret,
            h_p,
            h_r,
            dpdz,
        })
    }

    fn residual(
        &self,
        y0: &State,
        e0: &Eval,
        y1: &State,
        e1: &Eval,
        h: f64,
        theta: f64,
    ) -> [f64; NS] {
        let th = theta;
        let mix = |a: f64, b: f64| th * b + (1.0 - th) * a;
        let mut r = [0.0; NS];
        r[XA] = (y1[XA] - y0[XA] - h * mix(e0.ra, e1.ra)) / self.ch4_in;
        r[XB] = (y1[XB] - y0[XB] - h * mix(e0.rb, e1.rb)) / self.ch4_in;
        r[T] = (e1.h_p - e0.h_p - h * mix(e0.q_bed, e1.q_bed)) / self.h_scale;
        r[HE] = (self.c_he * (y1[HE] - y0[HE]) - h * mix(e0.q_he, e1.q_he)) / self.h_scale;
        r[RET] = if self.s.bayonet {
            (e1.h_r - e0.h_r - h * mix(e0.q_ret, e1.q_ret)) / self.h_scale
        } else {
            y1[RET] - y0[RET]
        };
        r[P] = (y1[P] - y0[P] + h * mix(e0.dpdz, e1.dpdz)) / self.p_in;
        r
    }

    /// One implicit step from `y0` over `h`.
    fn step(&self, y0: &State, h: f64, seg: &Seg, theta: f64, n_ret: &[f64; NSP]) -> Result<State> {
        let e0 = self.eval(y0, seg, n_ret)?;
        // Extents enter on a weight of 1e-3: stiff rates stall them near 1e-7 of the
        // feed, which is far below any reported digit, and elements close through
        // the extents whatever this residual is.
        let norm = |r: &[f64; NS]| {
            let rate = 1.0e-3 * r[XA].abs().max(r[XB].abs());
            r[T..].iter().fold(rate, |m, v| m.max(v.abs()))
        };
        let mut y = *y0;
        let mut e = e0;
        let mut r = self.residual(y0, &e0, &y, &e, h, theta);
        let dx: State = [
            1.0e-7 * self.ch4_in,
            1.0e-7 * self.ch4_in,
            1.0e-4,
            1.0e-4,
            1.0e-4,
            1.0e-1,
        ];
        for _ in 0..30 {
            if norm(&r) < NEWTON_TOL {
                return Ok(y);
            }
            let mut jac = [[0.0; NS]; NS];
            for j in 0..NS {
                let mut t = y;
                t[j] += dx[j];
                let et = self.eval(&t, seg, n_ret)?;
                let rt = self.residual(y0, &e0, &t, &et, h, theta);
                for i in 0..NS {
                    jac[i][j] = (rt[i] - r[i]) / dx[j];
                }
            }
            let Some(d) = lu_solve(jac, r.map(|v| -v)) else {
                return Err(EngineError::NotConverged("singular tube step".into()));
            };
            let mut a = 1.0;
            let mut moved = false;
            for _ in 0..12 {
                let mut t = y;
                for i in 0..NS {
                    t[i] += a * d[i];
                }
                if let Ok(et) = self.eval(&t, seg, n_ret) {
                    let rt = self.residual(y0, &e0, &t, &et, h, theta);
                    if norm(&rt) < norm(&r) {
                        y = t;
                        e = et;
                        r = rt;
                        moved = true;
                        break;
                    }
                }
                a *= 0.5;
            }
            if !moved {
                break;
            }
        }
        let _ = e;
        // The Shomate rows meet with a small jump at 1000 K and 1200 K, which leaves
        // a step that crosses one with a floor near 0.1 W on the residual.
        if norm(&r) < 1.0e5 * NEWTON_TOL {
            Ok(y)
        } else {
            Err(EngineError::NotConverged(format!(
                "tube step at residual {}",
                norm(&r)
            )))
        }
    }

    /// The first step into the bed is backward Euler, since the rate falls so
    /// steeply there that the trapezoid would ring.
    fn theta(&self, z0: f64) -> f64 {
        if self.at_bed_head(z0) {
            1.0
        } else {
            0.5
        }
    }

    fn at_bed_head(&self, z: f64) -> bool {
        (z - self.s.cat_start_m).abs() < 1.0e-12
    }

    /// Node entering the bed: a trace of reaction at constant enthalpy.
    fn seeded(&self, y: &State) -> Result<State> {
        if y[XA] > 0.0 || y[XB] > 0.0 {
            return Ok(*y);
        }
        let h = mixture::h_flow_w(&self.n0, 0.0, y[T], BAR_PA)?;
        let xa = SEED_CONVERSION * self.ch4_in;
        let xb = 0.5 * xa;
        let mut out = *y;
        out[XA] = xa;
        out[XB] = xb;
        out[T] = t_from_h(&self.flows(xa, xb), h, y[T])?;
        Ok(out)
    }

    /// Step over `h`, cutting it into equal parts when a single step fails. The
    /// parts stay off the grid, so a frozen grid keeps its nodes.
    fn step_cut(
        &self,
        y0: &State,
        h: f64,
        seg: &Seg,
        theta: f64,
        n_ret: &[f64; NSP],
        level: u32,
    ) -> Result<State> {
        match self.step(y0, h, seg, theta, n_ret) {
            Ok(y) => Ok(y),
            Err(e) => {
                if level == 0 {
                    return Err(e);
                }
                let m = self.step_cut(y0, 0.5 * h, seg, theta, n_ret, level - 1)?;
                self.step_cut(&m, 0.5 * h, seg, 0.5, n_ret, level - 1)
            }
        }
    }

    fn breaks(&self) -> Vec<f64> {
        let s = self.s;
        let mut b = vec![0.0, s.length_m];
        if s.cat_start_m > 0.0 {
            b.push(s.cat_start_m);
        }
        for v in &s.sections {
            b.push(v.end_m);
        }
        b.sort_by(f64::total_cmp);
        b.dedup_by(|a, c| (*a - *c).abs() < 1.0e-12);
        b
    }

    /// Fixed starting grid, drawn close at the head of the bed.
    fn base_grid(&self) -> Vec<f64> {
        let mut z = vec![0.0];
        let b = self.breaks();
        let target = self.s.length_m / 200.0;
        for w in b.windows(2) {
            let (lo, hi) = (w[0], w[1]);
            let count = (((hi - lo) / target).ceil() as usize).max(4);
            let head = self.at_bed_head(lo);
            for i in 1..=count {
                let f = i as f64 / count as f64;
                let g = if head { f * f * f } else { f };
                z.push(lo + (hi - lo) * g);
            }
        }
        z
    }

    fn march_grid(&self, y0: State, grid: &[f64], n_ret: &[f64; NSP]) -> Result<Path> {
        let mut y = y0;
        let mut out = Path {
            z: Vec::with_capacity(grid.len()),
            y: Vec::with_capacity(grid.len()),
        };
        out.z.push(0.0);
        out.y.push(y);
        for w in grid.windows(2) {
            if self.at_bed_head(w[0]) {
                y = self.seeded(&y)?;
            }
            let seg = self.seg(0.5 * (w[0] + w[1]));
            let theta = self.theta(w[0]);
            y = self
                .step_cut(&y, w[1] - w[0], &seg, theta, n_ret, SUBSTEP_LEVELS)
                .map_err(|e| {
                    EngineError::NotConverged(format!("{e}, from z = {} m, state {y:?}", w[0]))
                })?;
            out.z.push(w[1]);
            out.y.push(y);
        }
        Ok(out)
    }

    /// March with the step check, returning the grid it chose.
    fn march_adaptive(&self, y0: State, n_ret: &[f64; NSP]) -> Result<Path> {
        let scale: State = [self.ch4_in, self.ch4_in, 100.0, 100.0, 100.0, self.p_in];
        let breaks = self.breaks();
        let h_max = self.s.length_m / 100.0;
        let h_min = self.s.length_m * 1.0e-8;
        let mut y = y0;
        let mut z = 0.0;
        let mut out = Path {
            z: vec![0.0],
            y: vec![y],
        };
        for w in breaks.windows(2) {
            let (lo, hi) = (w[0], w[1]);
            let seg = self.seg(0.5 * (lo + hi));
            let head = self.at_bed_head(lo);
            if head {
                y = self.seeded(&y)?;
            }
            let mut h = if head { h_max * 1.0e-4 } else { h_max / 4.0 };
            while z < hi - 1.0e-12 {
                let span = (hi - z).min(h);
                let full = self.step(&y, span, &seg, self.theta(z), n_ret);
                let half = self
                    .step(&y, 0.5 * span, &seg, self.theta(z), n_ret)
                    .and_then(|m| {
                        self.step(&m, 0.5 * span, &seg, self.theta(z + 0.5 * span), n_ret)
                            .map(|e| (m, e))
                    });
                let (Ok(f), Ok((m, e))) = (full, half) else {
                    h = 0.5 * span;
                    if h < h_min {
                        return Err(EngineError::NotConverged(format!(
                            "tube step at z = {z} m fell below the floor"
                        )));
                    }
                    continue;
                };
                let err = (0..NS).fold(0.0_f64, |a, i| a.max((f[i] - e[i]).abs() / scale[i]));
                if err > STEP_TOL && span > h_min {
                    h = 0.5 * span;
                    continue;
                }
                out.z.push(z + 0.5 * span);
                out.y.push(m);
                z += span;
                out.z.push(z);
                out.y.push(e);
                y = e;
                let grow = if err > 0.0 {
                    (0.9 * (STEP_TOL / err).cbrt()).clamp(0.5, 2.0)
                } else {
                    2.0
                };
                h = (span * grow).min(h_max);
            }
            z = hi;
        }
        Ok(out)
    }

    fn start(&self, x: &[f64; 2]) -> State {
        [0.0, 0.0, self.t_in, x[0], x[1], self.p_in]
    }

    fn ends(&self, y: &State) -> [f64; 2] {
        let f = y[HE] - self.s.he_t_in_k;
        let g = if self.s.bayonet { y[RET] - y[T] } else { 0.0 };
        [f, g]
    }

    /// Newton on the helium and return outlet temperatures, on a frozen grid.
    fn shoot(&self, x: &mut [f64; 2], grid: &[f64], n_ret: &mut [f64; NSP]) -> Result<Path> {
        let bayonet = self.s.bayonet;
        let mut last_err = f64::INFINITY;
        for _ in 0..SHOOT_ITERS {
            let path = self.march_grid(self.start(x), grid, n_ret)?;
            let Some(end) = path.y.last().copied() else {
                return Err(EngineError::NotConverged("empty tube path".into()));
            };
            let f = self.ends(&end);
            let n_end = self.flows(end[XA], end[XB]);
            let change = n_end
                .iter()
                .zip(n_ret.iter())
                .fold(0.0_f64, |m, (a, b)| m.max((a - b).abs()));
            *n_ret = n_end;
            let err = f[0].abs().max(f[1].abs());
            if err < SHOOT_TOL_K && (change < 1.0e-9 * self.ch4_in || !bayonet) {
                return Ok(path);
            }
            last_err = err;
            let delta = 0.05;
            let mut xp = *x;
            xp[0] += delta;
            let p0 = self.march_grid(self.start(&xp), grid, n_ret)?;
            let e0 = p0.y.last().copied().unwrap_or(end);
            let f0 = self.ends(&e0);
            let dx = if bayonet {
                let mut xq = *x;
                xq[1] += delta;
                let p1 = self.march_grid(self.start(&xq), grid, n_ret)?;
                let e1 = p1.y.last().copied().unwrap_or(end);
                let f1 = self.ends(&e1);
                let jac = [
                    [(f0[0] - f[0]) / delta, (f1[0] - f[0]) / delta],
                    [(f0[1] - f[1]) / delta, (f1[1] - f[1]) / delta],
                ];
                let Some(d) = lu_solve(jac, [-f[0], -f[1]]) else {
                    return Err(EngineError::NotConverged("singular tube shoot".into()));
                };
                d
            } else {
                [-f[0] / ((f0[0] - f[0]) / delta), 0.0]
            };
            let d = [
                dx[0].clamp(-SHOOT_MOVE_K, SHOOT_MOVE_K),
                dx[1].clamp(-SHOOT_MOVE_K, SHOOT_MOVE_K),
            ];
            // Back off the move while it fails to reduce the end error.
            let mut a = 1.0;
            for _ in 0..5 {
                let trial = [x[0] + a * d[0], x[1] + a * d[1]];
                let better = self
                    .march_grid(self.start(&trial), grid, n_ret)
                    .ok()
                    .and_then(|p| p.y.last().copied())
                    .map(|e| self.ends(&e))
                    .is_some_and(|g| g[0].abs().max(g[1].abs()) < err);
                if better {
                    break;
                }
                a *= 0.5;
            }
            x[0] += a * d[0];
            x[1] += a * d[1];
        }
        Err(EngineError::NotConverged(format!(
            "tube shoot after {SHOOT_ITERS} passes, end error {last_err} K"
        )))
    }
}

/// Solve a tube for its feed.
pub fn solve(spec: &TubeSpec, feed: &Stream) -> Result<TubeResult> {
    let tube = Tube::new(spec, feed)?;
    let he_in = spec.he_t_in_k;
    let span = he_in - tube.t_in;
    let mut x = [
        spec.he_out_guess_k.unwrap_or(tube.t_in + 0.15 * span),
        spec.ret_out_guess_k.unwrap_or(tube.t_in + 0.4 * span),
    ];
    if !spec.bayonet {
        x[1] = tube.t_in;
    }
    let eq = equilibrium::reform(&tube.n0, he_in - 60.0, tube.p_in, spec.k_source)?;
    let mut n_ret = eq.n;
    let grid = tube.base_grid();
    tube.shoot(&mut x, &grid, &mut n_ret)?;
    let chosen = tube.march_adaptive(tube.start(&x), &n_ret)?;
    let path = tube.shoot(&mut x, &chosen.z, &mut n_ret)?;
    build(spec, feed, &tube, &path, &n_ret, &x)
}

fn build(
    spec: &TubeSpec,
    feed: &Stream,
    tube: &Tube<'_>,
    path: &Path,
    n_ret: &[f64; NSP],
    x: &[f64; 2],
) -> Result<TubeResult> {
    let mut nodes = Vec::with_capacity(path.z.len());
    for (z, y) in path.z.iter().zip(&path.y) {
        nodes.push(Node {
            z_m: *z,
            conversion: y[XA] / tube.ch4_in,
            t_k: y[T],
            t_he_k: y[HE],
            t_ret_k: if spec.bayonet { y[RET] } else { y[T] },
            p_pa: y[P],
        });
    }
    let Some(end) = path.y.last().copied() else {
        return Err(EngineError::NotConverged("empty tube path".into()));
    };
    let t_out = if spec.bayonet { x[1] } else { end[T] };
    let outlet = Stream::gas("TUBE_OUT", t_out, end[P], *n_ret);
    let he_heat_w = tube.c_he * (spec.he_t_in_k - x[0]);
    let mut lo = 0.0;
    let mut he_loss_w = 0.0;
    for s in &spec.sections {
        he_loss_w += s.q_he_w_m * (s.end_m - lo);
        lo = s.end_m;
    }
    let process_heat_w = he_heat_w - he_loss_w;
    let closure = closure(&[feed], &[&outlet], process_heat_w)?;
    let return_heat_w = if spec.bayonet {
        mixture::h_flow_w(n_ret, 0.0, end[T], BAR_PA)?
            - mixture::h_flow_w(n_ret, 0.0, x[1], BAR_PA)?
    } else {
        0.0
    };
    let conversion = end[XA] / tube.ch4_in;
    let record = UnitRecord {
        tag: spec.tag.clone(),
        class: ModelClass::ReducedOrder,
        boundary: Boundary::Inside,
        inlets: vec![feed.tag.clone()],
        outlets: vec![outlet.tag.clone()],
        duties: vec![EnergyStream {
            tag: format!("Q_{}", spec.tag),
            kind: EnergyKind::Heat,
            w: process_heat_w,
            t_hot_k: Some(spec.he_t_in_k),
            t_cold_k: Some(x[0]),
            from: "HELIUM".to_string(),
            to: spec.tag.clone(),
        }],
        closure: closure.clone(),
        flags: vec![
            format!(
                "Xu-Froment rates extrapolated, activity {:.4e} lumps catalyst state, \
                 effectiveness and bed density",
                spec.kinetics.activity()
            ),
            "syngas viscosity from a fit that stops at 483 C".to_string(),
        ],
    };
    Ok(TubeResult {
        conversion,
        he_out_k: x[0],
        he_heat_w,
        he_loss_w,
        process_heat_w,
        return_heat_w,
        outlet,
        nodes,
        closure,
        record,
    })
}

/// Molar mass helper for callers that need R on a mass basis.
pub fn gas_density(n: &[f64; NSP], t_k: f64, p_pa: f64) -> Result<f64> {
    let m = mixture::mean_mw_kg_mol(n)?;
    Ok(p_pa * m / (R_GAS * t_k))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::closes;

    fn spec() -> TubeSpec {
        TubeSpec {
            tag: "TUBE".into(),
            length_m: 6.0,
            cat_start_m: 0.0,
            bed_area_m2: 0.01,
            void: 0.45,
            dp_m: 0.005,
            bulk_density_kg_m3: 1000.0,
            sections: vec![Section {
                end_m: 6.0,
                up_he_w_m_k: 200.0,
                up_ret_w_m_k: 30.0,
                q_he_w_m: 0.0,
            }],
            heat: HeatModel::Sections,
            kinetics: Kinetics::with_activity(0.01).unwrap(),
            k_source: KSource::Zhang,
            bayonet: false,
            ergun: true,
            he_m_kg_s: 0.09,
            he_t_in_k: 1130.0,
            he_p_pa: 40.0e5,
            he_out_guess_k: None,
            ret_out_guess_k: None,
        }
    }

    fn feed() -> Stream {
        let mut n = [0.0; NSP];
        n[CH4] = 0.75;
        n[H2O] = 2.65;
        Stream::gas("F", 750.0, 42.0e5, n)
    }

    #[test]
    fn straight_tube_closes() {
        let r = solve(&spec(), &feed()).unwrap();
        closes!(r.closure);
        assert!(r.conversion > 0.1 && r.conversion < 1.0);
        assert!(r.he_out_k < 1130.0);
        let end = r.nodes.last().unwrap();
        assert!((end.t_he_k - 1130.0).abs() < 1e-5);
        assert!(end.p_pa < 42.0e5);
    }

    #[test]
    fn bayonet_tube_closes_and_meets_at_the_tip() {
        let mut s = spec();
        s.bayonet = true;
        let r = solve(&s, &feed()).unwrap();
        closes!(r.closure);
        let end = r.nodes.last().unwrap();
        assert!((end.t_ret_k - end.t_k).abs() < 1e-4);
        assert!(r.return_heat_w > 0.0);
    }

    #[test]
    fn c2_plus_is_converted_at_constant_enthalpy() {
        let mut f = feed();
        f.n[Sp::C2h6 as usize] = 0.05;
        f.n[H2] = 0.1;
        let r = solve(&spec(), &f).unwrap();
        closes!(r.closure);
        assert_eq!(r.outlet.flow(Sp::C2h6), 0.0);
    }

    #[test]
    fn bad_inputs_are_refused() {
        let mut s = spec();
        s.sections[0].end_m = 5.0;
        assert!(solve(&s, &feed()).is_err());
        let mut s = spec();
        s.he_t_in_k = 600.0;
        assert!(solve(&s, &feed()).is_err());
        let mut f = feed();
        f.n_liq_h2o = 0.1;
        assert!(solve(&spec(), &f).is_err());
    }
}
