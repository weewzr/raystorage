import ctypes, json, struct, sys
dll = ctypes.CDLL(sys.argv[1])
dll.alloc.restype = ctypes.c_void_p; dll.alloc.argtypes = [ctypes.c_size_t]
dll.call.restype = ctypes.c_void_p; dll.call.argtypes = [ctypes.c_uint32, ctypes.c_void_p, ctypes.c_size_t]
dll.dealloc.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
def solve(inputs):
    b = json.dumps({"schema": 1, "inputs": inputs, "kpi_only": False}).encode()
    p = dll.alloc(len(b)); ctypes.memmove(p, b, len(b))
    r = dll.call(2, p, len(b)); dll.dealloc(p, len(b))
    n = struct.unpack("<I", ctypes.string_at(r, 4))[0]
    t = ctypes.string_at(r + 4, n); dll.dealloc(r, n + 4)
    return json.loads(t)
def item(lst, i): return next(e["value"] for e in lst if e["id"] == i)
for inp in [{"rating_mwth": x} for x in (178.4, 178.494, 179.5, 181.0, 182.0, 210.0)] + [{"basis": "case_a"}, {"basis": "case_a", "tail_gas": "inl_selexol"}]:
    r = solve(inp)
    k = r["kpi"]; bal = r["balances"]
    notes = [m["text"] for m in r["messages"] if m["code"] == "import_memo_high"]
    print(inp, "need", round(item(bal["energy"], "htgr_heat_need"), 4), "feasible", r["feasibility"]["feasible"],
          "kpi", round(k["unmet_power_mwe"], 4), "memo", round(item(bal["power"], "import_at_rated"), 4),
          "list", round(item(bal["energy"], "unmet_electrical"), 4))
    for t in notes: print("   note:", t)
