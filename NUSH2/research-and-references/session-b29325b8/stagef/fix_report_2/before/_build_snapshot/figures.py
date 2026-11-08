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


def signed(v):
    return f"{v:+.1f}".replace("-", "−")


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
            ax.text(x, t["value"] + 5, t["text"], ha="center", va="bottom", fontsize=7.5, color=INK, zorder=4)
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
    fig.legend(loc="upper center", bbox_to_anchor=(0.54, 1.0), ncol=2, frameon=False, fontsize=7.8,
               handlelength=1.6, columnspacing=1.4, handletextpad=0.4)
    save(fig, "fig_cost_grid.png")


def waterfall(tok, life):
    """Each ledger line as its step in S$/t, with L3 and L5 as cumulative rows."""
    key = f"BOAK_{life}Y_N1P00"
    rows, cum = [], 0.0
    for level, line, name in LEDGER:
        end = tok[f"CUM_{line}_{key}"]["value"]
        rows.append((level, name, cum, end, False))
        cum = end
        if line in ("DECOM", "BASE_POWER"):
            token = ("AC_L3_" if line == "DECOM" else "AC_L5_") + key
            if abs(end - tok[token]["value"]) > 1e-3:
                sys.exit(f"ledger order no longer closes on {token}")
            rows.append(("", TOTAL_LABELS[line], 0.0, end, tok[token]["text"]))
    fig, ax = axes(150, 92)
    fig.subplots_adjust(left=0.36, right=0.97, top=0.9, bottom=0.1)
    low = min(min(r[2], r[3]) for r in rows)
    high = max(max(r[2], r[3]) for r in rows)
    span = high - low
    seen = set()
    for i, (level, name, start, end, total) in enumerate(rows):
        color = TOTAL if total else (ADDS if end >= start else CUTS)
        ax.barh(i, end - start, left=start, height=0.64, color=color, linewidth=0, zorder=3)
        if i:
            ax.plot([rows[i - 1][3]] * 2, [i - 1 + 0.32, i - 0.32], color=INK2, linewidth=0.5, zorder=2)
        ax.text(max(start, end) + span * 0.012, i, total or signed(end - start), va="center", ha="left", fontsize=7.5,
                fontweight="bold" if total else "normal", zorder=6,
                bbox=dict(boxstyle="square,pad=0.06", facecolor=SURFACE, edgecolor="none"))
        if level and level not in seen:
            seen.add(level)
            ax.annotate(level, xy=(0, i), xycoords=("axes fraction", "data"), xytext=(-150, 0), textcoords="offset points",
                        va="center", fontsize=7.5, color=INK2, fontweight="bold")
    ax.set_yticks(range(len(rows)))
    ax.set_yticklabels([r[1] for r in rows], fontsize=7.8, color=INK)
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
            fontsize=7.5, color=INK)
    ax.set_xlabel(f"S$/tCO₂e, BOAK, {life} y, n = 1.0, {tok['R_RATED_MWTH']['value']:.0f} MWth charged", fontsize=8)
    fig.legend(handles=[Patch(color=ADDS, label="Adds cost"), Patch(color=CUTS, label="Cuts cost"),
                        Patch(color=TOTAL, label="Cumulative")],
               loc="upper left", bbox_to_anchor=(0.01, 0.995), ncol=3, frameon=False, fontsize=7.8,
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
                ax.text(x, t["value"] - 22, t["text"], ha="center", va="bottom", fontsize=7.8, color=SURFACE,
                        fontweight="bold", zorder=8)
            else:
                ax.text(x, t["value"] + 4, t["text"], ha="center", va="bottom", fontsize=7.8, color=INK, zorder=4)
            top = max(top, t["value"])
    ax.axhline(target, color=INK, linewidth=1.0, linestyle=(0, (5, 3)), zorder=5)
    ax.text(4.1, target + 3, f"S${target:.0f}/t target", ha="right", va="bottom", fontsize=7.5, color=INK, zorder=6)
    ax.plot([3 - width - 0.02, 3 + width + 0.02], [point, point], color=INK, linewidth=1.0, linestyle=(0, (1, 1.5)), zorder=6)
    ax.plot([3 + width + 0.02], [point], marker="o", markersize=4, color=INK, zorder=7)
    ax.text(4.1, point - 4, f"S${point:.0f}/t\nsolve point", ha="right", va="top", fontsize=7.5, color=INK, zorder=7)
    ax.set_xticks(range(4))
    ax.set_xticklabels(labels, fontsize=7.8, color=INK)
    ax.set_xlim(-0.5, 4.12)
    ax.set_ylim(0, top * 1.12)
    ax.yaxis.grid(True, color=GRID, linewidth=0.6)
    ax.axhline(0, color=INK2, linewidth=0.8, zorder=2)
    ax.set_ylabel("Abatement cost, S$/tCO₂e", fontsize=8.5)
    fig.legend(loc="upper center", bbox_to_anchor=(0.54, 1.0), ncol=2, frameon=False, fontsize=7.8,
               handlelength=1.0, columnspacing=1.6, handletextpad=0.4)
    save(fig, "fig_stepdown.png")


PRES_FIG = B.parents[1] / "presentation" / "figures"
SVG_NS = "{http://www.w3.org/2000/svg}"
MIN_PT = 6.5


def pfd_report(width_mm=255.0):
    """Figure 2 for its landscape page: the deck drawing as it stands, with any label below 6.5 pt at
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
    raised = re.compile(r'<text x="([\d.]+)"([^>]*?) font-size="(' + "|".join(f"{x:g}" for x in small) + r')"') if small else None

    def raise_labels(shift):
        if not small:
            return svg
        return raised.sub(lambda m: f'<text x="{float(m[1]) + shift:g}"{m[2]} font-size="{need}"', svg)

    out, mark = MEDIA / "fig_pfd_report.png", MEDIA / "fig_pfd_report.sha"
    stamp = hashlib.sha256((svg + str(width_mm)).encode("utf-8")).hexdigest()
    if out.exists() and mark.exists() and mark.read_text() == stamp:
        print(f"chart: {out.name} unchanged")
        return
    out_svg = raise_labels(0)
    if small:
        # A raised label may touch a mark beside it; nudge the raised labels sideways a few pixels
        # until the drawing has no overlap it did not already have.
        before = set(bf.collisions(svg)["hits"])
        for shift in (0, 2, 4, 6, -2, -4, -6):
            out_svg = raise_labels(shift)
            extra = set(bf.collisions(out_svg)["hits"]) - before
            if not extra:
                break
        if extra:
            sys.exit(f"raising the small labels of Figure 2 adds overlaps: {sorted(extra)[:6]}")
        print(f"Figure 2 labels {small} px raised to {need} px, moved {shift:+d} px")
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
    ax.text(1.4e-3, 2.2e-6, "Schematic, no values", fontsize=7, color=INK2, ha="left", style="italic", zorder=6)
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
    """Figure 4 at its printed width with every label at 6.5 pt or more. The words, dates, colours and
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

    S_TOP, S_YEAR, S_DATE, S_TITLE, S_HEAD, S_BODY = 7.0, 7.5, 7.0, 7.6, 6.6, 7.0
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
    box_h = max(4.0 + pitch * len(p["lead_lines"]) + 2.2 + 3.8 + pitch * len(p["req_lines"]) + 1.0 for p in phases)
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
    ax.text(X(58), 2.4, texts("label-illustrative")[0], fontsize=S_TOP, fontweight="bold", color=grey, va="center")
    ax.text(X(58), 5.6, texts("marker-inir")[0], fontsize=S_TOP, color="#2f86c8", va="center")
    ax.add_patch(Polygon([(X(58), 7.3), (X(82), 7.3), (X(70), 9.4)], closed=True, color="#2f86c8", linewidth=0))
    x_htr0, x_htr1 = gates[-1], years[-1][0]
    ax.plot([X(x_htr0), X(x_htr0), X(x_htr1), X(x_htr1)], [5.2, 4.1, 4.1, 5.2], color="#88929a", linewidth=0.6)
    ax.text(X(x_htr1), 2.4, texts("htr-pm-reference")[0], fontsize=S_TOP, color=grey, ha="right", va="center")
    for pts, color in links:
        ax.add_patch(Polygon([(X(x), strip_y + 1.3 if y < 200 else chev_top) for x, y in pts], closed=True,
                             color=color, alpha=0.13, linewidth=0))
    for x, w, color in strip:
        ax.add_patch(plt.Rectangle((X(x), strip_y), X(w), 1.3, color=color, linewidth=0))
    for x in gates:
        ax.add_patch(Polygon([(X(x), strip_y - 0.7), (X(x) + 1.25, strip_y + 0.65), (X(x), strip_y + 2.0),
                              (X(x) - 1.25, strip_y + 0.65)], closed=True, facecolor=SURFACE, edgecolor=dark,
                             linewidth=0.7, zorder=4))
        ax.text(X(x), 7.9, texts("decision-gates")[0], fontsize=S_TOP, color=grey, ha="center", va="center")
    ax.text(X(1914), 7.9, texts("marker-target")[0], fontsize=S_TOP, color=grey, ha="right", va="center")
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
        y += 3.0
        for line in ph["lead_lines"]:
            ax.text(tx, y, line, fontsize=S_BODY, color=dark, va="center")
            y += pitch
        y += 0.4
        ax.plot([tx, X(bx + bw) - 1.5], [y, y], color=ph["color"], linewidth=0.5, alpha=0.5)
        y += 2.6
        ax.text(tx, y, ph["head2"], fontsize=S_HEAD, fontweight="bold", color=ph["color"], va="center")
        y += 3.0
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
