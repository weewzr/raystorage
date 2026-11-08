WAIT = "document.getElementById('status-text') && /Solved/.test(document.getElementById('status-text').textContent) && document.querySelector('.cost-table') ? true : false"


def run(d):
    d.nav("index.html")
    d.until(WAIT, 30000)
    d.eval("location.hash = '#export'")
    d.wait(1500)
    r = d.eval("""(async () => {
      const v = await import(document.querySelector('script[src^="views.js"]').src);
      const app = await import(document.querySelector('script[src^="app.js"]').src);
      const s = v.streamsCsv();
      const rc = v.resultCsv(app.state);
      const lines = rc.trim().split('\\r\\n');
      const kpi = lines.filter((l) => l.startsWith('kpi,'));
      const cls = {};
      for (const l of kpi) { const c = l.split(',').pop(); cls[c] = (cls[c] || 0) + 1; }
      const sl = s ? s.trim().split('\\r\\n') : [];
      return {streams_rows: sl.length - 1, streams_head: sl[0], streams_first: sl[1], results_rows: lines.length - 1, kpi_classes: cls,
              na_rows: lines.filter((l) => l.endsWith(',n/a')).length,
              buttons: Array.from(document.querySelectorAll('#route-export .btn')).map((b) => (b.dataset.dl || b.getAttribute('download') || '') + (b.disabled ? ' disabled' : ''))};
    })()""")
    d.out("export", r)
