# Batch 05 — 2026-10-13 tested links

Each source URL/DOI was opened. Browser tool errors do not establish broken links. Full tested URLs are the bibliography links associated with each key.

| Key | Status | Observed outcome / identity |
|---|---|---|
| `morris2016agr1` | UNRESOLVED | DOI inaccessible |
| `demkowicz2015facs` | UNRESOLVED | DOI inaccessible |
| `heliumsmr2022` | UNRESOLVED | DOI inaccessible |
| `nrctriso6844` | UNRESOLVED | NRC cache miss |
| `nrcngnpfpt2008` | VERIFIED_DIRECT | NRC NUREG/CR-6944 Volume 3 2008 publisher record, authors and title matched |
| `dipu2016tritium` | UNRESOLVED | DOI inaccessible |
| `takeda2000permeation` | UNRESOLVED | DOI inaccessible |
| `ding2018epz` | UNRESOLVED | DOI inaccessible |
| `mee2022htrpmepz` | ACCESS_RESTRICTED | MEE PDF >12MB tool limit, not broken |
| `mee2009htrpmreview` | CORRECTED_VERIFIED | Opened Chinese regulator page is 2008-05-15 consultation letter on HTR-PM safety review principles, NOT a 2009 published safety-review standard |

**Confirmed mismatch:** MEE link identifies a 2008 consultation letter, not a final 2009 safety-review publication. Correct the bibliography metadata and narrow claims accordingly. No Chinese HTR-PM criterion is transferable as a Singapore licensing standard.

Bibliography repaired at commit `4c33a19ea1aad8406c5b6d3aadec6e96b4a2d98a`: correct 2008 consultation-letter identity, same citation key retained.
