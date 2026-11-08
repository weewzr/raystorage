import json
import os

WAIT = "document.getElementById('status-text') && /Solved/.test(document.getElementById('status-text').textContent) && document.querySelector('.cost-table') ? true : false"
PRE = "const m = await import('./app.js?v=' + (document.querySelector('script[type=module]').src.split('v=')[1]));"
TAG = os.environ.get("AUDIT_TAG", "w")

# Contrast of every visible text node against its effective background. SVG
# text takes its fill and the background of the nearest HTML ancestor, or the
# white plate of the PFD drawing.
CONTRAST = r"""
(() => {
  const parse = (c) => { const m = c.match(/rgba?\(([^)]+)\)/); if (!m) return null; const p = m[1].split(',').map(Number); return {r:p[0],g:p[1],b:p[2],a:p.length>3?p[3]:1}; };
  const lum = (c) => { const f = (v) => { v/=255; return v<=0.03928? v/12.92 : Math.pow((v+0.055)/1.055,2.4); }; return 0.2126*f(c.r)+0.7152*f(c.g)+0.0722*f(c.b); };
  const ratio = (a,b) => { const x=lum(a), y=lum(b); return (Math.max(x,y)+0.05)/(Math.min(x,y)+0.05); };
  const bgOf = (el) => { let e = el; while (e && e.nodeType === 1) { if (e.namespaceURI === 'http://www.w3.org/2000/svg' && e.tagName.toLowerCase() === 'svg' && e.id === 'pfd-svg') return {r:255,g:255,b:255,a:1}; const c = parse(getComputedStyle(e).backgroundColor); if (c && c.a > 0.5) return c; e = e.parentElement; } return {r:255,g:255,b:255,a:1}; };
  const out = []; let n = 0;
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  while (walker.nextNode()) {
    const t = walker.currentNode; if (!t.textContent.trim()) continue;
    const el = t.parentElement; if (!el) continue;
    const r = el.getBoundingClientRect(); if (r.width === 0 || r.height === 0) continue;
    const cs = getComputedStyle(el); if (cs.visibility === 'hidden' || cs.display === 'none') continue;
    if (el.closest('[hidden]') || el.closest('.visually-hidden')) continue;
    const isSvg = el.namespaceURI === 'http://www.w3.org/2000/svg';
    const fg = parse(isSvg ? cs.fill : cs.color); if (!fg) continue;
    const bg = bgOf(el);
    const cr = ratio(fg, bg); n++;
    // Rendered size: SVG text scales with its drawing.
    let px = parseFloat(cs.fontSize);
    if (isSvg && el.ownerSVGElement) { const s = el.ownerSVGElement; const vb = s.viewBox.baseVal; if (vb && vb.width) px = px * s.getBoundingClientRect().width / vb.width; }
    const bold = parseInt(cs.fontWeight) >= 700;
    const large = px >= 24 || (bold && px >= 18.66);
    if (cr < 4.5 && !large) out.push({text: t.textContent.trim().slice(0,40), cls: el.getAttribute('class'), tag: el.tagName, cr: Math.round(cr*100)/100, px: Math.round(px*10)/10, fg: isSvg ? cs.fill : cs.color});
  }
  return {checked: n, fails: out.slice(0, 40), nfail: out.length};
})()
"""


def run(d):
    d.nav("index.html")
    d.until(WAIT, 30000)
    d.wait(800)
    res = {}
    for route in ["dashboard", "pfd", "parameters", "results", "export", "validation"]:
        d.eval(f"location.hash = '#{route}'")
        d.wait(900)
        c = d.eval(CONTRAST)
        sw = d.eval("({doc: document.documentElement.scrollWidth, win: window.innerWidth})")
        tabs = d.eval("Array.from(document.querySelectorAll('a[href], button, input, select, textarea, [tabindex]')).filter(e => e.tabIndex >= 0 && !e.closest('[hidden]') && e.getClientRects().length).length")
        res[route] = {"contrast_checked": c["checked"], "contrast_fails": c["nfail"], "fails": c["fails"][:12], "scroll": sw, "tab_stops": tabs}
        d.shot(f"{TAG}_{route}", full=True)
    d.out("routes", res)
    # Equation overflow inside panels at this width.
    d.eval("location.hash = '#pfd'")
    d.wait(600)
    over = d.eval("""(async () => { const ids = Array.from(document.querySelectorAll('#pfd-select option')).map(o => o.value); const bad = [];
      for (const id of ids) { const s = document.getElementById('pfd-select'); s.value = id; s.dispatchEvent(new Event('change')); await new Promise(r => setTimeout(r, 120));
        const p = document.getElementById('pfd-panel').getBoundingClientRect();
        for (const e of document.querySelectorAll('#pfd-panel .eq-body')) { const r = e.getBoundingClientRect(); if (r.right > p.right + 1) bad.push(id + ':' + Math.round(r.right - p.right)); }
        const doc = document.documentElement.scrollWidth; if (doc > window.innerWidth) bad.push(id + ' page ' + doc); }
      return bad; })()""")
    d.out("eq_overflow", over)
    # Skip link: first Tab stop, Enter focuses main and keeps the route.
    d.eval("location.hash = '#results'")
    d.wait(500)
    d.eval("window.scrollTo(0,0); document.activeElement && document.activeElement.blur(); document.body.focus()")
    d.key("Tab", "Tab", 9)
    first = d.eval("document.activeElement ? (document.activeElement.className + '|' + document.activeElement.textContent.trim()) : null")
    d.key("Enter", "Enter", 13, text="\r")
    d.wait(300)
    after = d.eval("({active: document.activeElement && document.activeElement.id, hash: location.hash})")
    d.out("skip_link", {"first_tab": first, "after_enter": after})
    # Stream keyboard: focus S12, Enter pins, the link is focused.
    d.eval("location.hash = '#pfd'")
    d.wait(600)
    d.eval("document.querySelector('[data-stream=\"S12\"]').focus()")
    d.wait(200)
    tip1 = d.eval("({hidden: document.getElementById('pfd-tip').hidden, text: document.getElementById('pfd-tip').innerText})")
    d.key("Enter", "Enter", 13, text="\r")
    d.wait(200)
    tip2 = d.eval("({active: document.activeElement && document.activeElement.outerHTML.slice(0, 120)})")
    d.key("Enter", "Enter", 13, text="\r")
    d.wait(1500)
    tip3 = d.eval("({hash: location.hash, target: (document.querySelector('tr.is-target') || {}).id, active: document.activeElement && document.activeElement.id, top: document.querySelector('tr.is-target') ? Math.round(document.querySelector('tr.is-target').getBoundingClientRect().top) : null})")
    d.out("stream_keyboard", {"focus_tip": tip1, "after_enter": tip2, "after_link": tip3})
