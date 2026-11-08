import json
from pathlib import Path

OUT = Path(__file__).resolve().parent / "dump"
WAIT = "document.getElementById('status-text') && /Solved/.test(document.getElementById('status-text').textContent) ? true : false"
PRE = "const m = await import('./app.js?v=' + (document.querySelector('script[type=module]').src.split('v=')[1]));"


def run(d):
    OUT.mkdir(exist_ok=True)
    d.nav("index.html")
    d.until(WAIT, 30000)
    reg = d.eval("(async () => {" + PRE + " return m.state.registry })()")
    (OUT / "registry.json").write_text(json.dumps(reg, indent=1, ensure_ascii=False), encoding="utf-8")
    cases = {
        "default": {},
        "rating100": {"rating_mwth": 100},
        "case_a": {"basis": "case_a"},
        "mature": {"stage": "10OAK_P46", "life_y": "60", "gas_s_gj": 20.7},
        "oak_gas25": {"stage": "10OAK_P46", "gas_s_gj": 25},
    }
    for name, over in cases.items():
        js = PRE + " const inp = Object.fromEntries(m.state.registry.params.map(p => [p.id, p.default])); Object.assign(inp, %s); const t0 = performance.now(); const {reply, ms} = await m.rpc('solve', {schema: 1, inputs: inp, kpi_only: false}); return {reply, ms, wall: performance.now() - t0};" % json.dumps(over)
        r = d.eval("(async () => {" + js + "})()")
        (OUT / f"{name}.json").write_text(json.dumps(r, indent=1, ensure_ascii=False), encoding="utf-8")
        rep = r.get("reply", {}) if isinstance(r, dict) else {}
        k = rep.get("kpi", {})
        d.out(name, {"status": rep.get("status"), "ms": r.get("ms") if isinstance(r, dict) else None,
                     "l5": k.get("cost_s_per_t"), "dcf": k.get("cost_lac_dcf_s_per_t"),
                     "ng": k.get("ng_mmscfd"), "feasible": (rep.get("feasibility") or {}).get("feasible"),
                     "cost_met": ((rep.get("targets") or {}).get("cost") or {}).get("met")})
