# MASTER ORGANISATIONAL PROMPT
## HTGR–SMR–MEA–CCS Research Library — Local File Organisation and GitHub Preparation

### 1. Objective

I have a folder on my computer containing scientific research papers, dissertations, technical reports, supplementary materials and other documents collected for my HTGR–SMR–MEA–CCS simulation project.

The filenames are inconsistent, and some papers may be duplicated or incorrectly named.

Your task is to organise the research library into two separate outputs:

**Output A — Local research library**

Organise and rename the actual research papers on my computer.

The original PDF files, supplementary documents and research materials must remain local.

**Output B — GitHub-ready research index**

Prepare a separate, lightweight repository containing the research folder structure, paper inventory, verified bibliographic references, DOI links, engineering-use descriptions and an empty folder for Theodore's future simulation development prompt.

Do not include the actual PDFs in the GitHub repository.

Do not publish to GitHub yet. I will provide a separate instruction when I am ready.

**This is strictly an organisational task. Do not develop or implement the simulation.**

---

### 2. Project context

The research supports an interactive, physics-based simulation of an integrated:

High-Temperature Gas-Cooled Reactor (HTGR)
→ Intermediate Heat Exchanger (IHX)
→ Helium-Heated Steam Methane Reforming (SMR)
→ Water-Gas Shift (WGS)
→ Monoethanolamine (MEA) Carbon Capture and Regeneration

The simulation will eventually be developed using:

https://github.com/theodoreOnzGit/outram-park-backend/tree/develop

The goal is a DWSIM-like simulation with material streams, energy streams, equipment models, thermodynamics, reaction kinetics, heat and mass transfer, numerical calculations and interactive visualisation.

The intended carbon capture solvent is 30 wt% aqueous MEA.

The provisional absorber pressure is 25 bar absolute.

The proposed nuclear reactor rating is 210 MWth, although design-specific thermal integration must be verified.

The simulation excludes:

- Pressure Swing Adsorption (PSA).
- PSA tail-gas recycling.
- Detailed hydrogen purification.
- Detailed CO₂ transport and offshore storage.

The objective of this task is to make the research easy to locate, understand and reference when Theodore eventually develops the simulator.

---

### 3. Scan and identify all research files

Recursively inspect the research folder on my computer.

Include:

- Journal articles.
- Research papers.
- Dissertations and theses.
- IAEA and JAEA reports.
- Experimental studies.
- Supplementary information.
- Aspen Plus model files.
- Fortran source code.
- Other relevant research files.

For each document, verify:

1. Full title.
2. Author names.
3. Publication year.
4. DOI, URL or report number.
5. Research topic.
6. Specific equipment supported.
7. Primary engineering usefulness.
8. Secondary engineering usefulness.
9. Whether it contains governing equations.
10. Whether it contains literature parameter values.
11. Whether it contains experimental measurements or validation data.
12. Whether it is a duplicate.
13. Whether it is complete or partial.

Use the actual document contents, not merely the existing filename.

Do not invent bibliographic information.

If a document cannot be identified reliably, flag it for manual review.

---

### 4. Rename papers locally according to engineering usefulness

**Important: Renaming applies only to the actual files in my local computer folder.**

Use this convention:

`Author_Year_Equipment_PrimaryUse_SecondaryUse.pdf`

For two-author papers, use both surnames when practical:

`Author1_Author2_Year_Equipment_PrimaryUse.pdf`

For institutional reports:

`Organisation_Year_Equipment_PrimaryUse.pdf`

The filename should tell Theodore what the paper is useful for.

Use the following standardised keywords:

| Keyword | Meaning |
|---|---|
| Reference_Design | Equipment geometry, configuration and operating conditions |
| Kinetic_Equations | Reaction-rate equations and constants |
| Thermodynamic_Equations | VLE, chemical equilibrium and property models |
| Heat_Transfer | Heat-transfer coefficients and thermal calculations |
| Mass_Transfer | Diffusion and gas–liquid transfer correlations |
| Literature_Values | Published numerical coefficients and properties |
| Experimental_Data | Direct experimental measurements |
| Validation_Data | Benchmark measurements for model testing |
| Dynamic_Model | Time-dependent governing equations |
| Integrated_Model | Coupled plant or equipment modelling |
| Numerical_Methods | Numerical algorithms and discretisation |

#### Expected naming examples

**Nuclear and heat integration**

- `IAEA_2003_HTGR_Reactor_Benchmark_Parameters.pdf`
- `Tochio_Nakagawa_2011_IHX_Reference_Design_Validation.pdf`
- `Sato_2006_HTGR_SMR_Dynamic_Coupling.pdf`
- `JAERI_2005_SMR_Heat_Transfer_Reference_Design.pdf`

**Steam methane reforming**

- `Xu_Froment_1989_Part1_SMR_Kinetic_Equations.pdf`
- `Xu_Froment_1989_Part2_SMR_Catalyst_Diffusion.pdf`
- `Yin_2007_SMR_Heat_Transfer_Reference_Design.pdf`
- `Hoseinzade_Adams_2017_SMR_Dynamic_Model_Equations.pdf`
- `Zhang_2021_HTGR_SMR_Integrated_Heat_Balance.pdf`

**Water-gas shift**

- `Chinchen_1984_WGS_Catalyst_Diffusion_Data.pdf`
- `Kraussler_Hofbauer_2017_WGS_Kinetics_Validation_Data.pdf`

**MEA absorber and stripper**

- `Dugas_2009_MEA_Equilibrium_Absorption_Rate_Data.pdf`
- `Plaza_2012_MEA_Thermodynamic_Kinetic_Equations.pdf`
- `Morgan_2018_MEA_Absorber_Stripper_Validation_Data.pdf`
- `Cownden_Lucquiaud_2026_MEA_Equilibrium_Mass_Transfer_Model.pdf`

These are suggested examples.

Verify each paper's contents and bibliographic details before applying its filename.

Use only the most important one or two engineering purposes in each filename.

Aim for filenames of approximately 80 characters or fewer where practical.

Preserve original file extensions.

Do not overwrite files with identical proposed names.

---

### 5. Organise the local research library

Create the following local directory structure:

HTGR_SMR_MEA_CCS_Research_Local/

    01_Nuclear_HTGR/
    02_Intermediate_Heat_Exchanger/
    03_SMR_Reactor/
    04_WGS_Reactor/
    05_MEA_Thermodynamics/
    06_MEA_Absorber/
    07_MEA_Stripper/
    08_Integrated_HTGR_SMR_CCS/
    09_Software_Models_and_Code/
    10_Supporting_References/
    11_Out_of_Scope/

Place each renamed paper in its most relevant folder.

Keep one primary copy of each distinct paper.

If a paper supports multiple subsystems, record those additional uses in the inventory rather than duplicating the PDF.

Preserve the original research folder as a backup.

Do not delete the original files.

---

### 6. Create a separate GitHub-ready folder

Create a second directory:

HTGR_SMR_MEA_CCS_Research_GitHub/

This must be separate from the local PDF library.

It should contain the same research categories but **without the actual research PDFs**.

Use the following structure:

    01_Nuclear_HTGR/
    02_Intermediate_Heat_Exchanger/
    03_SMR_Reactor/
    04_WGS_Reactor/
    05_MEA_Thermodynamics/
    06_MEA_Absorber/
    07_MEA_Stripper/
    08_Integrated_HTGR_SMR_CCS/
    09_Software_Models_and_Code/
    10_Supporting_References/
    11_Out_of_Scope/
    12_Theodore_Simulation_Prompt/

Each research category folder should contain an `INDEX.md` file listing:

- Paper title.
- Authors and year.
- DOI or official URL.
- Local renamed filename.
- Primary engineering purpose.
- Secondary engineering uses.
- Equipment supported.
- Whether equations, parameter values and validation data are available.
- Important page, table or equation references, where verified.
- Any missing supplementary materials.

Do not copy the PDFs into these folders.

Do not include absolute local computer paths such as `C:\Users\...` in the public GitHub index.

Use only the renamed filename and reference ID.

---

### 7. Reserve a folder for Theodore's future development prompt

Inside the GitHub-ready directory, create:

`12_Theodore_Simulation_Prompt/`

This folder is reserved for a future instruction.

Do not generate Theodore's simulation prompt yet.

Do not include:

- Implementation instructions.
- Mathematical derivations.
- Coding tasks.
- Simulation architecture specifications.
- Development plans.
- Placeholder prompt text.

Leave the folder empty except for a `.gitkeep` file.

The `.gitkeep` file is necessary because Git does not track empty directories.

When I provide the future simulation prompt, it will be placed in this folder.

---

### 8. Generate the master literature inventory

Create:

`Literature_Inventory.csv`

Place a copy in the local library and the GitHub-ready directory.

Include:

- Reference ID.
- Verified authors.
- Publication year.
- Full title.
- DOI or official URL.
- Renamed local filename.
- Primary folder.
- Primary engineering purpose.
- Secondary engineering purposes.
- Equipment supported.
- Equations available: Yes / Partial / No.
- Literature values available: Yes / Partial / No.
- Validation data available: Yes / Partial / No.
- Relevant pages, tables and equations.
- Priority: CORE / SUPPORTING / REFERENCE.
- Missing supplementary materials.
- Notes.

Use stable IDs such as:

`REF-HTGR-001`

`REF-IHX-001`

`REF-SMR-001`

`REF-WGS-001`

`REF-MEA-001`

`REF-INT-001`

Do not change reference IDs when filenames are renamed.

The inventory should make it easy to find the correct paper for each equipment model.

---

### 9. Generate an equipment-to-literature index

Create:

`Equipment_Literature_Map.md`

Place this in the GitHub-ready root directory.

For each equipment item, list the relevant papers under:

- Reference design.
- Governing equations.
- Numerical parameter values.
- Experimental validation.
- Supporting references.

Cover:

1. HTGR reactor.
2. Primary helium circulator.
3. Intermediate heat exchanger.
4. Secondary helium loop.
5. Helium-heated SMR.
6. WGS reactor.
7. Syngas cooler.
8. Condensate separator.
9. MEA absorber.
10. Rich amine flash drum.
11. Rich/lean amine exchanger.
12. MEA stripper.
13. Reboiler.
14. Condenser and reflux drum.
15. Lean amine pump and cooler.
16. Solvent recycle.

Only assign papers to a category when their actual contents support that use.

If a source is missing, write `Not yet identified`.

Do not invent references.

---

### 10. Handle duplicates safely

Detect duplicates using:

- SHA-256 hashes.
- DOI.
- Publication title.
- Authors.
- Page count.
- Document completeness.

Distinguish exact duplicates from:

- Different PDF versions.
- Main articles and supplementary files.
- Different parts of a multi-part study.
- Documents with similar titles.

Keep the best available copy in the organised local library.

Preserve duplicate originals in the original backup folder.

Do not delete files automatically.

Generate a duplicate report in the local library.

---

### 11. Create a rename manifest

Create:

`Rename_Manifest.csv`

Store this in the local library only.

Include:

- Original filename.
- Original relative path.
- Renamed filename.
- New relative path.
- File hash.
- Duplicate status.
- Rename status.
- Notes.

This manifest is for tracking changes to the local computer files.

**Do not publish the rename manifest to GitHub**, because it is an internal file-management record.

---

### 12. Create the GitHub README

Create a root-level `README.md` in the GitHub-ready directory.

It should contain:

1. Project title.
2. Brief explanation of the proposed HTGR–SMR–MEA–CCS simulation.
3. Reference to the OUTRAM PARK `develop` branch.
4. Description of the research library.
5. Folder structure.
6. Paper naming convention.
7. Explanation of the reference IDs.
8. Link to `Literature_Inventory.csv`.
9. Link to `Equipment_Literature_Map.md`.
10. Instructions for locating papers through their DOI or official publication links.
11. Explanation that the original PDFs are stored locally and are not included in GitHub.
12. Note that PSA is excluded.
13. Note that the Theodore simulation prompt folder is intentionally empty.

The README should help Theodore understand and navigate the research library.

Do not include the future simulator implementation prompt.

---

### 13. Prepare the GitHub directory safely

The GitHub-ready directory must contain only:

- Markdown files.
- CSV inventories.
- Bibliographic metadata.
- DOI and official publication links.
- `.gitkeep` files where needed.

Do not include:

- Research PDFs.
- Copyrighted journal downloads.
- Personal local file paths.
- Private documents.
- Credentials.
- Aspen model files unless separately approved and licensed.
- Fortran source files unless separately approved and licensed.
- Duplicate archives.
- Temporary extraction files.

Create a `.gitignore` to exclude PDFs, temporary files, credentials, local paths and other private material.

Before future publication, verify that the directory contains no unintended binary documents.

Do not initialise or publish a GitHub repository yet.

---

### 14. Execution workflow

Perform the task in two stages.

**Stage A — Audit and proposal**

1. Inspect the existing research folder.
2. Verify the papers.
3. Detect duplicates.
4. Propose filenames based on engineering usefulness.
5. Propose local folder assignments.
6. Generate a rename proposal.
7. Show the proposal for my approval.

Do not modify the original files during Stage A.

**Stage B — Organisation**

Only after I approve Stage A:

1. Preserve the original folder as a backup.
2. Create the local organised library.
3. Rename and classify the papers locally.
4. Create the GitHub-ready folder structure separately.
5. Create the GitHub category indexes.
6. Generate the master literature inventory.
7. Generate the equipment-to-literature map.
8. Generate the local-only rename manifest.
9. Create the GitHub README.
10. Create the empty Theodore prompt folder with `.gitkeep`.
11. Verify all files are accounted for.
12. Verify the GitHub-ready directory contains no PDFs or private files.

Do not publish to GitHub.

---

### 15. Final deliverables

After completing Stage B, report:

**Local computer**

- Path to the organised local research library.
- Number of papers renamed.
- Number of duplicates identified.
- Any files requiring manual review.
- Confirmation that all original files were preserved.

**GitHub preparation**

- Path to the GitHub-ready directory.
- Number of category indexes generated.
- Master literature inventory.
- Equipment-to-literature map.
- README.
- Confirmation that no PDFs were copied into the GitHub-ready directory.
- Confirmation that `12_Theodore_Simulation_Prompt` exists and contains only `.gitkeep`.

Do not create a remote repository or push files to GitHub.

Do not develop the simulator.

Do not generate Theodore's future simulation prompt.

**The sole objective is to organise the original research papers locally and prepare a clean, navigable, citation-based research repository for future publication on GitHub.**
