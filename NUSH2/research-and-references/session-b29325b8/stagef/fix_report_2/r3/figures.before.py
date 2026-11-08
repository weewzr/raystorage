#!/usr/bin/env python3
"""Plan step 6. Draw the report's cost charts from numbers_resolved.json into media/.

Every value and every label number comes from a token, so a rebuild on new results redraws
the charts with no hand edits. Charts are drawn at their printed width, so 9 pt chart text
prints at 9 pt. Colours: the first three slots of the dataviz reference categorical palette
for identity (documented as passing all pairs in light mode), its blue and red poles for
cost cuts and additions, and ink greys for text. Every bar carries its value, so no colour
carries meaning alone.
"""
import json
import math
import os
import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.patches import FancyBboxPatch, Patch, Polygon  # noqa: E402

from tables import ledger_rows  # noqa: E402

B = Path(__file__).resolve().parent
MEDIA = B / "media"
RESOLVED = Path(os.environ.get("NUSH2_RESOLVED", B / "numbers_resolved.json"))
DPI = 300
MM = 1 / 25.4

INK, INK2, GRID, SURFACE = "#0b0b0b", "#52514e", "#e4e3df", "#ffffff"
SLOT1, SLOT2, SLOT3 = "#2a78d6", "#eb6834", "#1baf7a"
ADDS, CUTS, TOTAL = "#e34948", "#2a78d6", "#52514e"
N_SERIES = (("N1P00", "n = 1.0, INL recipe", SLOT1),
            ("N0P60", "n = 0.6, project assumption", SLOT2),
            ("N0P31", "n = 0.31, ratio of two FOAK estimates", SLOT3))
# One label set for the ledger: the rows of Table 7 (tables.ledger_rows) name the chart bars too.
LEDGER = tuple((level, line, name) for level, name, line in ledger_rows())
TOTAL_LABELS = {"DECOM": "L3 named step", "BASE_POWER": "L5 full cost"}


def load():
    return json.loads(RESOLVED.read_text(encoding="utf-8"))["tokens"]


def axes(width_mm, height_mm):
    plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 8.5, "axes.edgecolor": GRID,
                         "text.color": INK, "axes.labelcolor": INK2, "xtick.color": INK2, "ytick.color": INK2})
    fig, ax = plt.subplots(figsize=(width_mm * MM, height_mm * MM), dpi=DPI)
    fig.patch.set_facecolor(SURFACE)
    ax.set_facecolor(SURFACE)
    for side in ("top", "right", "left", "bottom"):
        ax.spines[side].set_visible(False)
    ax.tick_params(length=0)
    ax.set_axisbelow(True)
    return fig, ax


def save(fig, name):
    out = MEDIA / name
    fig.savefig(out, dpi=DPI, facecolor=SURFACE)
    plt.close(fig)
    print("chart:", out.name)


def signed(t):
    """A step token's own printed text with its sign written out, for a waterfall label."""
    return ("+" if t["value"] >= 0 else "−") + t["text"].lstrip("-")


def cost_grid(tok):
    """Full cost L5 at BOAK and FOAK and both lives, three size treatments at equal weight, axis from zero."""
    cases = (("BOAK", 25), ("BOAK", 60), ("FOAK", 25), ("FOAK", 60))
    fig, ax = axes(150, 82)
    fig.subplots_adjust(left=0.09, right=0.99, top=0.84, bottom=0.09)
    width = 0.26
    top = 0.0
    for j, (tag, name, color) in enumerate(N_SERIES):
        for i, (stage, life) in enumerate(cases):
            t = tok[f"AC_L5_{stage}_{life}Y_{tag}"]
            x = i + (j - 1) * width
            ax.bar(x, t["value"], width=width - 0.025, color=color, linewidth=0, zorder=3, label=name if i == 0 else None)
            ax.text(x, t["value"] + 5, t["text"], ha="center", va="bottom", fontsize=8, color=INK, zorder=4)
            top = max(top, t["value"])
    target = tok["TARGET_COST"]["value"]
    ax.axhline(target, color=INK, linewidth=1.0, linestyle=(0, (5, 3)), zorder=5, label=f"S${target:.0f}/t target")
    ax.set_xticks(range(len(cases)))
    ax.set_xticklabels([f"{s}, {y} y" for s, y in cases], fontsize=8.5, color=INK)
    ax.set_xlim(-0.5, 3.65)
    ax.set_ylim(0, top * 1.1)
    ax.yaxis.grid(True, color=GRID, linewidth=0.6)
    ax.axhline(0, color=INK2, linewidth=0.8, zorder=2)
    ax.set_ylabel("Full cost L5, S$/tCO₂e", fontsize=8.5)
    fig.legend(loc="upper center", bbox_to_anchor=(0.54, 1.0), ncol=2, frameon=False, fontsize=8,
               handlelength=1.6, columnspacing=1.4, handletextpad=0.4)
    save(fig, "fig_cost_grid.png")


def waterfall(tok, life):
    """Each ledger line as its step in S$/t, with L3 and L5 as cumulative rows."""
    key = f"BOAK_{life}Y_N1P00"
    rows, cum = [], 0.0
    for level, line, name in LEDGER:
        end = tok[f"CUM_{line}_{key}"]["value"]
        step = tok[f"STEP_{line}_{key}"]
        if abs((end - cum) - step["value"]) > 1e-6:
            sys.exit(f"STEP_{line}_{key} does not span its bar; the chart order no longer follows the ledger")
        rows.append((level, name, cum, end, False, signed(step)))
        cum = end
        if line in ("DECOM", "BASE_POWER"):
            token = ("AC_L3_" if line == "DECOM" else "AC_L5_") + key
            if abs(end - tok[token]["value"]) > 1e-3:
                sys.exit(f"ledger order no longer closes on {token}")
            rows.append(("", TOTAL_LABELS[line], 0.0, end, True, tok[token]["text"]))
    fig, ax = axes(150, 92)
    fig.subplots_adjust(left=0.36, right=0.97, top=0.9, bottom=0.1)
    low = min(min(r[2], r[3]) for r in rows)
    high = max(max(r[2], r[3]) for r in rows)
    span = high - low
    seen = set()
    values = []
    for i, (level, name, start, end, total, label) in enumerate(rows):
        color = TOTAL if total else (ADDS if end >= start else CUTS)
        ax.barh(i, end - start, left=start, height=0.64, color=color, linewidth=0, zorder=3)
        if i:
            ax.plot([rows[i - 1][3]] * 2, [i - 1 + 0.32, i - 0.32], color=INK2, linewidth=0.5, zorder=2)
        values.append(ax.text(max(start, end) + span * 0.012, i, label, va="center", ha="left",
                              fontsize=8, fontweight="bold" if total else "normal", zorder=6,
                              bbox=dict(boxstyle="square,pad=0.06", facecolor=SURFACE, edgecolor="none")))
        if level and level not in seen:
            seen.add(level)
            ax.annotate(level, xy=(0, i), xycoords=("axes fraction", "data"), xytext=(-150, 0), textcoords="offset points",
                        va="center", fontsize=8, color=INK2, fontweight="bold")
    ax.set_yticks(range(len(rows)))
    ax.set_yticklabels([r[1] for r in rows], fontsize=8, color=INK)
    for lab, r in zip(ax.get_yticklabels(), rows):
        if r[4]:
            lab.set_fontweight("bold")
    ax.set_ylim(len(rows) - 0.4, -0.6)
    ax.set_xlim(low - span * 0.12, high + span * 0.14)
    ax.xaxis.grid(True, color=GRID, linewidth=0.6)
    ax.axvline(0, color=INK2, linewidth=0.8, zorder=2)
    target = tok["TARGET_COST"]["value"]
    ax.axvline(target, color=INK, linewidth=1.0, linestyle=(0, (5, 3)), zorder=2.5)
    ax.text(target, -0.75, f"S${tok['TARGET_COST']['value']:.0f}/t target", ha="center", va="bottom",
            fontsize=8, color=INK)
    # A value label whose box would cross the target line moves to the far side of the line.
    renderer = fig.canvas.get_renderer()
    to_data = ax.transData.inverted()
    clear = span * 0.012
    for label in values:
        (x0, _), (x1, _) = to_data.transform(label.get_window_extent(renderer).get_points())
        if x0 - clear < target < x1 + clear:
            label.set_x(target + clear)
            (x0, _), _ = to_data.transform(label.get_window_extent(renderer).get_points())
            if x0 <= target:
                sys.exit(f"value label {label.get_text()!r} still crosses the target line")
    ax.set_xlabel(f"S$/tCO₂e, BOAK, {life} y, n = 1.0, {tok['R_RATED_MWTH']['value']:.0f} MWth charged", fontsize=8)
    fig.legend(handles=[Patch(color=ADDS, label="Adds cost"), Patch(color=CUTS, label="Cuts cost"),
                        Patch(color=TOTAL, label="Cumulative")],
               loc="upper left", bbox_to_anchor=(0.01, 0.995), ncol=3, frameon=False, fontsize=8,
               handlelength=1.0, columnspacing=1.2, handletextpad=0.4)
    save(fig, f"fig_ledger_{life}y.png")


def stepdown(tok):
    """The STEPDOWN family: base, tenth of a kind at 25 y, at 60 y, then gas at the printed price.
    L5 and DCF bars side by side for every step, the target line and the S$96/t solve point."""
    gas_c = f"{tok['STEPDOWN_1_GAS_S_GJ']['value']:g}"
    gas_x = f"{tok['STEPDOWN_4_GAS_S_GJ']['value']:g}"
    labels = [f"Base\nBOAK, 25 y\ngas S${gas_c}/GJ", f"Tenth of a kind\n25 y\ngas S${gas_c}/GJ",
              f"Tenth of a kind\n60 y\ngas S${gas_c}/GJ", f"MATURE_60Y\ntenth of a kind\n60 y, gas S${gas_x}/GJ"]
    fig, ax = axes(132, 74)
    fig.subplots_adjust(left=0.09, right=0.99, top=0.88, bottom=0.2)
    width = 0.36
    top = 0.0
    target = tok["TARGET_COST"]["value"]
    point = tok["TARGET_MATURE_COST"]["value"]
    for j, (method, name, color) in enumerate((("L5", "Ledger L5", SLOT1), ("DCF", "Discounted cash flow", SLOT2))):
        for i in range(4):
            t = tok[f"STEPDOWN_{i + 1}_{method}"]
            x = i + (j - 0.5) * width
            ax.bar(x, t["value"], width=width - 0.03, color=color, linewidth=0, zorder=3, label=name if i == 0 else None)
            if t["value"] < target + 12:
                # Inside the bar, so the target line never crosses a value.
                ax.text(x, t["value"] - 22, t["text"], ha="center", va="bottom", fontsize=8, color=SURFACE,
                        fontweight="bold", zorder=8)
            else:
                ax.text(x, t["value"] + 4, t["text"], ha="center", va="bottom", fontsize=8, color=INK, zorder=4)
            top = max(top, t["value"])
    ax.axhline(target, color=INK, linewidth=1.0, linestyle=(0, (5, 3)), zorder=5)
    ax.text(4.1, target + 3, f"S${target:.0f}/t target", ha="right", va="bottom", fontsize=8, color=INK, zorder=6)
    ax.plot([3 - width - 0.02, 3 + width + 0.02], [point, point], color=INK, linewidth=1.0, linestyle=(0, (1, 1.5)), zorder=6)
    ax.plot([3 + width + 0.02], [point], marker="o", markersize=4, color=INK, zorder=7)
    ax.text(4.1, point - 4, f"S${point:.0f}/t\nsolve point", ha="right", va="top", fontsize=8, color=INK, zorder=7)
    ax.set_xticks(range(4))
    ax.set_xticklabels(labels, fontsize=8, color=INK)
    ax.set_xlim(-0.5, 4.12)
    ax.set_ylim(0, top * 1.12)
    ax.yaxis.grid(True, color=GRID, linewidth=0.6)
    ax.axhline(0, color=INK2, linewidth=0.8, zorder=2)
    ax.set_ylabel("Abatement cost, S$/tCO₂e", fontsize=8.5)
    fig.legend(loc="upper center", bbox_to_anchor=(0.54, 1.0), ncol=2, frameon=False, fontsize=8,
               handlelength=1.0, columnspacing=1.6, handletextpad=0.4)
    save(fig, "fig_stepdown.png")


PRES_FIG = B.parents[1] / "presentation" / "figures"
SVG_NS = "{http://www.w3.org/2000/svg}"
MIN_PT = 8.0


# Browser pass for Figure 2: for each label marked data-raise, in order, the first nudge that leaves
# it touching no other text and no mark, by the tests of build_figures.COLLISION_JS. A label that no
# nudge clears at its own spacing is tried again with its letters drawn a fraction of a pixel closer
# (TIGHTEN), so its glyphs keep the raised size. Only a label that still touches something is tried at
# sizes half a pixel smaller in turn, down to its size in the deck drawing, and the build prints it.
# A label whose group already holds a placed line tries that line's sideways nudge first, so the
# lines of one label, and the rows of one legend column, stay aligned. Legend labels, which the deck's own check leaves out of the mark
# test, are held here to their legend box and clear of every swatch, their own included.
PLACE_JS = """
<pre id="report"></pre>
<script>
(function(){
 var svg=document.querySelector('svg'), all=[].slice.call(svg.querySelectorAll('text')), nudges=NUDGES, pad=4, out={};
 var groupDx=new Map();
 function key(t){return t.closest('#legend')?'legend column '+t.getAttribute('x'):t.parentNode;}
 function order(t){
  var idx=nudges.map(function(n,i){return i;}), out=[];
  var first=groupDx.has(key(t))?idx.filter(function(i){return nudges[i][0]===groupDx.get(key(t));}):idx;
  var rest=idx.filter(function(i){return first.indexOf(i)<0;});
  [first,rest].forEach(function(part){TIGHTEN.forEach(function(s){part.forEach(function(i){out.push([i,s]);});});});
  return out;
 }
 function touches(t){
  var a=t.getBoundingClientRect();
  for(var j=0;j<all.length;j++){
   if(all[j]===t)continue;
   var b=all[j].getBoundingClientRect();
   var w=Math.min(a.right,b.right)-Math.max(a.left,b.left), h=Math.min(a.bottom,b.bottom)-Math.max(a.top,b.top);
   if(w>1&&h>1&&h>0.25*Math.min(a.height,b.height))return true;
  }
  var legend=t.closest('#legend'), frame=legend&&legend.querySelector(':scope > rect'), group=legend?null:t.parentNode;
  if(frame){
   var f=frame.getBoundingClientRect();
   if(a.left<f.left+pad||a.right>f.right-pad||a.top<f.top||a.bottom>f.bottom)return true;
  }
  for(var x=a.left-pad;x<a.right+pad;x+=2){for(var y=a.top+a.height*0.18-pad;y<a.bottom-a.height*0.12+pad;y+=2){
   var es=document.elementsFromPoint(x,y);
   for(var k=0;k<es.length;k++){
    var e=es[k];
    if(e===t||e===svg||e.tagName==='HTML'||e.tagName==='BODY'||e.tagName==='text'||e.id==='background')continue;
    if(e.id==='report'||e.tagName==='svg'||e===frame||(group&&group.contains(e)))continue;
    return true;
   }
  }}
  return false;
 }
 [].slice.call(svg.querySelectorAll('text[data-raise]')).forEach(function(t){
  var x0=parseFloat(t.getAttribute('x')), y0=parseFloat(t.getAttribute('y')), found=null;
  var size0=parseFloat(t.getAttribute('font-size')), orig=parseFloat(t.getAttribute('data-orig')), tries=order(t), k=key(t);
  for(var size=size0;size>=orig&&found===null;size-=0.5){
   t.setAttribute('font-size',size);
   for(var q=0;q<tries.length;q++){
    var i=tries[q][0], sp=tries[q][1];
    if(sp)t.setAttribute('letter-spacing',sp);else t.removeAttribute('letter-spacing');
    t.setAttribute('x',x0+nudges[i][0]); t.setAttribute('y',y0+nudges[i][1]);
    if(!touches(t)){found=[i,sp,size];break;}
   }
  }
  if(found===null){t.setAttribute('x',x0);t.setAttribute('y',y0);t.setAttribute('font-size',orig);t.removeAttribute('letter-spacing');}
  else if(!groupDx.has(k))groupDx.set(k,nudges[found[0]][0]);
  out[t.getAttribute('data-raise')]=found;
 });
 document.getElementById('report').textContent=JSON.stringify(out);
})();
</script>"""


TIGHTEN = [0, -0.4, -0.8]


def place_labels(bf, svg, nudges):
    """One headless browser run of PLACE_JS on the raised drawing; returns {order: [nudge index,
    letter spacing in px, font size in px] or None}."""
    import re
    box = re.search(r'viewBox="0 0 ([\d.]+) ([\d.]+)"', svg)
    w, h = int(float(box[1])), int(float(box[2]))
    js = PLACE_JS.replace("NUDGES", json.dumps([list(n) for n in nudges])).replace("TIGHTEN", json.dumps(TIGHTEN))
    old = bf.BROWSER_TIMEOUT_S
    bf.BROWSER_TIMEOUT_S = 900
    try:
        dom = bf.run_browser({"html": bf.page(svg, w, js), "flags": [f"--window-size={w},{h + 200}", "--dump-dom"]})
    finally:
        bf.BROWSER_TIMEOUT_S = old
    m = re.search(r'<pre id="report">(.*?)</pre>', dom, re.S)
    if not m:
        sys.exit("Figure 2 label placement report missing from the browser output.")
    return json.loads(m[1].replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">"))


def pfd_report(width_mm=255.0):
    """Figure 2 for its landscape page: the deck drawing as it stands, with any label below 8 pt at
    this print width raised to the size that reaches it. The browser checks that the raised labels add
    no text overlap the drawing did not already have, then renders it at 4000 px."""
    import hashlib
    import re
    sys.path.insert(0, str(PRES_FIG))
    import build_figures as bf
    svg = (PRES_FIG / "Figure_2_NUS_H2_210MWth.svg").read_text(encoding="utf-8")
    view = float(re.search(r'viewBox="0 0 ([\d.]+) ', svg)[1])
    pt_per_px = width_mm * 72 / 25.4 / view
    need = math.ceil(MIN_PT / pt_per_px)
    small = sorted({float(x) for x in re.findall(r'font-size="([\d.]+)"', svg) if float(x) < need})
    label_re = re.compile(r'<text((?: [a-z-]+="[^"]*")*?) x="([\d.]+)" y="([\d.]+)"([^>]*?) font-size="([\d.]+)"'
                          r'([^>]*)>([^<]*)</text>')
    # Nudges in px tried for a raised label that touches something, smallest move first.
    steps_x = (0, -1, 1, -2, 2, -3, 3, -4, 4, -5, 5, -6, 6, -8, 8, -10, 10, -12, 12)
    steps_y = (0, -2, 2, -3, 3, -4, 4, -5, 5, -6, 6, -8, 8, -10, 10, -12, 12, -14, 14, -16, 16)
    nudges = sorted(((dx, dy) for dx in steps_x for dy in steps_y), key=lambda d: (d[0] ** 2 + d[1] ** 2, abs(d[1])))

    def raise_labels(step, tag=False):
        """Raise every small label to `need`, moved by its nudge; step maps the label's order among the
        small labels to (nudge index, letter spacing in px, font size in px). tag=True marks each raised
        label with that order and its own size for the browser."""
        order = iter(range(10 ** 6))

        def one(m):
            if float(m[5]) >= need:
                return m[0]
            i = next(order)
            k, tight, size = step.get(i, (0, 0, need))
            dx, dy = nudges[k]
            mark = f' data-raise="{i}" data-orig="{m[5]}"' if tag else ""
            mark += f' letter-spacing="{tight:g}"' if tight else ""
            return (f'<text{mark}{m[1]} x="{float(m[2]) + dx:g}" y="{float(m[3]) + dy:g}"{m[4]} font-size="{size:g}"{m[6]}>'
                    f'{m[7]}</text>')
        return label_re.sub(one, svg)

    unmatched = len([x for x in re.findall(r'font-size="([\d.]+)"', svg) if float(x) < need]) - \
        len([m for m in label_re.finditer(svg) if float(m[5]) < need])
    if unmatched:
        sys.exit(f"{unmatched} small Figure 2 labels are not in the form this raise can edit")

    out, mark = MEDIA / "fig_pfd_report.png", MEDIA / "fig_pfd_report.sha"
    stamp = hashlib.sha256((svg + str(width_mm) + str(MIN_PT) + PLACE_JS + str(TIGHTEN)).encode("utf-8")).hexdigest()
    if out.exists() and mark.exists() and mark.read_text() == stamp:
        print(f"chart: {out.name} unchanged")
        return
    step = {}
    out_svg = raise_labels(step)
    if small:
        # A raised label may touch a mark or a label beside it. One browser pass places each raised
        # label in turn at the smallest nudge that gives it no overlap, by the same tests as
        # build_figures.collisions; a second collisions run on the result confirms it adds none.
        before = set(bf.collisions(svg)["hits"])
        if before:
            sys.exit(f"Figure 2 already has overlaps before its labels are raised: {sorted(before)[:6]}")
        texts = [m[7] for m in label_re.finditer(svg) if float(m[5]) < need]
        placed = place_labels(bf, raise_labels({}, tag=True), nudges)
        for i, s in enumerate(texts):
            if placed.get(str(i)) is None:
                sys.exit(f"Figure 2 label {s!r} has no nudge within the tried range that clears every mark")
            step[i] = tuple(placed[str(i)])
        out_svg = raise_labels(step)
        extra = set(bf.collisions(out_svg)["hits"]) - before
        if extra:
            sys.exit(f"raising the small labels of Figure 2 adds overlaps: {sorted(extra)[:6]}")
        moved = {texts[i]: nudges[k] for i, (k, _, _) in step.items() if k}
        tightened = {texts[i]: t for i, (_, t, _) in step.items() if t}
        short = {texts[i]: f"{z:g} px, {z * pt_per_px:.2f} pt" for i, (_, _, z) in step.items() if z < need}
        print(f"Figure 2 labels {small} px raised to {need} px; nudged {moved}; letter spacing {tightened}")
        if short:
            print(f"Figure 2 labels left below {MIN_PT:g} pt because no nudge clears them at {need} px: {short}")
    smallest = min(float(x) for x in re.findall(r'font-size="([\d.]+)"', out_svg)) * pt_per_px
    bf.render_png(out_svg, out, 4000)
    mark.write_text(stamp)
    print(f"chart: {out.name}, smallest label {smallest:.2f} pt at {width_mm:g} mm")


def epz_schematic():
    """Figure 3: the shape of a frequency-consequence target, with no values on either axis. The
    points only place the line on the page, with the same shape and labels as the deck's schematic;
    the values and their units belong to the cited application's own figure."""
    import re
    plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 8, "text.color": INK})
    fig, ax = plt.subplots(figsize=(120 * MM, 63 * MM), dpi=DPI)
    fig.patch.set_facecolor(SURFACE)
    fig.subplots_adjust(left=0.08, right=0.985, top=0.985, bottom=0.12)
    ax.set_xscale("log")
    ax.set_yscale("log")
    xs, ys = [1e-3, 1.0, 1.0, 1e3], [1e2, 1e-1, 1e-2, 1e-6]
    ax.set_xlim(1e-3, 1e3)
    ax.set_ylim(1e-6, 1e2)
    ax.fill_between(xs, ys, 1e2, color="#fbefef", zorder=1, linewidth=0)
    ax.fill_between(xs, 1e-6, ys, color="#e8f4ef", zorder=1, linewidth=0)
    ax.plot(xs, ys, color="#1f3864", linewidth=1.6, zorder=4)
    ax.set_xticks([])
    ax.set_yticks([])
    ax.minorticks_off()
    for side in ("top", "right"):
        ax.spines[side].set_visible(False)
    for side in ("left", "bottom"):
        ax.spines[side].set_color(INK2)
    ax.set_xlabel("Dose at the emergency planning zone boundary, log scale", fontsize=8, color=INK2)
    ax.set_ylabel("Event sequence frequency, log scale", fontsize=8, color=INK2)
    note = dict(fontsize=8, color="#1f3864", zorder=6)
    arrow = dict(arrowstyle="-", color=INK2, linewidth=0.6)
    ax.annotate("10 CFR 20", xy=(0.03, 10 ** (2 - 3 * (math.log10(0.03) + 3) / 3)), xytext=(0.004, 0.06),
                arrowprops=arrow, **note)
    ax.annotate("EPA PAG", xy=(1.0, 0.03), xytext=(3.0, 0.3), arrowprops=arrow, **note)
    ax.annotate("10 CFR 50.34", xy=(30.0, 10 ** (-2 - 4 * math.log10(30.0) / 3)), xytext=(60.0, 3e-3),
                arrowprops=arrow, **note)
    ax.text(30.0, 6.0, "Above the line:\nnot acceptable", fontsize=8, color="#b3261e", ha="center", zorder=6)
    ax.text(2e-3, 2e-5, "Below the line: the sequence meets\nthe target at the zone boundary", fontsize=8,
            color="#1d6b52", ha="left", zorder=6)
    ax.text(1.4e-3, 2.2e-6, "Schematic, no values", fontsize=8, color=INK2, ha="left", style="italic", zorder=6)
    for text in [t.get_text() for t in ax.texts] + [ax.get_xlabel(), ax.get_ylabel()]:
        if re.search(r"\bSMRs?\b|\d+(?:\.\d+)?\s*(?:mSv|rem|Sv)\b", text):
            sys.exit(f"schematic holds a bare abbreviation or a dose value: {text!r}")
    save(fig, "fig_epz.png")


def wrap(fig, text, width_mm, size, weight="normal"):
    """Greedy word wrap measured with the renderer, so a line never runs past its box."""
    r = fig.canvas.get_renderer()
    lines, cur = [], ""
    for word in text.split():
        trial = f"{cur} {word}".strip()
        t = fig.text(0, 0, trial, fontsize=size, fontweight=weight)
        w = t.get_window_extent(r).width / fig.dpi * 25.4
        t.remove()
        if w <= width_mm or not cur:
            cur = trial
        else:
            lines.append(cur)
            cur = word
    lines.append(cur)
    return lines


def roadmap(width_mm=170.0):
    """Figure 4 at its printed width with every label at 8 pt or more. The words, dates, colours and
    column positions are read from the deck's roadmap drawing, so the two say the same thing."""
    from xml.etree import ElementTree as ET
    root = ET.parse(PRES_FIG / "Figure_Roadmap_NUS_H2.svg").getroot()
    view = float(root.get("viewBox").split()[2])
    k = width_mm / view
    groups = {g.get("id"): g for g in root.iter(SVG_NS + "g") if g.get("id")}

    def texts(gid):
        return [t.text for t in groups[gid].iter(SVG_NS + "text")]

    def points(el):
        return [tuple(float(v) for v in xy.split(",")) for xy in el.get("points").split()]

    phases = []
    for i in range(1, 5):
        g, d = groups[f"phase-{i}"], groups[f"phase-{i}-details"]
        rect = d.find(SVG_NS + "rect")
        words = texts(f"phase-{i}-details")
        cut = words.index("KEY REQUIREMENT")
        phases.append(dict(color=g.find(SVG_NS + "polygon").get("fill"), xs=[x for x, _ in points(g.find(SVG_NS + "polygon"))],
                           dates=texts(f"phase-{i}")[0], title=" ".join(texts(f"phase-{i}")[1:]),
                           head1=words[0], lead=" ".join(words[1:cut]), head2=words[cut], req=" ".join(words[cut + 1:]),
                           box=(float(rect.get("x")), float(rect.get("width"))), fill=rect.get("fill")))
    years = [(float(t.get("x")), t.text, t.get("text-anchor")) for t in groups["year-labels"].iter(SVG_NS + "text")]
    gates = [float(t.get("x")) for t in groups["decision-gates"].iter(SVG_NS + "text")]
    strip = [(float(r.get("x")), float(r.get("width")), r.get("fill")) for r in groups["timeline-strip"].iter(SVG_NS + "rect")]
    links = [(points(pg), pg.get("fill")) for pg in groups["timeline-connectors"].iter(SVG_NS + "polygon")]

    S_TOP, S_YEAR, S_DATE, S_TITLE, S_HEAD, S_BODY = 8.0, 8.5, 8.0, 8.6, 8.0, 8.0
    plt.rcParams.update({"font.family": "DejaVu Sans"})
    probe = plt.figure(figsize=(width_mm * MM, 50 * MM), dpi=DPI)
    for ph in phases:
        inner = ph["box"][1] * k - 3.6
        ph["title_lines"] = wrap(probe, ph["title"], (ph["xs"][1] - ph["xs"][0]) * k - 5.5, S_TITLE, "bold")
        ph["lead_lines"] = wrap(probe, ph["lead"], inner, S_BODY)
        ph["req_lines"] = wrap(probe, ph["req"], inner, S_BODY)
    plt.close(probe)
    pitch = S_BODY * 1.28 * 25.4 / 72
    tpitch = S_TITLE * 1.2 * 25.4 / 72
    strip_y, chev_top = 10.2, 17.0
    chev_h = 4.4 + tpitch * max(len(p["title_lines"]) for p in phases) + 1.0
    box_top = chev_top + chev_h + 1.6
    box_h = max(4.6 + pitch * len(p["lead_lines"]) + 2.2 + 3.8 + pitch * len(p["req_lines"]) + 1.0 for p in phases)
    height = box_top + box_h + 0.6

    fig = plt.figure(figsize=(width_mm * MM, height * MM), dpi=DPI)
    fig.patch.set_facecolor(SURFACE)
    ax = fig.add_axes([0, 0, 1, 1])
    ax.set_xlim(0, width_mm)
    ax.set_ylim(height, 0)
    ax.axis("off")

    def X(x):
        return x * k

    grey, dark = "#4a5058", "#1d2329"
    ax.text(X(58), 2.0, texts("label-illustrative")[0], fontsize=S_TOP, fontweight="bold", color=grey, va="center")
    ax.text(X(58), 5.3, texts("marker-inir")[0], fontsize=S_TOP, color="#2f86c8", va="center")
    ax.add_patch(Polygon([(X(58), 7.3), (X(82), 7.3), (X(70), 9.4)], closed=True, color="#2f86c8", linewidth=0))
    x_htr0, x_htr1 = gates[-1], years[-1][0]
    ax.plot([X(x_htr0), X(x_htr0), X(x_htr1), X(x_htr1)], [5.2, 4.1, 4.1, 5.2], color="#88929a", linewidth=0.6)
    ax.text(X(x_htr1), 2.0, texts("htr-pm-reference")[0], fontsize=S_TOP, color=grey, ha="right", va="center")
    for pts, color in links:
        ax.add_patch(Polygon([(X(x), strip_y + 1.3 if y < 200 else chev_top) for x, y in pts], closed=True,
                             color=color, alpha=0.13, linewidth=0))
    for x, w, color in strip:
        ax.add_patch(plt.Rectangle((X(x), strip_y), X(w), 1.3, color=color, linewidth=0))
    for x in gates:
        ax.add_patch(Polygon([(X(x), strip_y - 0.7), (X(x) + 1.25, strip_y + 0.65), (X(x), strip_y + 2.0),
                              (X(x) - 1.25, strip_y + 0.65)], closed=True, facecolor=SURFACE, edgecolor=dark,
                             linewidth=0.7, zorder=4))
        ax.text(X(x), 8.1, texts("decision-gates")[0], fontsize=S_TOP, color=grey, ha="center", va="center")
    ax.text(X(1914), 8.1, texts("marker-target")[0], fontsize=S_TOP, color=grey, ha="right", va="center")
    ax.plot([X(years[-1][0])], [strip_y + 0.65], marker="*", markersize=8, color="#f2b705", markeredgecolor="#a87e00",
            markeredgewidth=0.5, zorder=5)
    for x, label, anchor in years:
        ax.text(X(x), 14.0, label, fontsize=S_YEAR, fontweight="bold", color=dark, va="center",
                ha={"start": "left", "middle": "center", "end": "right"}[anchor])
    for ph in phases:
        xs = ph["xs"]
        if len(xs) == 5:
            shape = [(xs[0], 0), (xs[1], 0), (xs[2], 0.5), (xs[3], 1), (xs[4], 1)]
            left = X(xs[0]) + 1.8
        else:
            shape = [(xs[0], 0), (xs[1], 0), (xs[2], 0.5), (xs[3], 1), (xs[4], 1), (xs[5], 0.5)]
            left = X(xs[5]) + 1.6
        ax.add_patch(Polygon([(X(x), chev_top + f * chev_h) for x, f in shape], closed=True, color=ph["color"], linewidth=0))
        ax.text(left, chev_top + 2.3, ph["dates"], fontsize=S_DATE, color=SURFACE, alpha=0.92, va="center")
        for j, line in enumerate(ph["title_lines"]):
            ax.text(left, chev_top + 5.3 + j * tpitch, line, fontsize=S_TITLE, fontweight="bold", color=SURFACE, va="center")
        bx, bw = ph["box"]
        ax.add_patch(FancyBboxPatch((X(bx), box_top), X(bw), box_h, boxstyle="round,pad=0,rounding_size=0.8",
                                    facecolor=ph["fill"], edgecolor="none"))
        ax.add_patch(plt.Rectangle((X(bx), box_top), 0.5, box_h, color=ph["color"], linewidth=0))
        tx = X(bx) + 1.8
        y = box_top + 2.6
        ax.text(tx, y, ph["head1"], fontsize=S_HEAD, fontweight="bold", color=ph["color"], va="center")
        y += 3.3
        for line in ph["lead_lines"]:
            ax.text(tx, y, line, fontsize=S_BODY, color=dark, va="center")
            y += pitch
        y += 0.4
        ax.plot([tx, X(bx + bw) - 1.5], [y, y], color=ph["color"], linewidth=0.5, alpha=0.5)
        y += 2.6
        ax.text(tx, y, ph["head2"], fontsize=S_HEAD, fontweight="bold", color=ph["color"], va="center")
        y += 3.3
        for line in ph["req_lines"]:
            ax.text(tx, y, line, fontsize=S_BODY, color=dark, va="center")
            y += pitch
    r = fig.canvas.get_renderer()
    for t in ax.texts:
        bb = t.get_window_extent(r)
        if bb.x0 < -1 or bb.x1 > fig.bbox.width + 1:
            sys.exit(f"roadmap text runs off the figure: {t.get_text()!r}")
        if t.get_fontsize() < MIN_PT:
            sys.exit(f"roadmap text below {MIN_PT} pt: {t.get_text()!r}")
    save(fig, "fig_roadmap.png")


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    tok = load()
    cost_grid(tok)
    waterfall(tok, 25)
    waterfall(tok, 60)
    stepdown(tok)
    epz_schematic()
    roadmap()
    pfd_report()
    return 0


if __name__ == "__main__":
    sys.exit(main())
