# HTGR-SMR-MEA-CCS Research Library Index

Last updated: 2026-10-30

## 1. Project

This is a lightweight, citation-based index of the research collected for a proposed interactive, physics-based simulation of an integrated plant:

HTGR -> intermediate heat exchanger (IHX) -> helium-heated steam methane reforming (SMR) -> water-gas shift (WGS) -> monoethanolamine (MEA) carbon capture and regeneration

The simulation is intended to be built on the OUTRAM PARK backend, `develop` branch: https://github.com/theodoreOnzGit/outram-park-backend/tree/develop

The aim is a DWSIM-like simulation with material streams, energy streams, equipment models, thermodynamics, reaction kinetics, heat and mass transfer, numerical calculations and interactive visualisation. Design inputs so far: 30 wt% aqueous MEA, a provisional absorber pressure of 25 bar absolute, and a proposed reactor rating of 210 MWth. Design-specific thermal integration for that rating still has to be verified, and a text search of the searchable papers in this library found no source for the 210 MWth figure. The closest rating found is the 215 MW(th) MHR-100SMR concept in REF-INT-003 (IAEA 2017, Table 20).

This folder holds the research library and its index. It contains no simulation and no implementation plan. It sits in the `simulation` folder of the repository, next to `MASTER_ORGANISATIONAL_PROMPT.md`.

## 2. The research library

The index covers 25 entries: 24 papers held in this folder and 1 paper that is listed but not yet obtained (REF-SMR-006). They are sorted into 11 category folders, each with an `INDEX.md` giving the title, authors, year, DOI or official link, the renamed filename, engineering purposes, equipment supported, availability of equations, parameter values and validation data, verified page or table references and any missing supplementary materials.

Two files cover the whole library:

- [Literature_Inventory.csv](Literature_Inventory.csv): one row per paper with all of the above.
- [Equipment_Literature_Map.md](Equipment_Literature_Map.md): for each of 16 equipment items, the papers that support its reference design, governing equations, parameter values and experimental validation. Items with no verified source read Not yet identified.

## 3. Folder structure

```
simulation/
    MASTER_ORGANISATIONAL_PROMPT.md
    README.md
    Literature_Inventory.csv
    Equipment_Literature_Map.md
    .gitignore
    01_Nuclear_HTGR/   (INDEX.md, 1 PDF)
    02_Intermediate_Heat_Exchanger/   (INDEX.md, 1 PDF)
    03_SMR_Reactor/   (INDEX.md, 5 PDFs)
    04_WGS_Reactor/   (INDEX.md, 2 PDFs)
    05_MEA_Thermodynamics/   (INDEX.md, 4 PDFs)
    06_MEA_Absorber/   (INDEX.md, 4 PDFs)
    07_MEA_Stripper/   (INDEX.md, 1 PDF)
    08_Integrated_HTGR_SMR_CCS/   (INDEX.md, 3 PDFs)
    09_Software_Models_and_Code/   (INDEX.md only, no files yet)
    10_Supporting_References/   (INDEX.md, 2 PDFs)
    11_Out_of_Scope/   (INDEX.md, 1 PDF)
    12_Theodore_Simulation_Prompt/   (.gitkeep only)
```

## 4. Paper naming convention

Paper files are named by what they are useful for:

- `Author_Year_Equipment_PrimaryUse_SecondaryUse.pdf`
- Two authors: `Author1_Author2_Year_Equipment_PrimaryUse.pdf`
- Institutional reports: `Organisation_Year_Equipment_PrimaryUse.pdf`

Standard keywords:

| Keyword | Meaning |
|---|---|
| Reference_Design | Equipment geometry, configuration and operating conditions |
| Kinetic_Equations | Reaction-rate equations and constants |
| Thermodynamic_Equations | VLE, chemical equilibrium and property models |
| Heat_Transfer | Heat-transfer coefficients and thermal calculations |
| Mass_Transfer | Diffusion and gas-liquid transfer correlations |
| Literature_Values | Published numerical coefficients and properties |
| Experimental_Data | Direct experimental measurements |
| Validation_Data | Benchmark measurements for model testing |
| Dynamic_Model | Time-dependent governing equations |
| Integrated_Model | Coupled plant or equipment modelling |
| Numerical_Methods | Numerical algorithms and discretisation |

## 5. Reference IDs

Each entry has a stable ID of the form `REF-<AREA>-<number>`. The ID never changes when a file is renamed. Areas: HTGR (folder 01), IHX (02), SMR (03), WGS (04), MEA (05 to 07), INT (08), SUP (10, supporting references) and OOS (11, out of scope). The folder an entry sits in is given in `Literature_Inventory.csv`, and some papers also support other folders; those extra uses are recorded in the inventory rather than by duplicating files.

## 6. Locating the papers

1. Open the DOI as `https://doi.org/<DOI>`, or use the official link in the inventory (IAEA record, NETL report, University of Texas repository handles for the two dissertations).
2. Xu and Froment Part 2 (REF-SMR-002) and Sato et al. 2006 (REF-INT-001) print no DOI and none has been verified. Search by title, journal and year.
3. JAERI-Tech 2005-031 (REF-SMR-005) is identified by report number; an official URL has not been verified. The same applies to IAEA NP-T-4.3 (REF-INT-003) and IEAGHG 2022-07 (REF-SUP-002), which are identified by series number.
4. Most journal articles are behind publisher paywalls. Morgan et al. 2018 and Kraussler and Hofbauer 2017 state on the files that they are published open access.

## 7. Papers in this folder

The 24 PDFs in the category folders are renamed copies of the files collected for this project. The content of each file is unchanged. Copyright in each paper remains with its publisher and authors. Morgan et al. 2018 and Kraussler and Hofbauer 2017 state on the files that they are published open access. For the other papers, check the reuse rights before keeping them in a public repository. Supplementary documents, saved web pages, archives, model and source files, temporary files and credentials are not included, and `.gitignore` excludes them.

## 8. Scope exclusions

The simulation excludes Pressure Swing Adsorption (PSA), PSA tail-gas recycling, detailed hydrogen purification, and detailed CO2 transport and offshore storage. REF-OOS-001 is a PSA adsorption paper kept in `11_Out_of_Scope` for reference only.

## 9. Reserved folder

`12_Theodore_Simulation_Prompt/` is intentionally empty and contains only a `.gitkeep` file so that Git tracks the folder.

## 10. Reading the availability flags

Equations, literature values and validation data are marked Yes, Partial or No. Unverified means the content has not yet been checked at page level. Not assessed is used for the one paper that is not in the library.

## 11. Known gaps

1. REF-SMR-006 (Ishii et al. 2023, helium-heated reformer validation) is not in the library. Obtain it through its DOI.
2. REF-HTGR-001 is Part 1 only (231 of 352 pages). Chapter 4 onward (HTR-10 core physics benchmarks) is missing.
3. REF-MEA-003: the Supplementary Material and the Aspen Plus source file it refers to are missing. REF-INT-002: supplementary material is missing.
4. REF-MEA-004 is Part 2 of a pair; Part 1 is not in the library.
5. No Aspen Plus, Fortran or other model files were found, so `09_Software_Models_and_Code` is empty.
6. Several equipment items have no verified source yet (primary helium circulator, syngas cooler, condensate separator, rich amine flash drum, and parts of the reboiler, condenser, lean amine pump and cooler and solvent recycle entries). See the equipment map.
7. MEA at the provisional 25 bar absolute condition is only partly covered. REF-MEA-009 gives CO2-MEA equilibrium and speciation at 1 to 50 bar and 303 to 323 K for 10 to 30 mass% MEA, and REF-MEA-008 gives packed-column absorption data at 0.1, 1 and 5 MPa, but for 20 wt% MEA and CO2 in natural gas. The other papers that state a pressure describe near-atmospheric flue gas. No column test at 25 bar with 30 wt% MEA and a syngas feed has been identified.
8. The proposed 210 MWth rating has no source. The nearest design-scale value found is 215 MW(th) (REF-INT-003, Table 20).
