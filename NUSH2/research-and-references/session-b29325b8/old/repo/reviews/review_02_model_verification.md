# Independent Review 2 — Mathematical and computational model

## Scope and provenance

Review target: current `main` model state after Independent Review 1 closure.
This review examines mathematical closure, thermodynamic consistency, numerical
solver behaviour, conservation, validation logic and whether model outputs are
physically supported. It does not repeat the literature/research-foundation
review and does not reopen Review 1.

## Decision — SUPERSEDED

**This file's former CLOSED decision is superseded by
`reviews/review_02_resolution.md`. Current Review 2 gate: OPEN.**

The later acceptance-criterion reconciliation restored the original Review-2
finding taxonomy and found R2-B02 plus dependent major criteria still open.
Use `reviews/review_02_resolution.md` and `STATUS.md` for current state.
The scientific evidence recorded below remains historical evidence and is not
deleted.

All Review-2 mathematical/computational blockers are resolved and the current
main-branch Rust CI is green. Closure is for the declared screening-model scope:
ideal-gas equilibrium reformer chemistry, bounded PSA recovery uncertainty, and
source-limited IEAGHG reformer-state validation. It is not a claim of detailed
catalyst kinetics, PSA adsorption-cycle prediction, or bankable plant design.

The current repository contains substantial new full-species/equilibrium work,
but the new recycle solver fails its own CI convergence tests. Legacy reduced
recycle results therefore remain screening diagnostics and must not be promoted
to validated predictions.

## Findings

### R2-B01 — Full-species recycle steady-state solver fails acceptance cases
**Disposition: RESOLVED.**

The new `solve_full_recycle6` couples a six-species wet-gas state, SMR
conversion, equilibrium WGS, composition-sensitive PSA recovery, inert purge and
fixed-H2 product feedback. This is directionally the correct strengthening over
the legacy CHO fixed-point model.

However Rust CI run 36533494216 fails:
- `finite_purge_solution_is_nonnegative_when_converged`;
- `fresh_feed_depends_on_reformer_and_psa_performance`.

The first failure means the nominal finite-purge case does not reach the stated
steady-state tolerance. The second means both contrasting reformer/PSA cases do
not converge, so the intended physical dependence cannot yet be demonstrated.

The unstable simultaneous proportional controller has been superseded by a
nested numerical formulation. The inner solve closes recycle composition at
fixed fresh feed; the outer scalar bisection solves the fixed-H2 production
residual. The inert purge is imposed from the steady-state balance
purge_N2 = fresh external N2, eliminating the unphysical zero-purge switch.

The new nominal and bracketing cases converge. CI verifies fixed-H2 product
closure, nonnegative species, exact inert balance, C/H/O/N conservation through
SMR and WGS steps, and sensitivity of fresh-feed demand to reformer/PSA
performance. The old proportional-controller tests were retired only after the
new acceptance tests passed; their tolerances were not weakened.

Evidence: commits `33b9bba5f3b8f229502099758d4b5b5eb7a8d295`,
`331d231afe4a325b338f86b2b4f35f650f6ca49b`,
`5e3fd0e5f33d79c8feaec05b21f9ef8ef19298a5`; CI run 36533912202 passed.

### R2-B02 — Legacy analytical recycle fixed point is structurally purge-blind
**Disposition: RESOLVED.**

The legacy analytical result
`analytical_tail_recycle_fresh_ng_fraction()` has no purge argument and is
algebraically invariant to all positive CO/CH4 conversion and H2-recovery
coefficients. The later purge/carbon ledger removes CO/CH4 but does not feed
those losses back into fresh-feed demand or fixed-H2 production.

This is mathematically self-consistent only for the reduced no-purge surrogate;
it is not a closed physical recycle prediction once Review-1 inert control is
required.

The legacy analytical 0.737 result is retained only as a mathematical/provenance
diagnostic. A new canonical reference interface calls the purge-aware,
thermodynamically constrained nested full-species solver and passes that solved
fresh-feed fraction into lifecycle and NG-displacement calculations. CI verifies
that the new solution is not algebraically identical to the legacy fraction and
that upstream-NG lifecycle burden is computed from the solved fresh feed.

Legacy tests that locked the 0.70-0.77 fraction or downstream economic screens
to the reduced surrogate have been removed from predictive acceptance and an
explicit retirement flag/test prevents treating that path as canonical.

Evidence: commits `c22dc1bf8da07d049a88a8e7fad5875859596ddd`
and `fbe357f3747f647e3c229a2150c17d3522b0ec7e`; CI passed.

### R2-B03 — Thermodynamic layer is not yet coupled to SMR conversion
**Disposition: RESOLVED for the ideal-equilibrium screening model.**

The model now has NIST/JANAF equilibrium constants and an equilibrium WGS solve,
but `solve_full_recycle6` still takes `smr_conversion` as an arbitrary input.
Thus reformer chemistry is only partially thermodynamically constrained.
Numerical convergence would not by itself validate the reformer.

An ideal-gas SMR equilibrium extent solver now closes ln(Q/K)=0 while
conserving C/H/O/N. Coupled SMR/WGS equilibrium sweeps close both reaction
residuals at declared reformer T/P. The formulation is checked first against
the source-limited IEAGHG hot-product composition over 900-950 C; this remains
a thermodynamic compatibility diagnostic because the exact primary-reformer
inlet state is not published. The independently validated radiant-duty envelope
from Review 1 remains the energy-side once-through benchmark.

The same equilibrium formulation is now coupled into the nested full-species
recycle solver. No arbitrary `smr_conversion` appears on this predictive path.
CI verifies convergence at the 900 C reference screen and a physically
non-invariant fresh-feed response when reformer temperature changes to 950 C.

This resolves the mathematical blocker for an ideal-equilibrium screening
model; it does not claim kinetic/catalyst validation.

Evidence: commits `66fbcaaf51f59b3fa6d7308a67ee9dd239f883ea`
and `aab7779e81a462bd02eb65f84c40a552b542b0be`; CI passed.

### R2-M01 — PSA model is a bounded surrogate, not adsorption validation
**Disposition: RESOLVED AS BOUNDED SCREENING UNCERTAINTY.**

`psa6_bounded` conserves H2 but imposes pure-H2 product and a linear impurity
penalty with an arbitrary sensitivity parameter. This is useful for sensitivity,
not predictive PSA performance.

A universal impurity-response slope is not supported by the literature and is
no longer required for the canonical uncertainty screen. Published high-purity
SMR PSA studies support a broad conventional H2-recovery range around 70-90%,
with detailed examples near 75-82%, strong N2 sensitivity, and optimized modern
carbon-captured SMR VPSA above 88%. The IEAGHG reconstructed recovery lies
inside this range.

The model now propagates fixed PSA recoveries of 70%, the IEAGHG source anchor,
and 90% through the same thermodynamic/purge-aware nested recycle solver.
Component balances remain exact, every envelope case converges at fixed H2
product, and CI verifies the physically expected monotonic result that lower
PSA recovery requires no less fresh feed.

This resolves PSA treatment only as a bounded screening uncertainty; it does
not claim adsorption-cycle validation or predict purity/recovery outside the
declared envelope.

Evidence: commit `5e88e46a5712321db4c89c9442cb85354f98f109`,
CI passed. Literature basis includes Li et al. (2019, SMR PSA >99.95% purity,
~80% recovery design), Golmakani et al. (2020, impurity/N2 sensitivity),
and industrial/validated SMR-VPSA literature reporting high-purity recoveries
within/above the selected envelope.

### R2-M02 — Review-1 radiant validation is source-limited, not unique state validation
**Disposition: RESOLVED AS EXPLICIT MODEL LIMITATION.**

The <5% nearest-envelope agreement is a useful independent duty check, but the
600-700 C inlet interval is an uncertainty envelope because the exact primary
reformer inlet is unavailable. The test establishes consistency with the
authoritative radiant metric, not uniqueness of the reconstructed state.

This interpretation has been preserved throughout the thermodynamic coupling:
the IEAGHG hot-product calculation is explicitly called a compatibility
diagnostic, while the independently validated radiant-duty envelope remains the
energy benchmark. No exact unpublished reformer inlet state is claimed.

### R2-M03 — STATUS contains stale contradictory gate/task statements
**Disposition: RESOLVED.**

`STATUS.md` begins with `Independent Review 1 gate: CLOSED` but later still
states `Independent Review 1 gate OPEN` and describes obsolete recycle tasks
and pending-CI states. This can mislead future work selection.

`STATUS.md` has been reduced to a concise canonical state after Review-2
scientific closure; historical detail remains in the review records and git
history.

## Sound work retained

- Source-stream C/H/O reconstruction and interstage-water diagnosis.
- Temperature-dependent NIST/IAPWS property layer.
- IEAGHG radiant-duty and WHB validation as source-limited consistency checks.
- WGS equilibrium/Q/K diagnostics and limiting-case tests.
- Explicit N2 purge requirement and stream-specific CCS topology.
- Common H2 product basis and nested system boundaries.
- Analytical-vs-iterative verification of the legacy reduced model as a
  mathematical surrogate (not physical validation).

## Closure state

Review 2 is closed. The next project milestone is Gate 4 — verified computational
model — but no Gate-4 work is begun by this review closure.

Retained limitations for the next phase:
- reformer chemistry is ideal-gas equilibrium screening, not catalyst kinetics;
- PSA is bounded by a literature-supported recovery envelope, not an adsorption
  cycle simulator;
- IEAGHG primary-reformer inlet reconstruction remains source-limited;
- legacy 0.737 recycle results are provenance diagnostics only, not predictive
  outputs.
