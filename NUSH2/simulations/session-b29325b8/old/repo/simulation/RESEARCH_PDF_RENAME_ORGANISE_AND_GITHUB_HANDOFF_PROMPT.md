# Research PDF Rename, Organise, and GitHub Handoff Prompt

## Purpose and execution order

This prompt is for the local Claude coding agent working on the user's computer. The target repository is:

https://github.com/weewzr/Implementation-of-Nuclear-Assisted-SMR-with-Carbon-Capture-in-Singapore

The project is an HTGR–helium heat integration–SMR–WGS–MEA carbon-capture simulation. The intended simulator will later be developed in the existing OUTRAM PARK Rust/DWSIM-derived codebase. The initial capture solvent is aqueous 30 wt% MEA. PSA is OUT OF SCOPE.

The user's local research folder contains the full-text PDFs and supporting files. **Do not search the web for paper identities, authors, titles, years, or technical purposes.** The target names and known reference mapping are supplied below. Read each local PDF only far enough to verify that its title page / first pages match the listed paper. Use the document itself, not the current filename, as the identity check.

The task has two stages:

- **Stage 1 — Local rename:** identify the supplied PDFs and rename them locally to the exact target filenames below. Do not upload or move anything to GitHub during this stage.
- **Stage 2 — Organise and publish:** after Stage 1 is complete and its manifest is checked, organise the renamed PDFs into the repository's research folders, generate the indexes, and publish only materials that are permitted to be redistributed. Do not overwrite or disturb the repository's existing manuscript, models, results, attachments, workflows, or other files.

Do not develop the process simulator, derive new equations, or write the future simulator implementation prompt in this task.

---

# STAGE 1 — Rename local research files using the supplied manifest

## 1. Locate the local files

Ask the user to select or provide the path to the folder containing their research PDFs if it is not already clear from the working directory. Search that folder recursively.

Create a read-only inventory first. For each PDF record:
- Current filename and relative path
- Page count
- Title read from the title page
- Authors
- Publication year
- DOI / report number if printed
- Whether the document is a main paper, dissertation, report, or supplement

Do not use web search, Crossref, Google Scholar, publisher pages, or other online lookup. The local source documents and the manifest below are the only identity sources for this task.

## 2. Fixed target filename manifest

Use these exact target filenames when the local document's title page matches. Do not rename a different paper to fit a manifest entry.

### Nuclear reactor, HTGR, and helium systems

| Verified document | Exact target filename | Primary use |
|---|---|---|
| IAEA-TECDOC-1382, HTTR/HTR-10 benchmark report (verify exact title/year from report cover) | `IAEA_2003_HTTR_HTR10_Reactor_Benchmarks.pdf` | Reactor physics and thermal-hydraulic benchmark data |
| Sato et al. (2006), N-HYPAC / HTTR hydrogen-production dynamic coupling paper (verify title page) | `Sato_2006_HTTR_Hydrogen_Dynamic_Coupling.pdf` | Dynamic reactor–helium–hydrogen plant coupling |
| JAERI-Tech-2005-031, HTTR hydrogen-production system mock-up heat-transfer report (verify cover title) | `JAERI_2005_HTTR_Hydrogen_System_Heat_Transfer_Data.pdf` | Equipment design, heat-transfer correlations and mock-up measurements |
| Tochio and Nakagawa (2011), HTTR intermediate heat exchanger thermal performance paper (verify title page) | `Tochio_Nakagawa_2011_HTTR_IHX_Reference_Design_Validation.pdf` | IHX geometry, helium conditions and operating validation |
| Zhang et al. (2021), HTR-PM / nuclear-assisted SMR thermodynamic integration paper (verify title page) | `Zhang_2021_HTRPM_SMR_Integrated_Heat_Material_Balance.pdf` | Integrated process thermodynamics and heat/material balance |

### SMR and reformer kinetics

| Verified document | Exact target filename | Primary use |
|---|---|---|
| Xu and Froment (1989), Part I, intrinsic kinetics (verify journal title and part title) | `Xu_Froment_1989_Part1_SMR_Intrinsic_Kinetic_Equations.pdf` | Intrinsic methane reforming / methanation / shift kinetic equations and fitted parameters |
| Xu and Froment (1989), Part II, diffusional limitations and reactor simulation (verify title page) | `Xu_Froment_1989_Part2_SMR_Catalyst_Diffusion_Effectiveness.pdf` | Catalyst diffusion, effectiveness factors and reactor simulation |
| Yin et al. (2007), helium-heated steam reformer for HTR-10 (verify title page) | `Yin_2007_HTR10_Helium_Heated_SMR_Reference_Model.pdf` | One-dimensional helium-heated reformer equations, geometry and experimental benchmark |
| Hoseinzade and Adams (2017), integrated steam reforming and nuclear heat system (verify title page) | `Hoseinzade_Adams_2017_Nuclear_Heated_SMR_Dynamic_Model.pdf` | Dynamic two-dimensional reformer model and nuclear heat coupling |

### WGS

| Verified document | Exact target filename | Primary use |
|---|---|---|
| Chinchen et al. (1984), Fe/Cr water-gas-shift catalyst transport study (verify exact title page) | `Chinchen_1984_WGS_FeCr_Catalyst_Mass_Transport_Data.pdf` | High-pressure catalyst mass-transfer / pore diffusion evidence |
| Kraussler and Hofbauer (2017), WGS kinetic model and experimental validation (verify title page) | `Kraussler_Hofbauer_2017_WGS_Kinetic_Equations_Validation.pdf` | Packed-bed WGS kinetics, reactor balances and validation |

### MEA thermodynamics, absorber, and stripper

| Verified document | Exact target filename | Primary use |
|---|---|---|
| Dugas (2009) doctoral dissertation, CO2 absorption, desorption and diffusion in aqueous piperazine and MEA (verify title page) | `Dugas_2009_MEA_CO2_Equilibrium_Absorption_Diffusion_Data.pdf` | MEA equilibrium, wetted-wall absorption/desorption rates and diffusion data |
| Plaza (2012) dissertation on modelling MEA / piperazine / promoted carbonate systems (verify title page) | `Plaza_2012_MEA_Phoenix_Thermodynamic_Kinetic_Equations.pdf` | Phoenix thermodynamics, kinetic formulation and Fortran routines |
| Morgan et al. (2018), rigorous solvent-based CO2 capture modelling, Part 2 (verify title page) | `Morgan_2018_MEA_Absorber_Stripper_Pilot_Validation_Data.pdf` | Pilot validation cases, capture, loading and column temperature data |
| Cownden and Lucquiaud (2026), improved open-access steady-state MEA model (verify title page) | `Cownden_Lucquiaud_2026_MEA_Improved_Equilibrium_Mass_Transfer_Model.pdf` | Updated equilibrium chemistry, kinetics, packing mass transfer and three-facility validation |
| Sønderby et al. (2013), pilot absorber / 30 wt% MEA study previously discussed (only if this exact paper is present; verify title page) | `Sonderby_2013_MEA_Packed_Absorber_Experimental_Validation.pdf` | Independent packed-absorber experimental validation |
| Kwak et al. (2012), CO2 capture pilot plant by amine absorption (only if this exact paper is present; verify title page) | `Kwak_2012_MEA_Absorber_Stripper_Pilot_Data.pdf` | Pilot absorber/stripper operation and regeneration-energy benchmark |

### Engineering process design reference

| Verified document | Exact target filename | Primary use |
|---|---|---|
| NETL / DOE DICE-CRCC Pre-FEED Final Report (2020; verify cover details) | `NETL_2020_DICE_CRCC_MEA_Capture_Plant_Equipment_Design.pdf` | Industrial capture-plant equipment arrangement and preliminary equipment duties |

### Source code and supplementary files

Preserve file extensions and name them using the related paper's base name plus a descriptive suffix, for example:
- `Cownden_Lucquiaud_2026_MEA_Improved_Equilibrium_Mass_Transfer_Model_Supplementary.pdf`
- `Cownden_Lucquiaud_2026_MEA_Improved_Equilibrium_Mass_Transfer_Model_Aspen_V11.bkp`
- `Cownden_Lucquiaud_2026_MEA_Improved_Equilibrium_Mass_Transfer_Model_Aspen_V14.bkp`
- `CCSI_MEA_Steady_State_Model_Fortran_Source.f`
- `CCSI_MEA_Steady_State_Model_Aspen_Backup.bkp`

Only use these names when the files are actually present and verified. Do not invent supplementary files or source files.

### Unmapped documents

For any other PDF that does not match the manifest:
1. Do not search the web.
2. Read the title page / first pages to identify it.
3. Preserve it unchanged during Stage 1.
4. Add it to `UNMAPPED_DOCUMENTS.csv` with its current path, visible title, authors/year if printed, and reason it did not match.
5. Do not guess a target filename. Propose a name only after identifying it from its own contents.

## 3. Naming and collision rules

- Use the exact filenames in the manifest when the match is verified.
- If two files match the same paper, classify them as exact duplicate, alternate scan/version, or supplement. Do not overwrite.
- If a target filename already exists, compare file hashes and document metadata before taking action.
- Never delete a file.
- Preserve an untouched backup of the original research folder.
- Keep a mapping from original path to renamed path.
- Do not silently merge separate parts of a multi-part paper.
- Do not convert file formats.
- Do not rename non-paper files based on assumptions.

## 4. Stage 1 outputs

Create locally:
- `LOCAL_RENAME_MANIFEST.csv`
- `UNMAPPED_DOCUMENTS.csv`
- `DUPLICATE_REVIEW.csv`

The rename manifest must contain original path, original filename, verified title, authors, year, new filename, new path, SHA-256, match confidence, and rename status.

After renaming, verify that every renamed file opens and that its page count/hash is recorded. Do not proceed to Stage 2 until the Stage 1 report is complete. If an identity mismatch exists, stop and report it rather than forcing the rename.

---

# STAGE 2 — Organise and publish eligible research to the existing GitHub repository

## 5. Target repository

Use the user's existing repository:

https://github.com/weewzr/Implementation-of-Nuclear-Assisted-SMR-with-Carbon-Capture-in-Singapore

Default branch: `main`.

Do not create a new repository.

Do not modify or delete any existing project files outside the new research/simulation paths described below.

## 6. Repository folder structure

Create the following new top-level directory:

`research/`

Within it create:

```
research/
  README.md
  Literature_Inventory.csv
  Equipment_Literature_Map.md
  01_Nuclear_HTGR/
    INDEX.md
  02_Intermediate_Heat_Exchanger/
    INDEX.md
  03_SMR_Reactor/
    INDEX.md
  04_WGS_Reactor/
    INDEX.md
  05_MEA_Thermodynamics/
    INDEX.md
  06_MEA_Absorber/
    INDEX.md
  07_MEA_Stripper/
    INDEX.md
  08_Integrated_HTGR_SMR_CCS/
    INDEX.md
  09_Software_Models_and_Code/
    INDEX.md
  10_Supporting_References/
    INDEX.md
  11_Out_of_Scope/
    INDEX.md
```

Also use the already-created `simulation/` directory for the prompt and future development instructions. Do not place research PDFs in `simulation/`.

The simulation folder is reserved for implementation prompts and related simulator specifications. Do not create or invent the actual final implementation prompt in this task. If the current task prompt is the only file in that folder, preserve it.

## 7. GitHub publication rules

The user previously requested that the organised research papers be published to the existing GitHub repository after the local rename stage. Therefore, upload actual renamed PDF files and permitted supporting files into the matching `research/` subfolders.

However:
- Check redistribution permissions and licence terms first.
- Do not assume a downloadable publisher PDF can be publicly redistributed.
- The repository is public; do not upload copyrighted publisher copies unless the licence or explicit permission allows redistribution.
- Where redistribution is not clearly permitted, do not upload the PDF. Instead, publish its verified citation, DOI/official URL, local renamed filename and engineering-use description in the inventory/index.
- Do not upload private, confidential, credential-bearing, or unrelated files.
- Do not upload unnecessary duplicate copies.
- For supplementary data/code, check its specific licence separately.
- Do not commit local absolute paths.
- Do not alter existing manuscript, economics, model, result, attachment, or workflow files.

If any large binary upload exceeds the available GitHub/API tool limits, do not truncate or corrupt it. Prepare a clearly reported local upload manifest and ask the user to upload that file through Git or GitHub Desktop.

## 8. Literature inventory and engineering usefulness

Create `research/Literature_Inventory.csv` with columns:
- Reference_ID
- Verified_Title
- Authors
- Year
- DOI_or_Official_URL
- Local_Renamed_Filename
- Repository_Path (if uploaded)
- Primary_Subsystem
- Secondary_Subsystems
- Primary_Engineering_Use
- Specific_Information_Available
- Equations_Available (Yes/Partial/No)
- Numerical_Values_Available (Yes/Partial/No)
- Experimental_Data_Available (Yes/Partial/No)
- Validation_Data_Available (Yes/Partial/No)
- Relevant_Pages_Tables_Equations
- Redistribution_Status
- Publication_Status
- Notes

Use stable reference IDs, e.g. REF-HTGR-001, REF-IHX-001, REF-SMR-001, REF-WGS-001, REF-MEA-001, REF-INT-001.

Use these usefulness labels consistently in the descriptions:
- Reference_Design
- Kinetic_Equations
- Thermodynamic_Equations
- Heat_Transfer
- Mass_Transfer
- Literature_Values
- Experimental_Data
- Validation_Data
- Dynamic_Model
- Integrated_Model
- Numerical_Methods

Do not claim a paper supplies parameters if it only describes a method. Distinguish experimental measurements, fitted coefficients, assumed design inputs and model predictions.

## 9. Equipment-to-literature map

Create `research/Equipment_Literature_Map.md`.

For each in-scope item, list its reference design, governing-equation sources, parameter sources and validation sources:

1. HTGR reactor core
2. Primary helium loop and circulator
3. Intermediate heat exchanger
4. Secondary helium loop
5. Helium-heated SMR reformer
6. WGS reactor
7. Syngas cooler
8. Condensate separator
9. MEA absorber
10. Rich-amine flash / pressure-letdown equipment, if justified by the final process configuration
11. Rich/lean amine heat exchanger
12. MEA stripper
13. Reboiler
14. Overhead condenser and reflux drum
15. Lean amine pump and cooler
16. Solvent makeup, inventory and recycle

For each item distinguish:
- Literature available
- Code already present in OUTRAM PARK
- New implementation required
- Experimental validation status

Do not mark a unit operation as implemented merely because generic equipment infrastructure exists.

## 10. Category indexes

Each category `INDEX.md` should contain concise entries:
- Reference ID
- Full paper title and authors/year
- DOI/official URL
- Repository PDF path, if uploaded
- Local renamed filename
- What it is useful for
- Which equipment/model it supports
- Specific equations/data/validation available
- Any caveat or missing supplementary material

Add links between related papers where useful. For example, link Plaza, Dugas, CCSI and Cownden/Lucquiaud in the MEA folders.

## 11. Root research README

The `research/README.md` should explain:
- Why the research folder exists
- Project scope: HTGR–SMR–WGS–MEA capture; PSA excluded
- Folder structure
- File naming convention
- Reference ID convention
- How to locate papers by equipment and engineering purpose
- Difference between an available paper, an implemented model and a validated model
- How redistribution restrictions were handled

## 12. GitHub write safety and verification

Before publishing:
1. Inspect the current repository tree and default branch.
2. Preserve all existing files.
3. Produce a proposed publication manifest showing every new path and whether it will contain a binary PDF or metadata-only entry.
4. Do not change the existing `simulation/` prompt file.
5. Confirm the repository's public visibility and licence implications.
6. Publish only after the user's explicit approval of the publication manifest.

After approval:
1. Upload eligible PDFs and support files into `research/`.
2. Add README, CSV inventory, equipment map and category indexes.
3. Commit with a clear message such as `Add organised HTGR-SMR-MEA-CCS research library`.
4. Verify the resulting GitHub tree.
5. Verify that all uploaded files have the expected paths and sizes.
6. Confirm which papers were uploaded and which are metadata-only because of redistribution restrictions.
7. Provide the repository URL and a concise publication report.

Do not force-push. Do not rewrite history. Do not delete existing files.

---

# Final completion criteria

The task is complete only when:

- Local PDFs have been renamed using the fixed manifest where identity was verified.
- All unmatched PDFs are reported without guessed names.
- Original files remain backed up.
- Stage 1 manifests are saved locally.
- The `research/` repository tree is organised and documented.
- Eligible PDFs are published only after approval and licence checks.
- Restricted PDFs are represented by citation/DOI entries instead of unauthorised uploads.
- The equipment-to-literature map is complete to the extent supported by the documents.
- The `simulation/` folder and its existing prompt file are preserved.
- No simulator code or final simulator implementation prompt has been generated.

The result should let Theodore navigate the research repository and immediately determine which paper provides:
- equipment reference design,
- governing equations,
- numerical literature values,
- experimental measurements,
- validation benchmarks,
- and known modelling limitations.

Do not perform web research. Do not invent documents. Do not fabricate missing data. Do not develop the simulator in this task.
