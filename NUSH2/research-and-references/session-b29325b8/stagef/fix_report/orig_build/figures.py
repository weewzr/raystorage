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
import os
import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.patches import Patch  # noqa: E402

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
LEDGER = (("L0", "GAS", "Gas saved"), ("L0", "REACTOR_CAP", "Reactor capital"),
          ("L0", "CAPTURE_CAP", "Capture capital"), ("L0", "CAPTURE_FOM", "Capture fixed O&M"),
          ("L0", "TS", "CO₂ transport and storage"), ("L1", "REACTOR_OM", "Reactor O&M and fuel"),
          ("L2", "IHX", "IHX and secondary loop"), ("L3", "LICENCE", "Extra licensing"),
          ("L3", "DECOM", "Decommissioning"), ("L4", "IDC", "Interest during construction"),
          ("L4", "POWER_ISLAND", "Power island"), ("L4", "RECYCLE_TRAIN", "Recycle and cryogenic train"),
          ("L5", "LOST_STEAM", "Lost steam export"), ("L5", "BASE_POWER", "Baseline purchased power avoided"))


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
            rows.append(("", "L3 named step" if line == "DECOM" else "L5 full cost", 0.0, end, tok[token]["text"]))
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
    fig, ax = axes(136, 80)
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


def main():
    tok = load()
    cost_grid(tok)
    waterfall(tok, 25)
    waterfall(tok, 60)
    stepdown(tok)
    return 0


if __name__ == "__main__":
    sys.exit(main())
