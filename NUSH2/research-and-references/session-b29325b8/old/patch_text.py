import pathlib, re, sys
R = pathlib.Path(sys.argv[1])

def sub(rel, pairs):
    p = R / rel
    s = p.read_bytes().decode("utf-8")
    for old, new in pairs:
        assert old in s, (rel, old)
        s = s.replace(old, new)
    p.write_bytes(s.encode("utf-8"))

PH = r"\fbox{\parbox{0.9\textwidth}{\centering Artwork not distributed in this repository.}}"
sub("paper/sections/02_background.tex", [
    (r"\includegraphics[width=\textwidth]{../attachments/figure1_conventional_smr_ccs.png}", PH),
    (r"\includegraphics[width=0.94\textwidth]{../attachments/figure2_htgr_reference_power.png}", PH),
    (r"\includegraphics[width=\textwidth]{../attachments/figure3_proposed_htgr_smr_ccs.png}", PH),
])

sub("results/FIGURE_PROVENANCE.md", [(
    "Figures 1--3 are now included directly from user-provided project artwork under `attachments/`.",
    "Figures 1--3 were user-provided project artwork. The image files were removed from this "
    "repository on 2026-11-02 and the paper prints a placeholder for each. The paths in the "
    "table are a historical record.",
)])

sub("simulation/README.md", [
    ("Last updated: 2026-10-30", "Last updated: 2026-11-02"),
    ("This folder holds the research library and its index. It contains",
     "This folder holds the citation index of the research library. The papers are not "
     "stored in this repository. It contains"),
    ("24 papers held in this folder", "24 papers cited here"),
    ("## 7. Papers in this folder", "## 7. Papers are not stored here"),
    ("The 24 PDFs in the category folders are renamed copies of the files collected for "
     "this project. The content of each file is unchanged. Copyright in each paper remains "
     "with its publisher and authors. Morgan et al. 2018 and Kraussler and Hofbauer 2017 "
     "state on the files that they are published open access. For the other papers, check "
     "the reuse rights before keeping them in a public repository.",
     "The 24 PDFs were removed from this repository on 2026-11-02. Filenames in the index "
     "are the names used in the local library. Copyright in each paper remains with its "
     "publisher and authors. Use the DOI or official link in the inventory to obtain a copy."),
    ("paper kept in `11_Out_of_Scope`", "paper listed in `11_Out_of_Scope`"),
])
p = R / "simulation/README.md"
s, n = re.subn(r"\(INDEX\.md, \d+ PDFs?\)", "(INDEX.md)", p.read_bytes().decode("utf-8"))
assert n == 10, n
p.write_bytes(s.encode("utf-8"))

total = 0
for folder in ("01_Nuclear_HTGR", "02_Intermediate_Heat_Exchanger", "03_SMR_Reactor",
               "04_WGS_Reactor", "05_MEA_Thermodynamics", "06_MEA_Absorber", "07_MEA_Stripper",
               "08_Integrated_HTGR_SMR_CCS", "10_Supporting_References", "11_Out_of_Scope"):
    p = R / "simulation" / folder / "INDEX.md"
    s = p.read_bytes().decode("utf-8")
    assert "Papers in this folder: " in s, folder
    s = s.replace("Papers in this folder: ", "Papers listed here: ")
    total += s.count(".pdf` (in this folder)")
    s = s.replace(".pdf` (in this folder)", ".pdf` (local library name, not stored here)")
    s = s.replace("The file in this folder is Part 1 only.", "The indexed file is Part 1 only.")
    p.write_bytes(s.encode("utf-8"))
assert total == 24, total
print("patched")
