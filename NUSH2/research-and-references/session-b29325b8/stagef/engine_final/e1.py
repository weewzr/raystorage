from pathlib import Path
def rep(path, a, b, n=1):
    p = Path(path); s = p.read_bytes().decode("utf-8")
    crlf = "\r\n" in s
    s = s.replace("\r\n", "\n")
    assert s.count(a) == n, (path, a[:60], s.count(a))
    s = s.replace(a, b)
    if crlf: s = s.replace("\n", "\r\n")
    p.write_bytes(s.encode("utf-8"))
rep("engine/plant/src/terms.rs", "use nush2_units::columns::ColumnTerms;\n", "use nush2_units::columns::ColumnTerms;\npub use nush2_units::columns::IslandDuties;\n")
rep("engine/plant/src/terms.rs", "pub fn design_duties_cached() -> Result<nush2_units::columns::IslandDuties> {\n    static DUTIES: OnceLock<Result<nush2_units::columns::IslandDuties>> = OnceLock::new();", "pub fn design_duties_cached() -> Result<IslandDuties> {\n    static DUTIES: OnceLock<Result<IslandDuties>> = OnceLock::new();")
rep("engine/wasm/src/solve.rs", """    pub island: std::result::Result<nush2_units_duties::IslandDuties, String>,
}

/// Local name for the island duty type, so the reply code reads plainly.
mod nush2_units_duties {
    pub use nush2_plant::terms::IslandDuties;
}
""", """    pub island: std::result::Result<IslandDuties, String>,
}
""")
rep("engine/wasm/src/solve.rs", ") -> std::result::Result<nush2_units_duties::IslandDuties, String> {", ") -> std::result::Result<IslandDuties, String> {")
rep("engine/wasm/src/solve.rs", "use nush2_plant::terms::{self, Source};", "use nush2_plant::terms::{self, IslandDuties, Source};")
print("ok")
