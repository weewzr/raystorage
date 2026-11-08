import json
import os

WAIT = "document.getElementById('status-text') && /Solved/.test(document.getElementById('status-text').textContent) && document.querySelector('.cost-table') ? true : false"
PRE = "const m = await import('./app.js?v=' + (document.querySelector('script[type=module]').src.split('v=')[1]));"
TAG = os.environ.get("AUDIT_TAG", "s")


def clip_of(d, sel):
    r = d.eval(f"(() => {{ const e = document.querySelector('{sel}'); if (!e) return null; e.scrollIntoView(); const r = e.getBoundingClientRect(); return {{x: r.left + window.scrollX, y: r.top + window.scrollY, width: r.width, height: Math.min(r.height, 4000)}}; }})()")
    return r


def shot_el(d, sel, name):
    r = clip_of(d, sel)
    if not r:
        d.out(name, "missing")
        return
    d.wait(200)
    m = d.call("Page.getLayoutMetrics")
    params = {"format": "png", "captureBeyondViewport": True, "clip": {**r, "scale": 1}}
    import base64
    res = d.call("Page.captureScreenshot", params, 60)
    from pathlib import Path
    out = Path(__file__).resolve().parent / "out" / f"{TAG}_{name}.png"
    out.write_bytes(base64.b64decode(res["data"]))
    d.out(name, str(out))


def solve(d, values):
    js = PRE + " for (const p of m.state.registry.params) m.state.inputs[p.id] = p.default; Object.assign(m.state.inputs, %s); m.requestSolve(); return true;" % json.dumps(values)
    d.eval("(async () => {" + js + "})()")
    d.wait(1500)


def run(d):
    d.nav("index.html")
    d.until(WAIT, 30000)
    d.wait(600)
    d.eval("location.hash = '#results'")
    d.wait(1200)
    shot_el(d, "#res-profiles", "profiles")
    shot_el(d, "#res-columns", "columns")
    d.eval("location.hash = '#dashboard'")
    d.wait(600)
    d.eval("document.querySelector('input[name=scenario][value=mature]').click()")
    d.wait(1500)
    shot_el(d, "#route-dashboard", "dash_mature")
    d.out("mature_strip", d.eval("document.getElementById('targets').innerText"))
    d.out("mature_line", d.eval("(document.querySelector('.target-line')||{}).innerText"))
    solve(d, {"rating_mwth": 100})
    shot_el(d, "#route-dashboard", "dash_r100")
    d.out("r100_strip", d.eval("document.getElementById('targets').innerText"))
    d.out("r100_feas", d.eval("document.getElementById('feasibility').innerText"))
    d.out("r100_line", d.eval("(document.querySelector('.target-line')||{}).innerText"))
    solve(d, {"tail_gas": "burn_on_site"})
    d.out("sens_tag", d.eval("document.getElementById('sens-tag').innerText"))
    solve(d, {"eta_power": 0.36})
    d.out("eta_flag", d.eval("document.getElementById('flag-list').innerText"))
    solve(d, {})
    d.eval("location.hash = '#parameters'")
    d.wait(800)
    d.eval("(() => { const b = document.getElementById('p-rating_mwth'); b.focus(); b.value = '200.5'; b.dispatchEvent(new Event('change')); b.blur(); return true; })()")
    d.wait(1500)
    d.out("typed_200_5", d.eval("({box: document.getElementById('p-rating_mwth').value})"))
    d.out("ihx_range", d.eval("document.getElementById('p-ihx_approach_k-range-text').textContent"))
