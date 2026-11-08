//! Verifier probe, scratch copy only: read every NAME.req.json in PROBE_DIR,
//! run it through `dispatch` (command 2, solve) and write NAME.reply.json.

#[test]
fn probe_dump() {
    let Ok(dir) = std::env::var("PROBE_DIR") else {
        return;
    };
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_string_lossy().ends_with(".req.json"))
        .collect();
    names.sort();
    for p in names {
        let req = std::fs::read(&p).unwrap();
        let t0 = std::time::Instant::now();
        let cmd = if p.to_string_lossy().contains("sweep") { 3 } else { 2 };
        let reply = crate::dispatch(cmd, &req);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        let out = p.to_string_lossy().replace(".req.json", ".reply.json");
        std::fs::write(&out, reply).unwrap();
        println!("{} {:.1} ms", p.display(), ms);
    }
}
