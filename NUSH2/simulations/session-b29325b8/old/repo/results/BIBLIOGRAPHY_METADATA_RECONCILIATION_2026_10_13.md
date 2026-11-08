# Citation metadata reconciliation continuation (13 October 2026)

## Corrections verified against publisher pages

1. `mti2025nuclear`: The existing link resolves to MTI's **Singapore Energy Lecture by Min(EST) Tan See Leng at the 2025 Singapore International Energy Week**, dated **27 October 2025**, not a document titled *Nuclear Energy* dated 22 October. The BibTeX title and note now describe the actual linked source. The original citation key is preserved for LaTeX compatibility. Source: https://www.mti.gov.sg/newsroom/singapore-energy-lecture-by-min-est--tan-see-leng-at-the-2025-singapore-international-energy-week/
2. `hamamoto2006ihx`: JAEA JOPSS confirms Hamamoto, Saikusa, Shinohara and Tachibana; *Development of intermediate heat exchanger for high temperature engineering test reactor*, *Koon Gakkai-Shi* 32(1), 43-49, January 2006, and the constructed 10 MW helium-helium IHX. Removed duplicate and contradictory `note` fields; retain the official JOPSS record URL. Source: https://jopss.jaea.go.jp/search/servlet/search?5000261=&language=1
3. `jaeri2004gthtr300c`: Previous commit changed the publication type to workshop contribution within JAERI-Review 2004-026. Further page-level checking is still warranted before using it as the unique authority for a 170 MW physical IHX.

## Remaining checks
- Cross-check every DOI against publisher title, year and author list, rather than inferring validity from a DOI-shaped string.
- Audit whether the changed MTI source actually supports each claim where `mti2025nuclear` is cited.
- Confirm whether JAEA workshop pages 91-98 contain the precise reactor/IHX claims attributed to them; seek an independent design paper for corroboration.
- Run bibliography and paper build at final HEAD; do not assume the current commits have passed CI.
- Keep unresolved access failures distinct from actual dead links.

This is a targeted verified correction, not a claim that all 56 citations are now fully checked.
