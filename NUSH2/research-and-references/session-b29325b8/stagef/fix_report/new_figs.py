PRES_FIG = B.parents[1] / "presentation" / "figures"
SVG_NS = "{http://www.w3.org/2000/svg}"
MIN_PT = 6.5


def pfd_report(width_mm=246.0):
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
    out_svg = svg
    for x in small:
        out_svg = out_svg.replace(f'font-size="{x:g}"', f'font-size="{need}"')
    smallest = min(float(x) for x in re.findall(r'font-size="([\d.]+)"', out_svg)) * pt_per_px
    out, mark = MEDIA / "fig_pfd_report.png", MEDIA / "fig_pfd_report.sha"
    stamp = hashlib.sha256((out_svg + str(width_mm)).encode("utf-8")).hexdigest()
    if out.exists() and mark.exists() and mark.read_text() == stamp:
        print(f"chart: {out.name} unchanged, smallest label {smallest:.2f} pt")
        return
    if small:
        before = set(bf.collisions(svg)["hits"])
        after = set(bf.collisions(out_svg)["hits"])
        if after - before:
            sys.exit(f"raising the small labels of Figure 2 adds overlaps: {sorted(after - before)[:6]}")
    bf.render_png(out_svg, out, 4000)
    mark.write_text(stamp)
    print(f"chart: {out.name}, labels {small} px raised to {need} px, smallest label {smallest:.2f} pt at {width_mm:g} mm")


def epz_schematic():
    """Figure 3: the shape of a frequency-consequence target, with no values on either axis. The
    points only place the line on the page, with the same shape and labels as the deck's schematic;
    the values and their units belong to the cited application's own figure."""
    import re
    plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 8, "text.color": INK})
    fig, ax = plt.subplots(figsize=(128 * MM, 70 * MM), dpi=DPI)
    fig.patch.set_facecolor(SURFACE)
    fig.subplots_adjust(left=0.075, right=0.985, top=0.98, bottom=0.115)
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


def roadmap(width_mm=159.0):
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

    S_TOP, S_YEAR, S_DATE, S_TITLE, S_HEAD, S_BODY = 7.0, 7.5, 7.0, 7.8, 6.6, 7.2
    plt.rcParams.update({"font.family": "DejaVu Sans"})
    probe = plt.figure(figsize=(width_mm * MM, 50 * MM), dpi=DPI)
    for ph in phases:
        inner = ph["box"][1] * k - 3.6
        ph["title_lines"] = wrap(probe, ph["title"], (ph["xs"][1] - ph["xs"][0]) * k - 5.5, S_TITLE, "bold")
        ph["lead_lines"] = wrap(probe, ph["lead"], inner, S_BODY)
        ph["req_lines"] = wrap(probe, ph["req"], inner, S_BODY)
    plt.close(probe)
    pitch = S_BODY * 1.32 * 25.4 / 72
    tpitch = S_TITLE * 1.25 * 25.4 / 72
    strip_y, chev_top = 11.2, 18.6
    chev_h = 4.6 + tpitch * max(len(p["title_lines"]) for p in phases) + 1.4
    box_top = chev_top + chev_h + 2.0
    box_h = max(4.2 + pitch * len(p["lead_lines"]) + 2.4 + 4.0 + pitch * len(p["req_lines"]) + 1.6 for p in phases)
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
    ax.text(X(58), 6.0, texts("marker-inir")[0], fontsize=S_TOP, color="#2f86c8", va="center")
    ax.add_patch(Polygon([(X(58), 8.0), (X(82), 8.0), (X(70), 10.2)], closed=True, color="#2f86c8", linewidth=0))
    x_htr0, x_htr1 = gates[-1], years[-1][0]
    ax.plot([X(x_htr0), X(x_htr0), X(x_htr1), X(x_htr1)], [5.5, 4.4, 4.4, 5.5], color="#88929a", linewidth=0.6)
    ax.text(X((x_htr0 + x_htr1) / 2), 2.4, texts("htr-pm-reference")[0], fontsize=S_TOP, color=grey, ha="center", va="center")
    for pts, color in links:
        ax.add_patch(Polygon([(X(x), strip_y + 1.3 if y < 200 else chev_top) for x, y in pts], closed=True,
                             color=color, alpha=0.13, linewidth=0))
    for x, w, color in strip:
        ax.add_patch(plt.Rectangle((X(x), strip_y), X(w), 1.3, color=color, linewidth=0))
    for x in gates:
        ax.add_patch(Polygon([(X(x), strip_y - 0.7), (X(x) + 1.25, strip_y + 0.65), (X(x), strip_y + 2.0),
                              (X(x) - 1.25, strip_y + 0.65)], closed=True, facecolor=SURFACE, edgecolor=dark,
                             linewidth=0.7, zorder=4))
        ax.text(X(x), 8.6, texts("decision-gates")[0], fontsize=S_TOP, color=grey, ha="center", va="center")
    ax.text(X(1914), 8.6, texts("marker-target")[0], fontsize=S_TOP, color=grey, ha="right", va="center")
    ax.plot([X(years[-1][0])], [strip_y + 0.65], marker="*", markersize=8, color="#f2b705", markeredgecolor="#a87e00",
            markeredgewidth=0.5, zorder=5)
    for x, label, anchor in years:
        ax.text(X(x), 15.4, label, fontsize=S_YEAR, fontweight="bold", color=dark, va="center",
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
        ax.text(left, chev_top + 2.4, ph["dates"], fontsize=S_DATE, color=SURFACE, alpha=0.92, va="center")
        for j, line in enumerate(ph["title_lines"]):
            ax.text(left, chev_top + 5.6 + j * tpitch, line, fontsize=S_TITLE, fontweight="bold", color=SURFACE, va="center")
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


