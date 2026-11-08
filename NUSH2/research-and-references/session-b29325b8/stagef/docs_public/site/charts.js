// NUS-H2 charts: inline SVG, a hover and focus layer, and a table view for
// every chart. Colours are the dataviz reference palette, light mode, unchanged
// (tokens --viz-* in views.css). This file places marks for values the caller
// passes from the engine reply or from CI JSON. It holds no model arithmetic:
// the only sums here are bar positions on screen, and a subtotal printed on a
// waterfall is always the caller's engine value.

const NS = "http://www.w3.org/2000/svg";
export const SLOTS = ["--viz-s1", "--viz-s2", "--viz-s3", "--viz-s4", "--viz-s5", "--viz-s6", "--viz-s7", "--viz-s8"];
export const slot = (i) => `var(${SLOTS[i % SLOTS.length]})`;
const CHAR_PX = 6.9;
// Text class of a label printed inside a bar of each slot, so it keeps 4.5:1
// on the fill: dark ink where it reaches that, white where white does, and
// white with a dark halo on the blue of slot 1, where neither does alone.
const INLABEL = ["viz-inlabel viz-inlabel-halo", "viz-inlabel viz-inlabel-dark", "viz-inlabel viz-inlabel-dark", "viz-inlabel viz-inlabel-dark",
  "viz-inlabel viz-inlabel-dark", "viz-inlabel", "viz-inlabel", "viz-inlabel viz-inlabel-dark"];
const tableOpen = new Map();

// ------------------------------------------------------------ small helpers

function el(tag, attrs, parent) {
  const n = document.createElementNS(NS, tag);
  for (const [k, v] of Object.entries(attrs || {})) {
    if (v !== undefined && v !== null) n.setAttribute(k, String(v));
  }
  if (parent) parent.appendChild(n);
  return n;
}

function svgText(parent, x, y, str, attrs) {
  const t = el("text", { x, y, ...(attrs || {}) }, parent);
  t.textContent = str;
  return t;
}

function textWidth(str) {
  return String(str).length * CHAR_PX;
}

const finite = (v) => typeof v === "number" && Number.isFinite(v);

/** Round an axis domain out to clean ticks. Axis scaling only. */
export function niceScale(lo, hi, count = 5) {
  if (!finite(lo) || !finite(hi)) { lo = 0; hi = 1; }
  if (lo > hi) [lo, hi] = [hi, lo];
  if (lo === hi) {
    const pad = Math.abs(lo) * 0.1 || 1;
    lo -= pad;
    hi += pad;
  }
  const raw = (hi - lo) / count;
  const mag = 10 ** Math.floor(Math.log10(raw));
  const norm = raw / mag;
  const step = (norm <= 1 ? 1 : norm <= 2 ? 2 : norm <= 2.5 ? 2.5 : norm <= 5 ? 5 : 10) * mag;
  const nlo = Math.floor(lo / step + 1e-9) * step;
  const nhi = Math.ceil(hi / step - 1e-9) * step;
  const ticks = [];
  for (let i = 0, v = nlo; v <= nhi + step * 1e-6 && i < 50; i += 1, v = nlo + i * step) {
    ticks.push(Math.abs(v) < step * 1e-6 ? 0 : v);
  }
  return { lo: nlo, hi: nhi, step, ticks };
}

function decimalsFor(step) {
  for (let d = 0; d <= 4; d += 1) {
    const s = step * 10 ** d;
    if (Math.abs(Math.round(s) - s) < 1e-6 * Math.max(1, s)) return d;
  }
  return 4;
}

/** Tick text, compact from 10,000 up. Display formatting only. */
export function tickText(v, step) {
  const a = Math.abs(v);
  const opts = (d) => ({ minimumFractionDigits: 0, maximumFractionDigits: d });
  if (a >= 1e6) return `${(v / 1e6).toLocaleString("en-GB", opts(decimalsFor(step / 1e6)))}M`;
  if (a >= 1e4) return `${(v / 1e3).toLocaleString("en-GB", opts(decimalsFor(step / 1e3)))}K`;
  return v.toLocaleString("en-GB", opts(decimalsFor(step)));
}

// ------------------------------------------------------------ tooltip

let tip = null;

function tipEl() {
  if (!tip) {
    tip = document.createElement("div");
    tip.className = "viz-tip";
    tip.setAttribute("role", "tooltip");
    tip.hidden = true;
    document.body.appendChild(tip);
  }
  return tip;
}

/** Show the shared tooltip. Rows are {value, label, color}; text goes in by textContent. */
export function showTip(x, y, title, rows) {
  const t = tipEl();
  t.replaceChildren();
  if (title) {
    const h = document.createElement("div");
    h.className = "tip-title";
    h.textContent = title;
    t.appendChild(h);
  }
  for (const r of rows || []) {
    const d = document.createElement("div");
    d.className = "tip-row";
    if (r.color) {
      const k = document.createElement("span");
      k.className = "tip-key";
      k.style.background = r.color;
      d.appendChild(k);
    }
    const v = document.createElement("strong");
    v.textContent = r.value;
    d.appendChild(v);
    if (r.label) {
      const l = document.createElement("span");
      l.className = "tip-label";
      l.textContent = r.label;
      d.appendChild(l);
    }
    t.appendChild(d);
  }
  t.hidden = false;
  const w = t.offsetWidth;
  const h = t.offsetHeight;
  let left = x + 14;
  let top = y + 14;
  if (left + w > window.innerWidth - 8) left = Math.max(8, x - w - 14);
  if (top + h > window.innerHeight - 8) top = Math.max(8, y - h - 14);
  t.style.left = `${left}px`;
  t.style.top = `${top}px`;
}

export function hideTip() {
  if (tip) tip.hidden = true;
}

/** Hover and focus on one hit target. getTip returns {title, rows}. */
function bindHit(node, label, getTip, group) {
  node.setAttribute("tabindex", "0");
  node.setAttribute("role", "img");
  node.setAttribute("aria-label", label);
  node.classList.add("viz-hit");
  const on = (x, y) => {
    if (group) group.classList.add("hot");
    const { title, rows } = getTip();
    showTip(x, y, title, rows);
  };
  const off = () => {
    if (group) group.classList.remove("hot");
    hideTip();
  };
  node.addEventListener("pointermove", (e) => on(e.clientX, e.clientY));
  node.addEventListener("pointerleave", off);
  node.addEventListener("focus", () => {
    const r = node.getBoundingClientRect();
    on(r.left + Math.min(r.width, 160), r.top + r.height / 2);
  });
  node.addEventListener("blur", off);
}

// ------------------------------------------------------------ frame, legend, table

/**
 * Build a chart container in `mount`: title, Live or Precomputed tag, a table
 * toggle, legend row, plot, table and note. Returns the parts.
 */
export function frame(mount, o) {
  const fig = document.createElement("figure");
  fig.className = "viz";
  fig.dataset.chart = o.key;
  const cap = document.createElement("figcaption");
  cap.className = "viz-head";
  const title = document.createElement("span");
  title.className = "viz-title";
  title.textContent = o.title;
  cap.appendChild(title);
  if (o.tag) {
    const tag = document.createElement("span");
    tag.className = "viz-tag";
    tag.dataset.tag = o.tag.toLowerCase();
    tag.textContent = o.tag;
    cap.appendChild(tag);
  }
  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = "viz-toggle";
  cap.appendChild(btn);
  fig.appendChild(cap);
  if (o.sub) {
    const sub = document.createElement("p");
    sub.className = "viz-sub";
    sub.textContent = o.sub;
    fig.appendChild(sub);
  }
  const legend = document.createElement("div");
  legend.className = "viz-legend";
  const plot = document.createElement("div");
  plot.className = "viz-plot";
  const tablebox = document.createElement("div");
  tablebox.className = "viz-table table-scroll";
  const note = document.createElement("p");
  note.className = "viz-note";
  fig.append(legend, plot, tablebox, note);
  mount.appendChild(fig);
  tablebox.id = `viz-${o.key}-table`;
  btn.setAttribute("aria-controls", tablebox.id);
  const apply = () => {
    const open = Boolean(tableOpen.get(o.key));
    btn.textContent = open ? "Show chart" : "Show table";
    btn.setAttribute("aria-pressed", String(open));
    plot.hidden = open;
    legend.hidden = open || !legend.childElementCount;
    tablebox.hidden = !open;
  };
  btn.addEventListener("click", () => {
    tableOpen.set(o.key, !tableOpen.get(o.key));
    apply();
  });
  const f = {
    fig, plot, legend, tablebox, note, apply,
    width: () => Math.max(280, Math.floor(plot.clientWidth || fig.clientWidth || o.width || 640)),
  };
  if (o.note) note.textContent = o.note;
  return f;
}

/** A chart the data cannot draw: title, tag and the stated reason. */
export function unavailable(mount, o) {
  const fig = document.createElement("figure");
  fig.className = "viz viz-na";
  fig.dataset.chart = o.key;
  fig.dataset.state = "na";
  const cap = document.createElement("figcaption");
  cap.className = "viz-head";
  const title = document.createElement("span");
  title.className = "viz-title";
  title.textContent = o.title;
  cap.appendChild(title);
  if (o.tag) {
    const tag = document.createElement("span");
    tag.className = "viz-tag";
    tag.dataset.tag = o.tag.toLowerCase();
    tag.textContent = o.tag;
    cap.appendChild(tag);
  }
  fig.appendChild(cap);
  const p = document.createElement("p");
  p.className = "viz-reason";
  const s = document.createElement("strong");
  s.textContent = "n/a. ";
  p.appendChild(s);
  p.appendChild(document.createTextNode(o.reason));
  fig.appendChild(p);
  mount.appendChild(fig);
  return fig;
}

function legendRow(f, items) {
  f.legend.replaceChildren();
  if (items.length < 2) return;
  for (const it of items) {
    const span = document.createElement("span");
    span.className = "viz-key";
    const sw = document.createElement("span");
    sw.className = `viz-swatch viz-swatch-${it.kind || "rect"}`;
    sw.style.background = it.color;
    span.appendChild(sw);
    span.appendChild(document.createTextNode(it.name));
    f.legend.appendChild(span);
  }
}

/** Fill the table view. columns: [{label, num}], rows: arrays of display strings. */
export function fillTable(f, columns, rows) {
  const table = document.createElement("table");
  const thead = document.createElement("thead");
  const hr = document.createElement("tr");
  for (const c of columns) {
    const th = document.createElement("th");
    th.scope = "col";
    th.textContent = c.label;
    if (c.num) th.className = "num";
    hr.appendChild(th);
  }
  thead.appendChild(hr);
  table.appendChild(thead);
  const tb = document.createElement("tbody");
  for (const r of rows) {
    const tr = document.createElement("tr");
    r.forEach((cell, i) => {
      const td = document.createElement("td");
      td.textContent = cell;
      if (columns[i] && columns[i].num) td.className = "num";
      tr.appendChild(td);
    });
    tb.appendChild(tr);
  }
  table.appendChild(tb);
  f.tablebox.replaceChildren(table);
}

function finish(f, svg, label) {
  svg.setAttribute("role", "group");
  svg.setAttribute("aria-label", label);
  rove(svg);
  f.plot.replaceChildren(svg);
  f.apply();
}

/**
 * One tab stop per chart: the first mark takes focus from Tab and the arrow
 * keys, Home and End move between marks, so a page of charts does not hold
 * one stop per bar.
 */
function rove(svg) {
  const hits = Array.from(svg.querySelectorAll(".viz-hit"));
  if (hits.length < 2) return;
  hits.forEach((h, i) => h.setAttribute("tabindex", i === 0 ? "0" : "-1"));
  svg.setAttribute("aria-label", `${svg.getAttribute("aria-label")}. Use the arrow keys to move between marks.`);
  svg.addEventListener("keydown", (e) => {
    const i = hits.indexOf(document.activeElement);
    if (i < 0) return;
    let j = null;
    if (e.key === "ArrowDown" || e.key === "ArrowRight") j = Math.min(hits.length - 1, i + 1);
    else if (e.key === "ArrowUp" || e.key === "ArrowLeft") j = Math.max(0, i - 1);
    else if (e.key === "Home") j = 0;
    else if (e.key === "End") j = hits.length - 1;
    if (j === null) return;
    e.preventDefault();
    hits[i].setAttribute("tabindex", "-1");
    hits[j].setAttribute("tabindex", "0");
    hits[j].focus();
  });
}

// ------------------------------------------------------------ shared horizontal layout

/** Bar path, square at the baseline end xa, 4 px rounded at the data end xb. */
function barPath(xa, xb, y, h, r = 4) {
  const w = Math.abs(xb - xa);
  if (w < 0.25) return "";
  const rr = Math.min(r, w, h / 2);
  if (xb >= xa) {
    return `M${xa},${y}H${xb - rr}Q${xb},${y} ${xb},${y + rr}V${y + h - rr}Q${xb},${y + h} ${xb - rr},${y + h}H${xa}Z`;
  }
  return `M${xa},${y}H${xb + rr}Q${xb},${y} ${xb},${y + rr}V${y + h - rr}Q${xb},${y + h} ${xb + rr},${y + h}H${xa}Z`;
}

/**
 * Rows of horizontal marks with the row label above each bar, so long labels
 * never collide at phone width. The value axis always includes zero.
 */
function hFrame(f, o) {
  const W = f.width();
  const values = o.values.filter(finite);
  const lo0 = Math.min(0, ...values, ...(o.extra || []).filter(finite));
  const hi0 = Math.max(0, ...values, ...(o.extra || []).filter(finite));
  const sc = niceScale(lo0, hi0, W < 480 ? 4 : 6);
  const padL = Math.min(W * 0.3, Math.max(10, o.padLeft || 0));
  const padR = Math.min(W * 0.35, Math.max(16, o.padRight || 0));
  const barH = Math.min(24, o.barH || 18);
  const labelH = 17;
  const rowH = labelH + barH + (o.rowGap ?? 12);
  const top = o.top ?? 6;
  const plotH = o.rows * rowH;
  const H = top + plotH + 30;
  const svg = el("svg", { viewBox: `0 0 ${W} ${H}`, width: W, height: H, class: "viz-svg" });
  const x = (v) => padL + ((v - sc.lo) / (sc.hi - sc.lo)) * (W - padL - padR);
  const grid = el("g", { class: "viz-grid" }, svg);
  for (const t of sc.ticks) {
    const gx = x(t);
    el("line", { x1: gx, x2: gx, y1: top, y2: top + plotH, class: t === 0 ? "viz-base" : "viz-gridline" }, grid);
    svgText(grid, gx, top + plotH + 16, tickText(t, sc.step), { "text-anchor": "middle", class: "viz-tick" });
  }
  if (o.unit) svgText(grid, W - 2, top + plotH + 29, o.unit, { "text-anchor": "end", class: "viz-axis-title" });
  const rowY = (i) => top + i * rowH;
  const barY = (i) => rowY(i) + labelH;
  return { svg, x, sc, W, H, top, rowH, barH, labelH, rowY, barY, plotH, padL, padR };
}

/** Row label on a surface-coloured box, so grid and reference lines drawn earlier stop at the text. */
function rowLabel(L, i, label) {
  el("rect", { x: 0, y: L.rowY(i), width: Math.min(L.W, textWidth(label) + 8), height: 16, class: "viz-labelbox" }, L.svg);
  svgText(L.svg, 2, L.rowY(i) + 12, label, { class: "viz-rowlabel" });
}

function valueLabel(L, xEnd, i, str, positive) {
  const y = L.barY(i) + L.barH / 2 + 4;
  return svgText(L.svg, positive ? xEnd + 6 : xEnd - 6, y, str, {
    "text-anchor": positive ? "start" : "end",
    class: "viz-value",
  });
}

/** Vertical reference line across a horizontal layout, label at the top. */
function vRef(L, ref) {
  if (!ref || !finite(ref.value)) return;
  const rx = L.x(ref.value);
  el("line", { x1: rx, x2: rx, y1: L.top - 2, y2: L.top + L.plotH, class: "viz-ref" }, L.svg);
  const right = rx + textWidth(ref.label) + 8 < L.W;
  svgText(L.svg, right ? rx + 4 : rx - 4, L.top - 5, ref.label, { "text-anchor": right ? "start" : "end", class: "viz-reflabel" });
}

// ------------------------------------------------------------ single-series bars

/**
 * One bar per row from zero. rows: [{label, value, text, note}]. ref: {value,
 * label} draws a reference line. A null value draws no bar and prints n/a.
 */
export function barChart(f, o) {
  const fmtv = o.fmt || ((v) => String(v));
  const texts = o.rows.map((r) => (finite(r.value) ? (r.text || fmtv(r.value)) : "n/a"));
  const pad = Math.max(...texts.map(textWidth), 0) + 12;
  const anyNeg = o.rows.some((r) => finite(r.value) && r.value < 0);
  const L = hFrame(f, { values: o.rows.map((r) => r.value), extra: o.ref ? [o.ref.value] : [], rows: o.rows.length, unit: o.unit, padRight: pad, padLeft: anyNeg ? pad : 0, barH: o.barH, top: o.ref ? 16 : 6 });
  const color = slot(o.slot ?? 0);
  vRef(L, o.ref);
  o.rows.forEach((r, i) => {
    rowLabel(L, i, r.label);
    const grp = el("g", {}, L.svg);
    if (finite(r.value)) {
      const d = barPath(L.x(0), L.x(r.value), L.barY(i), L.barH);
      if (d) el("path", { d, class: "viz-mark", style: `fill:${color}` }, grp);
      else el("line", { x1: L.x(0), x2: L.x(0), y1: L.barY(i), y2: L.barY(i) + L.barH, class: "viz-zero-mark" }, grp);
    }
    const up = !finite(r.value) || r.value >= 0;
    valueLabel(L, L.x(finite(r.value) ? r.value : 0), i, texts[i], up);
    const hit = el("rect", { x: 0, y: L.rowY(i), width: L.W, height: L.rowH, fill: "transparent" }, grp);
    bindHit(hit, `${r.label}: ${texts[i]}`, () => ({ title: r.label, rows: [{ value: texts[i], label: r.note || o.valueName || "", color }] }), grp);
  });
  legendRow(f, []);
  fillTable(f, [{ label: o.rowHead || "Item" }, { label: o.valueHead || `Value, ${o.unit}`, num: true }, ...(o.extraHead ? [{ label: o.extraHead }] : [])],
    o.rows.map((r, i) => [r.label, texts[i], ...(o.extraHead ? [r.note || ""] : [])]));
  if (o.note) f.note.textContent = o.note;
  finish(f, L.svg, o.label || "Bar chart");
}

// ------------------------------------------------------------ stacked bar

/**
 * Horizontal stacked bars. series: [{name, slot}] in a fixed order, so a
 * colour follows its entity. rows: [{label, values: [number|null per series],
 * totalText}]. The printed total is the caller's value.
 */
export function stackedBars(f, o) {
  const fmtv = o.fmt || ((v) => String(v));
  const ends = [];
  for (const r of o.rows) {
    let run = 0;
    for (const v of r.values) if (finite(v) && v > 0) run += v;
    ends.push(run);
  }
  const totalTexts = o.rows.map((r) => r.totalText || "");
  const padRight = Math.max(...totalTexts.map(textWidth), 0) + 12;
  const L = hFrame(f, { values: ends, rows: o.rows.length, unit: o.unit, padRight, barH: o.barH });
  const used = new Set();
  o.rows.forEach((r, i) => {
    rowLabel(L, i, r.label);
    const segs = [];
    let run = 0;
    r.values.forEach((v, s) => {
      if (!finite(v) || v <= 0) return;
      segs.push({ s, a: run, b: run + v, v });
      run += v;
      used.add(s);
    });
    segs.forEach((g, k) => {
      const xa = L.x(g.a) + (k > 0 ? 1 : 0);
      const xb = L.x(g.b) - (k < segs.length - 1 ? 1 : 0);
      const grp = el("g", {}, L.svg);
      const sl = o.series[g.s].slot ?? g.s;
      const d = k === segs.length - 1 ? barPath(xa, xb, L.barY(i), L.barH) : `M${xa},${L.barY(i)}H${xb}V${L.barY(i) + L.barH}H${xa}Z`;
      if (d && xb > xa) el("path", { d, class: "viz-mark", style: `fill:${slot(sl)}` }, grp);
      const segW = xb - xa;
      const txt = fmtv(g.v);
      if (o.inlineLabels && segW > textWidth(txt) + 10) {
        svgText(grp, (xa + xb) / 2, L.barY(i) + L.barH / 2 + 4, txt, {
          "text-anchor": "middle",
          class: INLABEL[sl % INLABEL.length],
        });
      }
      const hit = el("rect", { x: Math.min(xa, xb) - 1, y: L.rowY(i), width: Math.max(4, Math.abs(xb - xa) + 2), height: L.labelH + L.barH + 4, fill: "transparent" }, grp);
      bindHit(hit, `${r.label}, ${o.series[g.s].name}: ${txt}`, () => ({
        title: r.label,
        rows: [{ value: txt, label: o.series[g.s].name, color: slot(sl) }],
      }), grp);
    });
    if (r.totalText) valueLabel(L, L.x(run), i, r.totalText, true);
  });
  legendRow(f, o.series.map((s, k) => ({ s, k })).filter(({ k }) => used.has(k)).map(({ s, k }) => ({ name: s.name, color: slot(s.slot ?? k) })));
  const hasTotal = o.rows.some((r) => r.totalText);
  const cols = [{ label: o.rowHead || "Row" }, ...o.series.map((s) => ({ label: s.name, num: true })), ...(hasTotal ? [{ label: o.totalHead || "Total", num: true }] : [])];
  fillTable(f, cols, o.rows.map((r) => [r.label, ...r.values.map((v) => (finite(v) ? fmtv(v) : "n/a")), ...(hasTotal ? [r.totalText || ""] : [])]));
  if (o.note) f.note.textContent = o.note;
  finish(f, L.svg, o.label || "Stacked bar chart");
}

// ------------------------------------------------------------ waterfall

/**
 * Horizontal waterfall. steps: [{label, value, kind: "delta"|"total",
 * subtract, text, note}]. A delta moves the running position by its value, or
 * back by it when `subtract` is set (a "less" line whose engine value is
 * positive). A total bar runs from zero to the caller's engine value and
 * resets the running position, so a printed subtotal is never a sum made
 * here. When the bars before a total do not reach it, the note says so.
 */
export function waterfall(f, o) {
  const fmtv = o.fmt || ((v) => String(v));
  const pos = [];
  let run = 0;
  let moved = 0;
  const mismatch = [];
  for (const s of o.steps) {
    if (s.kind === "total") {
      // A total that follows no delta opens the chart, so there is nothing to check.
      if (moved && o.checkTotals !== false && finite(s.value) && Math.abs(run - s.value) > (o.tolerance ?? 1e-6) * Math.max(1, Math.abs(s.value))) mismatch.push(s.label);
      pos.push({ a: 0, b: finite(s.value) ? s.value : 0 });
      run = finite(s.value) ? s.value : run;
      moved = 0;
    } else {
      const v = finite(s.value) ? s.value : 0;
      const b = s.subtract ? run - v : run + v;
      pos.push({ a: run, b });
      run = b;
      moved += 1;
    }
  }
  const all = pos.flatMap((p) => [p.a, p.b]);
  const texts = o.steps.map((s) => s.text || (finite(s.value) ? fmtv(s.value) : "n/a"));
  const padRight = Math.max(...texts.map(textWidth), 0) + 12;
  const negLabels = o.steps.map((s, i) => (pos[i].b < pos[i].a ? textWidth(texts[i]) + 12 : 0));
  const L = hFrame(f, { values: all, rows: o.steps.length, unit: o.unit, padRight, padLeft: Math.max(...negLabels, 0), barH: o.barH });
  const kinds = new Set();
  const kindName = (k) => (k === "total" ? (o.totalName || "Subtotal") : k === "up" ? (o.upName || "Increase") : (o.downName || "Decrease"));
  o.steps.forEach((s, i) => {
    rowLabel(L, i, s.label);
    const p = pos[i];
    const up = p.b >= p.a;
    const kind = s.kind === "total" ? "total" : up ? "up" : "down";
    kinds.add(kind);
    const color = kind === "total" ? "var(--viz-total)" : kind === "up" ? "var(--viz-up)" : "var(--viz-down)";
    const grp = el("g", { "data-kind": kind }, L.svg);
    const d = barPath(L.x(p.a), L.x(p.b), L.barY(i), L.barH);
    if (d) el("path", { d, class: s.named ? "viz-mark viz-named" : "viz-mark", style: `fill:${color}` }, grp);
    else el("line", { x1: L.x(p.a), x2: L.x(p.a), y1: L.barY(i), y2: L.barY(i) + L.barH, class: "viz-zero-mark" }, grp);
    valueLabel(L, L.x(p.b), i, texts[i], up);
    if (i + 1 < o.steps.length && o.steps[i + 1].kind !== "total") {
      const cx = L.x(p.b);
      el("line", { x1: cx, x2: cx, y1: L.barY(i) + L.barH, y2: L.barY(i + 1), class: "viz-connector" }, L.svg);
    }
    const hit = el("rect", { x: 0, y: L.rowY(i), width: L.W, height: L.rowH, fill: "transparent" }, grp);
    bindHit(hit, `${s.label}: ${texts[i]}`, () => ({
      title: s.label,
      rows: [{ value: texts[i], label: s.note || kindName(kind), color }],
    }), grp);
  });
  legendRow(f, ["up", "down", "total"].filter((k) => kinds.has(k)).map((k) => ({
    name: kindName(k), color: k === "total" ? "var(--viz-total)" : k === "up" ? "var(--viz-up)" : "var(--viz-down)",
  })));
  fillTable(f, [{ label: o.rowHead || "Step" }, { label: o.kindHead || "Kind" }, { label: o.valueHead || `Value, ${o.unit}`, num: true }, ...(o.extraHead ? [{ label: o.extraHead, num: true }] : [])],
    o.steps.map((s, i) => [s.label, kindName(s.kind === "total" ? "total" : pos[i].b >= pos[i].a ? "up" : "down"), texts[i], ...(o.extraHead ? [s.extra || ""] : [])]));
  if (mismatch.length) {
    f.note.textContent = `${o.note ? `${o.note} ` : ""}The bars before ${mismatch.join(", ")} do not reach the engine subtotal; the subtotal shown is the engine value.`;
    f.fig.dataset.mismatch = mismatch.join("|");
  } else if (o.note) {
    f.note.textContent = o.note;
  }
  finish(f, L.svg, o.label || "Waterfall chart");
  return { mismatch };
}

// ------------------------------------------------------------ tornado (range bars)

/**
 * Bars from a base to each value. rows: [{label, segs: [{s, value, text, input}]}],
 * series: [{name, slot}]. Two series split each row into an upper and lower
 * half, so a low and a high case on the same side never hide each other. The
 * axis includes zero; the base is drawn as a reference line.
 */
export function rangeBars(f, o) {
  const fmtv = o.fmt || ((v) => String(v));
  const base = finite(o.base) ? o.base : 0;
  const vals = o.rows.flatMap((r) => r.segs.map((g) => g.value));
  const texts = o.rows.flatMap((r) => r.segs.map((g) => g.text || fmtv(g.value)));
  const pad = Math.max(...texts.map(textWidth), 0) + 12;
  const nS = o.series.length;
  const barH = nS > 1 ? 22 : 18;
  const leftPad = vals.some((v) => finite(v) && v < base) ? pad : 0;
  const L = hFrame(f, { values: [...vals, base], extra: o.ref ? [o.ref.value] : [], rows: o.rows.length, unit: o.unit, padRight: pad, padLeft: leftPad, barH, top: 16 });
  if (base !== 0) el("line", { x1: L.x(base), x2: L.x(base), y1: L.top - 2, y2: L.top + L.plotH, class: "viz-baseref" }, L.svg);
  vRef(L, o.ref);
  o.rows.forEach((r, i) => rowLabel(L, i, r.label));
  const used = new Set();
  o.rows.forEach((r, i) => {
    r.segs.forEach((g) => {
      if (!finite(g.value)) return;
      used.add(g.s);
      const h = nS > 1 ? (L.barH - 2) / 2 : L.barH;
      const y = L.barY(i) + (nS > 1 ? g.s * (h + 2) : 0);
      const color = slot(o.series[g.s].slot ?? g.s);
      const grp = el("g", {}, L.svg);
      const d = barPath(L.x(base), L.x(g.value), y, h, Math.min(4, h / 2));
      if (d) el("path", { d, class: "viz-mark", style: `fill:${color}` }, grp);
      else el("line", { x1: L.x(base), x2: L.x(base), y1: y, y2: y + h, class: "viz-zero-mark" }, grp);
      const txt = g.text || fmtv(g.value);
      const up = g.value >= base;
      svgText(L.svg, up ? L.x(g.value) + 6 : L.x(g.value) - 6, y + h / 2 + 4, txt, { "text-anchor": up ? "start" : "end", class: "viz-value viz-value-sm" });
      const hit = el("rect", { x: 0, y, width: L.W, height: h + 2, fill: "transparent" }, grp);
      bindHit(hit, `${r.label}, ${o.series[g.s].name}${g.input !== undefined ? ` at ${g.input}` : ""}: ${txt}`, () => ({
        title: r.label,
        rows: [{ value: txt, label: `${o.series[g.s].name}${g.input !== undefined ? `, input ${g.input}` : ""}`, color }],
      }), grp);
    });
  });
  const keys = o.series.map((s, k) => ({ s, k })).filter(({ k }) => used.has(k)).map(({ s, k }) => ({ name: s.name, color: slot(s.slot ?? k) }));
  if (base !== 0 && o.baseLabel) keys.push({ name: `${o.baseLabel}, ${fmtv(base)}`, color: "var(--viz-ink-2)", kind: "line" });
  legendRow(f, keys);
  const cols = [{ label: o.rowHead || "Parameter" }];
  o.series.forEach((s) => { cols.push({ label: `${s.name}, input` }); cols.push({ label: `${s.name}, ${o.unit}`, num: true }); });
  fillTable(f, cols, o.rows.map((r) => {
    const out = [r.label];
    o.series.forEach((_, s) => {
      const g = r.segs.find((x) => x.s === s);
      out.push(g && g.input !== undefined ? String(g.input) : "");
      out.push(g && finite(g.value) ? (g.text || fmtv(g.value)) : "n/a");
    });
    return out;
  }));
  if (o.note) f.note.textContent = o.note;
  finish(f, L.svg, o.label || "Tornado chart");
}

// ------------------------------------------------------------ dumbbell

/**
 * Two values per row joined by a line. series: two names, drawn in the first
 * two categorical slots. ref: {value, label} draws a vertical reference line.
 */
export function dumbbell(f, o) {
  const fmtv = o.fmt || ((v) => String(v));
  const vals = o.rows.flatMap((r) => [r.a, r.b]);
  const ends = o.rows.map((r) => `${finite(r.a) ? fmtv(r.a) : "n/a"} / ${finite(r.b) ? fmtv(r.b) : "n/a"}`);
  const L = hFrame(f, { values: vals, extra: o.ref ? [o.ref.value] : [], rows: o.rows.length, unit: o.unit, padRight: Math.max(...ends.map(textWidth), 0) + 14, barH: 14, rowGap: 10, top: o.ref ? 16 : 6 });
  const colors = [slot(0), slot(1)];
  vRef(L, o.ref);
  o.rows.forEach((r, i) => {
    rowLabel(L, i, r.label);
    const cy = L.barY(i) + L.barH / 2;
    const grp = el("g", {}, L.svg);
    if (finite(r.a) && finite(r.b)) el("line", { x1: L.x(r.a), x2: L.x(r.b), y1: cy, y2: cy, class: "viz-dumb-line" }, grp);
    [r.a, r.b].forEach((v, k) => {
      if (!finite(v)) return;
      el("circle", { cx: L.x(v), cy, r: 5, class: "viz-dot", style: `fill:${colors[k]}` }, grp);
    });
    const right = Math.max(...[r.a, r.b].filter(finite).map(L.x), L.x(0));
    svgText(L.svg, right + 10, cy + 4, ends[i], { class: "viz-value viz-value-sm" });
    const hit = el("rect", { x: 0, y: L.rowY(i), width: L.W, height: L.rowH, fill: "transparent" }, grp);
    bindHit(hit, `${r.label}: ${o.series[0]} ${finite(r.a) ? fmtv(r.a) : "n/a"}, ${o.series[1]} ${finite(r.b) ? fmtv(r.b) : "n/a"}`, () => ({
      title: r.label,
      rows: [
        { value: finite(r.a) ? fmtv(r.a) : "n/a", label: o.series[0], color: colors[0] },
        { value: finite(r.b) ? fmtv(r.b) : "n/a", label: o.series[1], color: colors[1] },
        ...(r.extra || []),
      ],
    }), grp);
  });
  legendRow(f, [{ name: o.series[0], color: colors[0], kind: "dot" }, { name: o.series[1], color: colors[1], kind: "dot" }]);
  fillTable(f, [{ label: o.rowHead || "Row" }, { label: `${o.series[0]}, ${o.unit}`, num: true }, { label: `${o.series[1]}, ${o.unit}`, num: true }, ...(o.extraCols || []).map((c) => ({ label: c, num: true }))],
    o.rows.map((r) => [r.label, finite(r.a) ? fmtv(r.a) : "n/a", finite(r.b) ? fmtv(r.b) : "n/a", ...(r.extraCells || [])]));
  if (o.note) f.note.textContent = o.note;
  finish(f, L.svg, o.label || "Dumbbell chart");
}

// ------------------------------------------------------------ line

/**
 * Line chart with a crosshair. series: [{name, slot, points: [{x, y}]}].
 * Categorical x: pass xLabels and use x = index. step: "mid" draws each point
 * as a level across its band. ref: {y, label}. A null y breaks the line.
 * zero: true keeps zero on the y axis.
 */
export function lineChart(f, o) {
  const W = f.width();
  const H = o.height || 260;
  const fy = o.fmtY || ((v) => String(v));
  const fx = o.fmtX || ((v) => String(v));
  const cat = Array.isArray(o.xLabels);
  const xs = cat ? o.xLabels.map((_, i) => i) : [...new Set(o.series.flatMap((s) => s.points.map((p) => p.x)))].sort((a, b) => a - b);
  const ys = o.series.flatMap((s) => s.points.map((p) => p.y)).filter(finite);
  if (o.ref && finite(o.ref.y)) ys.push(o.ref.y);
  if (o.zero || !ys.length) ys.push(0);
  const sc = niceScale(Math.min(...ys), Math.max(...ys), 5);
  const tickW = Math.max(...sc.ticks.map((t) => textWidth(tickText(t, sc.step))), 20);
  const padL = tickW + 10;
  const padR = 16;
  const padT = o.yTitle ? 24 : 10;
  const padB = cat ? 44 : 38;
  const svg = el("svg", { viewBox: `0 0 ${W} ${H}`, width: W, height: H, class: "viz-svg" });
  const plotW = W - padL - padR;
  const plotH = H - padT - padB;
  let xmin = cat ? -0.5 : Math.min(...xs);
  let xmax = cat ? xs.length - 0.5 : Math.max(...xs);
  if (!finite(xmin) || !finite(xmax)) { xmin = 0; xmax = 1; }
  if (xmin === xmax) { xmin -= 1; xmax += 1; }
  const x = (v) => padL + ((v - xmin) / (xmax - xmin)) * plotW;
  const y = (v) => padT + (1 - (v - sc.lo) / (sc.hi - sc.lo)) * plotH;
  const grid = el("g", { class: "viz-grid" }, svg);
  for (const t of sc.ticks) {
    el("line", { x1: padL, x2: W - padR, y1: y(t), y2: y(t), class: t === 0 ? "viz-base" : "viz-gridline" }, grid);
    svgText(grid, padL - 6, y(t) + 4, tickText(t, sc.step), { "text-anchor": "end", class: "viz-tick" });
  }
  el("line", { x1: padL, x2: W - padR, y1: padT + plotH, y2: padT + plotH, class: "viz-base" }, grid);
  if (cat) {
    const widest = Math.max(...o.xLabels.map((s) => textWidth(s)), 20);
    const every = Math.max(1, Math.ceil((widest + 8) / (plotW / Math.max(1, xs.length))));
    xs.forEach((i) => {
      if (i % every) return;
      svgText(grid, x(i), padT + plotH + 16, o.xLabels[i], { "text-anchor": "middle", class: "viz-tick" });
    });
  } else {
    const xsc = niceScale(xmin, xmax, W < 480 ? 4 : 6);
    for (const t of xsc.ticks) {
      if (t < xmin - 1e-9 || t > xmax + 1e-9) continue;
      svgText(grid, x(t), padT + plotH + 16, tickText(t, xsc.step), { "text-anchor": "middle", class: "viz-tick" });
    }
  }
  if (o.xTitle) svgText(grid, W - padR, H - 4, o.xTitle, { "text-anchor": "end", class: "viz-axis-title" });
  if (o.yTitle) svgText(grid, 2, 12, o.yTitle, { class: "viz-axis-title" });
  if (o.ref && finite(o.ref.y)) {
    el("line", { x1: padL, x2: W - padR, y1: y(o.ref.y), y2: y(o.ref.y), class: "viz-ref" }, svg);
    svgText(svg, W - padR - 2, y(o.ref.y) - 5, o.ref.label, { "text-anchor": "end", class: "viz-reflabel" });
  }
  const band = cat ? plotW / Math.max(1, xs.length) : 0;
  o.series.forEach((s, k) => {
    const color = slot(s.slot ?? k);
    let d = "";
    let pen = false;
    const pts = cat ? xs.map((i) => s.points[i] || { x: i, y: null }) : s.points;
    pts.forEach((p) => {
      if (!finite(p.y)) { pen = false; return; }
      const px = x(p.x);
      const py = y(p.y);
      if (o.step === "mid" && cat) {
        const a = px - band / 2;
        const b = px + band / 2;
        d += pen ? `V${py}H${b}` : `M${a},${py}H${b}`;
      } else {
        d += pen ? `L${px},${py}` : `M${px},${py}`;
      }
      pen = true;
    });
    if (d) el("path", { d, class: "viz-line", style: `stroke:${color}` }, svg);
    pts.forEach((p) => {
      if (!finite(p.y)) return;
      el("circle", { cx: x(p.x), cy: y(p.y), r: 4, class: "viz-dot", style: `fill:${color}` }, svg);
    });
  });
  if (o.series.length === 1) {
    const pts = o.series[0].points.filter((p) => finite(p.y));
    const last = pts[pts.length - 1];
    if (last && !cat) {
      svgText(svg, Math.min(x(last.x), W - padR) - 2, y(last.y) - 9, fy(last.y), { "text-anchor": "end", class: "viz-value viz-value-sm" });
    }
  }
  const cross = el("line", { x1: 0, x2: 0, y1: padT, y2: padT + plotH, class: "viz-cross", visibility: "hidden" }, svg);
  const overlay = el("rect", { x: padL, y: padT, width: plotW, height: plotH, fill: "transparent", class: "viz-overlay" }, svg);
  overlay.setAttribute("tabindex", "0");
  overlay.setAttribute("role", "img");
  overlay.setAttribute("aria-label", `${o.label || "Line chart"}. Use the arrow keys to step through points.`);
  let cur = -1;
  const at = (i, cx, cy) => {
    if (!xs.length) return;
    cur = Math.max(0, Math.min(xs.length - 1, i));
    const xv = xs[cur];
    cross.setAttribute("x1", x(xv));
    cross.setAttribute("x2", x(xv));
    cross.setAttribute("visibility", "visible");
    const rows = o.series.map((s, k) => {
      const p = cat ? s.points[cur] : s.points.find((q) => q.x === xv);
      return { value: p && finite(p.y) ? fy(p.y) : "n/a", label: s.name, color: slot(s.slot ?? k) };
    });
    const title = cat ? `${o.xLabels[cur]}${o.xNames ? `, ${o.xNames[cur]}` : ""}` : `${o.xName || "x"} ${fx(xv)}`;
    const r = overlay.getBoundingClientRect();
    showTip(cx ?? (r.left + ((x(xv) - padL) / plotW) * r.width), cy ?? (r.top + 20), title, rows);
  };
  const nearest = (clientX) => {
    const r = overlay.getBoundingClientRect();
    const sx = padL + ((clientX - r.left) / r.width) * plotW;
    let best = 0;
    let bd = Infinity;
    xs.forEach((v, i) => { const dd = Math.abs(x(v) - sx); if (dd < bd) { bd = dd; best = i; } });
    return best;
  };
  overlay.addEventListener("pointermove", (e) => at(nearest(e.clientX), e.clientX, e.clientY));
  overlay.addEventListener("pointerleave", () => { cross.setAttribute("visibility", "hidden"); hideTip(); });
  overlay.addEventListener("focus", () => at(cur < 0 ? 0 : cur));
  overlay.addEventListener("blur", () => { cross.setAttribute("visibility", "hidden"); hideTip(); });
  overlay.addEventListener("keydown", (e) => {
    if (e.key === "ArrowRight") { at(cur + 1); e.preventDefault(); }
    if (e.key === "ArrowLeft") { at(cur - 1); e.preventDefault(); }
  });
  legendRow(f, o.series.map((s, k) => ({ name: s.name, color: slot(s.slot ?? k), kind: "line" })));
  const cols = [{ label: o.xHead || o.xName || "x" }, ...(o.xNames ? [{ label: "Name" }] : []), ...o.series.map((s) => ({ label: s.name, num: true }))];
  fillTable(f, cols, xs.map((xv, i) => [
    cat ? o.xLabels[i] : fx(xv),
    ...(o.xNames ? [o.xNames[i]] : []),
    ...o.series.map((s) => {
      const p = cat ? s.points[i] : s.points.find((q) => q.x === xv);
      return p && finite(p.y) ? fy(p.y) : "n/a";
    }),
  ]));
  if (o.note) f.note.textContent = o.note;
  finish(f, svg, o.label || "Line chart");
}
