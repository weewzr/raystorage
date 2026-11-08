| crate | workspace crates reached | external pkgs (lock, upper bound) | hazards present in closure |
|---|---|---|---|
| petir | 1: petir | 1 | none |
| tampines-steam-tables | 3: outram-foam-basic-lib, petir, tampines-steam-tables | 58 | getrandom, js-sys, rand, wasm-bindgen |
| outram-park-fork-coolprop | 3: outram-foam-basic-lib, outram-park-fork-coolprop, petir | 58 | getrandom, js-sys, rand, wasm-bindgen |
| outram-park-fork-dwsim-libs | 2: outram-park-fork-dwsim-libs, petir | 75 | getrandom, js-sys, rand, wasm-bindgen, zip |
| dover | 3: dover, outram-park-fork-dwsim-libs, petir | 82 | getrandom, js-sys, rand, wasm-bindgen, zip |
| tuas_boussinesq_solver | 2: petir, tuas_boussinesq_solver | 62 | getrandom, js-sys, rand, wasm-bindgen |
| chem-eng-real-time-process-control-simulator | 2: chem-eng-real-time-process-control-simulator, petir | 19 | none |
| outram-foam-basic-lib | 2: outram-foam-basic-lib, petir | 58 | getrandom, js-sys, rand, wasm-bindgen |
| tampines | 15: boon-lay, chem-eng-real-time-process-control-simulator, njoy-outram-park-fork, outram-blender, outram-foam-basic-lib, outram-foam-multiphase, outram-mc-libs, outram-park-fork-coolprop, outram-park-fork-dwsim-libs, outram-park-fork-liggghts, petir, raffles, tampines, tampines-steam-tables, tuas_boussinesq_solver | 206 | getrandom, js-sys, rand, tokio, wasm-bindgen, web-sys, wgpu, zip |

| crate | lib .rs files scanned | thread::spawn | Instant::now | SystemTime | std::fs | rayon |
|---|---|---|---|---|---|---|
| petir | 156 | 0 | 0 | 0 | 0 | 1 |
| tampines-steam-tables | 213 | 0 | 0 | 0 | 0 | 0 |
| outram-park-fork-coolprop | 371 | 0 | 0 | 0 | 0 | 0 |
| outram-park-fork-dwsim-libs | 143 | 0 | 3 | 0 | 1 | 0 |
| dover | 5 | 0 | 0 | 0 | 1 | 0 |
| tuas_boussinesq_solver | 246 | 37 | 0 | 20 | 0 | 0 |
| chem-eng-real-time-process-control-simulator | 46 | 5 | 0 | 0 | 24 | 0 |
| outram-foam-basic-lib | 140 | 0 | 0 | 0 | 12 | 44 |
| tampines | 40 | 0 | 0 | 0 | 0 | 0 |
