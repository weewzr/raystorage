// Runs the Rust engine off the main thread. The module imports nothing, so it
// is instantiated with an empty import object. A trap discards the instance;
// the next request builds a fresh one.
"use strict";

const CMD = { registry: 1, solve: 2, sweep: 3 };
const enc = new TextEncoder();
const dec = new TextDecoder();
let ex = null;

async function init() {
  const res = await fetch(new URL("nush2.wasm?v=f2ad0dd32ac146bb85a9331ad11fc64c1938b04a", self.location.href));
  if (!res.ok) throw new Error(`nush2.wasm could not be fetched (HTTP ${res.status})`);
  const bytes = await res.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, {});
  ex = instance.exports;
}

function call(cmd, text) {
  const data = enc.encode(text);
  const p = ex.alloc(data.length);
  new Uint8Array(ex.memory.buffer, p, data.length).set(data);
  const r = ex.call(cmd, p, data.length);
  ex.dealloc(p, data.length);
  // Memory may have grown during the call, so read the buffer again.
  const n = new DataView(ex.memory.buffer).getUint32(r, true);
  const out = dec.decode(new Uint8Array(ex.memory.buffer, r + 4, n));
  ex.dealloc(r, n + 4);
  return out;
}

self.onmessage = async (e) => {
  const { id, cmd, body } = e.data;
  const t0 = performance.now();
  try {
    if (!(cmd in CMD)) throw new Error(`unknown command ${cmd}`);
    if (!ex) await init();
    const text = call(CMD[cmd], body);
    self.postMessage({ id, ok: true, text, ms: performance.now() - t0 });
  } catch (err) {
    const trapped = err instanceof WebAssembly.RuntimeError;
    if (trapped) ex = null;
    self.postMessage({ id, ok: false, error: String(err && err.message ? err.message : err), trapped, ms: performance.now() - t0 });
  }
};
