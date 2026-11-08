//! Browser ABI of the NUS-H2 engine. A plain cdylib with no wasm-bindgen, so
//! the module imports nothing and the page instantiates it with `{}`.
//!
//! Exports: `memory`, `alloc(n)`, `dealloc(p, n)`, `call(cmd, p, n)` and
//! `abi_version()`. A request is UTF-8 JSON written into a block from `alloc`.
//! A reply is a 4-byte little-endian length followed by that many bytes of
//! UTF-8 JSON; the caller frees it with `dealloc(r, len + 4)`.
//!
//! Every block, request or reply, is a `Box<[u8]>` whose length equals its
//! capacity, so `dealloc` rebuilds exactly the box that was handed out.
//!
//! Commands: 1 `registry`, 2 `solve`, 3 `sweep`. See README.md for the JSON.

pub mod registry;
pub mod solve;

#[cfg(test)]
mod tests;

use serde_json::Value;

const ABI: u32 = 1;

#[no_mangle]
pub extern "C" fn abi_version() -> u32 {
    ABI
}

fn leak(b: Box<[u8]>) -> *mut u8 {
    Box::into_raw(b) as *mut u8
}

/// Allocate `n` zeroed bytes for a request.
#[no_mangle]
pub extern "C" fn alloc(n: usize) -> *mut u8 {
    leak(vec![0u8; n].into_boxed_slice())
}

/// Free a block from `alloc(n)`, or a reply from `call` with `n = len + 4`.
///
/// # Safety
/// `p` must come from `alloc(n)` or from `call` with that exact `n`, and must
/// not be freed twice.
#[no_mangle]
pub unsafe extern "C" fn dealloc(p: *mut u8, n: usize) {
    if p.is_null() {
        return;
    }
    // SAFETY: the caller hands back a pointer and length from `leak`.
    drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(p, n)));
}

/// Run one command on a JSON body and return the reply text.
pub fn dispatch(cmd: u32, req: &[u8]) -> String {
    let body: Value = if req.is_empty() {
        Value::Object(Default::default())
    } else {
        match serde_json::from_slice(req) {
            Ok(v) => v,
            Err(e) => {
                return solve::refused("parse", &format!("request is not JSON: {e}")).to_string()
            }
        }
    };
    let reply = match cmd {
        1 => solve::registry(),
        2 => solve::solve(&body),
        3 => solve::sweep(&body),
        other => solve::refused("bad_cmd", &format!("unknown command {other}")),
    };
    reply.to_string()
}

/// Frame a reply: 4-byte little-endian length, then the bytes.
pub fn frame(text: &str) -> Box<[u8]> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() + 4);
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
    out.into_boxed_slice()
}

/// Run command `cmd` on the JSON request at `p`, `n` bytes long.
///
/// # Safety
/// `p` must point to `n` readable bytes, or be null with `n = 0`.
#[no_mangle]
pub unsafe extern "C" fn call(cmd: u32, p: *const u8, n: usize) -> *mut u8 {
    let req: &[u8] = if p.is_null() || n == 0 {
        &[]
    } else {
        // SAFETY: guaranteed by the caller.
        std::slice::from_raw_parts(p, n)
    };
    leak(frame(&dispatch(cmd, req)))
}

#[cfg(test)]
mod probe;
