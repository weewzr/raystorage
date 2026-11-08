# Targeted bibliography repair continuation - 2026-10-13

## Verified repairs applied
1. `hamamoto2006ihx`: Added the JAEA JOPSS original publication record https://jopss.jaea.go.jp/search/servlet/search?5000261=&language=1 . It matches the four authors, 2006 journal, volume 32(1), pages 43-49, and the 10 MW helium-helium IHX description. This is a publication record, not a full-text PDF.
2. `iaeacoupling2009`: Corrected an apparent report-number/year conflation. The cited title **Advances in Nuclear Power Process Heat Applications** belongs to **IAEA-TECDOC-1682 (2012)**, not TECDOC-1614 (2009). Added official PDF https://www-pub.iaea.org/MTCD/Publications/PDF/TE_1682_web.pdf .
3. `jaeri2004gthtr300c`: The linked JAERI-Review-2004-026 PDF exists and contains a workshop summary mentioning Kunitomi's GTHTR300C presentation. However, the BibTeX entry conflates the article/presentation title with the report-level title and possibly author attribution. Do not treat the proceedings as an independently authored Kunitomi/Yan monograph. Verify exact article pages/title or replace with a properly matched peer-reviewed reactor design reference, preserving any distinct 170 MW IHX source basis.

## Remaining work
The earlier independent link audit classifies many DOI redirects and web fetch restrictions as **unconfirmed**, not broken. Independently resolve each DOI against publisher metadata and compare titles/authors/year, check all direct links for publication identity, and update only verifiable metadata. Prioritize the 600 MWth / 170 MWth reactor design source, INL TEV-953/961, 2023 INL cost report, IAEA, and IEAGHG.
Rebuild BibTeX/LaTeX, inspect link formatting and ensure CI success. Do not invent or silently substitute sources.
