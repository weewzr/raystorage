from pathlib import Path
p = Path('frontend/tour.js'); s = p.read_text(encoding='utf-8')
def rep(old, new, count=1):
    global s
    assert s.count(old) == count, (old[:90], s.count(old))
    s = s.replace(old, new)

rep('''const n = () => STEPS.length;
const pick = (v, c) => (typeof v === "function" ? v(c) : v || []);
''', '''const n = () => STEPS.length;
const pick = (v, c) => (typeof v === "function" ? v(c) : v || []);
const titleOf = (s, c) => (typeof s.title === "function" ? s.title(c) : s.title);
const leadOf = (s, c) => (typeof s.lead === "function" ? s.lead(c) : s.lead);
''')

# applyMarks: units may depend on the scenario; the tour's own lead keeps the
# glow and drops the selection ring, which crosses labels next to some units.
rep('''  for (const id of s.units) light(`[data-unit="${CSS.escape(id)}"]`, "tour-on");''',
    '''  for (const id of pick(s.units, c)) light(`[data-unit="${CSS.escape(id)}"]`, "tour-on");''')
rep('''  for (const k of pick(s.marks, c)) light(`[data-tour="${CSS.escape(k)}"]`, "tour-on");
}''', '''  for (const k of pick(s.marks, c)) light(`[data-tour="${CSS.escape(k)}"]`, "tour-on");
  const lead = leadOf(s, c);
  if (lead) light(`[data-unit="${CSS.escape(lead)}"]`, "tour-lead");
}''')
rep('''  for (const e of el.svg.querySelectorAll(".tour-on, .tour-ctx, .tour-move")) e.classList.remove("tour-on", "tour-ctx", "tour-move");''',
    '''  for (const e of el.svg.querySelectorAll(".tour-on, .tour-ctx, .tour-move, .tour-lead")) e.classList.remove("tour-on", "tour-ctx", "tour-move", "tour-lead");''')

rep('''  el.kicker.textContent = `Step ${idx + 1} of ${n()}`;
  el.title.textContent = s.title;
  const figs = pick(s.figs, c);
  const eqs = pick(s.eq, c);
  const stale = api.state.result && api.state.result.status === "not_converged";
  const leadStatus = c.r && c.r.units && c.r.units[s.lead] ? c.r.units[s.lead].status : null;
  const usesRow = figs.some((k) => s.nums[k] && s.nums[k].row);
''', '''  el.kicker.textContent = `Step ${idx + 1} of ${n()}`;
  el.title.textContent = titleOf(s, c);
  const figs = pick(s.figs, c);
  const eqs = pick(s.eq, c);
  const stale = api.state.result && api.state.result.status === "not_converged";
  const lead = leadOf(s, c);
  const leadStatus = c.r && c.r.units && c.r.units[lead] ? c.r.units[lead].status : null;
''')
rep('''  if (usesRow) tags.push(tagHtml("pre", "Some values precomputed"));
''', '')
rep('''      return `<div data-fig="${esc(k)}"><dt>${esc(spec.l)}</dt><dd>${numHtml(spec, c, false)}<span''',
    '''      return `<div data-fig="${esc(k)}"><dt>${esc(spec.l)}</dt><dd>${numHtml(spec, c)}<span''')
# The picture follows the text it illustrates, so the words come first in
# reading order and on a phone.
rep('''    s.inset ? insetHtml(s.inset) : "",
    mediaHtml,
    `<div class="tour-text">${text}</div>`,
    eqHtml,
    figHtml,''', '''    mediaHtml,
    `<div class="tour-text">${text}</div>`,
    eqHtml,
    s.inset ? insetHtml(s.inset) : "",
    figHtml,''')
rep('''  el.card.classList.toggle("is-still", still);
  el.card.classList.toggle("is-paused", !still && !playing);
}

function announce() {
  const s = STEPS[idx];
  const text = Array.from(el.body.querySelectorAll(".tour-warn, .tour-text p")).map((p) => p.textContent.trim()).join(" ");
  el.live.textContent = `Step ${idx + 1} of ${n()}. ${s.title}. ${text}`;
}''', '''  el.card.classList.toggle("is-still", still);
  el.card.classList.toggle("is-paused", !still && !playing);
}

function announce() {
  const text = Array.from(el.body.querySelectorAll(".tour-warn, .tour-text p")).map((p) => p.textContent.trim()).join(" ");
  el.live.textContent = `Step ${idx + 1} of ${n()}. ${el.title.textContent}. ${text}`;
}

/** The key line under the card, which names only the keys that act now. */
function keyHint() {
  if (!el.keys) return;
  el.keys.textContent = still
    ? "With focus off the drawing, the Left and Right arrow keys change the step. Escape leaves the tour."
    : "With focus off the drawing, the Left and Right arrow keys change the step and Space pauses or plays. Escape leaves the tour.";
}''')
rep('''function updateBar() {
  const s = STEPS[idx];
  el.count.textContent = `Step ${idx + 1} of ${n()}`;
  el.track.setAttribute("aria-valuemin", "1");
  el.track.setAttribute("aria-valuemax", String(n()));
  el.track.setAttribute("aria-valuenow", String(idx + 1));
  el.track.setAttribute("aria-valuetext", `Step ${idx + 1} of ${n()}: ${s.title}`);
  el.prev.setAttribute("aria-disabled", String(idx === 0));
  el.next.setAttribute("aria-disabled", String(idx === n() - 1));''', '''function updateBar() {
  el.count.textContent = `Step ${idx + 1} of ${n()}`;
  el.track.setAttribute("aria-valuemin", "1");
  el.track.setAttribute("aria-valuemax", String(n()));
  el.track.setAttribute("aria-valuenow", String(idx + 1));
  el.track.setAttribute("aria-valuetext", `Step ${idx + 1} of ${n()}: ${el.title.textContent}`);
  for (const b of el.prevs) b.setAttribute("aria-disabled", String(idx === 0));
  for (const b of el.nexts) b.setAttribute("aria-disabled", String(idx === n() - 1));''')

rep('''function go(i) {
  if (!active) return;
  idx = Math.max(0, Math.min(n() - 1, i));
  ended = false;
  const s = STEPS[idx];
  const c = context();
  applyMarks(s, c);
  buildFlows(s, c);
  if (s.lead && api.select) api.select(s.lead);
  renderCard();
  updateBar();
  armTimer();
  announce();
  frame();
}''', '''/** Keep the head of the caption card in view when the step changes: the side
 * column goes back to its top, and when the card head sits above the window,
 * the page scrolls up to it. A card below the window is left there, since the
 * reader is then looking at the drawing. */
function keepCard() {
  const side = el.card.parentElement;
  if (side && side.scrollTop > 0) side.scrollTop = 0;
  const top = el.card.getBoundingClientRect().top;
  if (top < 0) window.scrollBy({ top: top - 8, behavior: still ? "auto" : "smooth" });
}

function go(i, first) {
  if (!active) return;
  idx = Math.max(0, Math.min(n() - 1, i));
  ended = false;
  const s = STEPS[idx];
  const c = context();
  applyMarks(s, c);
  buildFlows(s, c);
  const lead = leadOf(s, c);
  if (lead && api.select) api.select(lead);
  renderCard();
  updateBar();
  armTimer();
  announce();
  frame();
  if (!first) keepCard();
}''')
p.write_text(s, encoding='utf-8')
print('ok')
