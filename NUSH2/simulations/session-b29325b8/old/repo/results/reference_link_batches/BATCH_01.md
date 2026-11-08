# Batch 01 — actual link tests (2026-10-13)

Method: live browser `open` on each supplied URL; page title and visible institutional metadata checked. Browser tool did not expose HTTP status codes or full redirect chains; none are invented. Original tested URL is the bibliography URL; final URLs below reflect observed landing pages where shown.

| Key | Status | Tested / final landing URL | Observed identity evidence and limitation |
|---|---|---|---|
| `ieaghg2017smr` | VERIFIED_DIRECT | https://ieaghg.org/publications/techno-economic-evaluation-of-smr-based-standalone-merchant-hydrogen-plant-with-ccs/ | Publisher report 2017-02, title/date/author and report summary matched; redirected to trailing slash. |
| `jaeahttr` | VERIFIED_DIRECT | https://www.jaea.go.jp/04/o-arai/nhc/jp/use_htgr/production.html | JAEA Japanese HTTR hydrogen heat-use programme page loaded; title and institutional identity match, webpage rather than report. |
| `nistwebbook` | VERIFIED_DIRECT | https://webbook.nist.gov/chemistry/ | NIST Chemistry WebBook landing loaded after trailing-slash redirect; individual species data not checked. |
| `iea2024hydrogen` | VERIFIED_DIRECT | https://www.iea.org/reports/global-hydrogen-review-2024 | IEA report page title/year matched. |
| `jaeri2004gthtr300c` | ACCESS_RESTRICTED | https://jopss.jaea.go.jp/pdfdata/JAERI-Review-2004-026.pdf | PDF open returned Internal Error; known workshop identity mismatch remains unresolved, not proof of broken link. |
| `mti2025nuclear` | VERIFIED_DIRECT | https://www.mti.gov.sg/newsroom/singapore-energy-lecture-by-min-est--tan-see-leng-at-the-2025-singapore-international-energy-week/ | MTI Singapore Energy Lecture title matched, not standalone nuclear report. |
| `mti2026inir` | VERIFIED_DIRECT | https://www.mti.gov.sg/newsroom/singapore-to-undertake-iaea-review-by-international-experts--to-assess-its-ability-to-make-informed-decision-on-nuclear-energy-deployment/ | MTI official IAEA review announcement loaded. |
| `mti2025carbon` | VERIFIED_DIRECT | https://www.mti.gov.sg/energy-and-carbon/carbon/ | Official MTI carbon overview loaded. |
| `mti2025indonesiaccs` | VERIFIED_DIRECT | https://www.mti.gov.sg/newsroom/singapore-and-indonesia-sign-three-memoranda-of-understanding-with-indonesia-on-cross-border-electricity-trade--carbon-capture-and-storage--and-sustainable-industrial-zone/ | Official MTI three-MOU statement loaded. |
| `iea2026lng` | VERIFIED_DIRECT | https://www.iea.org/reports/assessing-emissions-from-lng-supply-and-abatement-options | IEA report title matched. |

JAERI PDF access is unresolved; exact contribution pages and 170 MWth attribution must not be inferred. No BibTeX changes justified by this batch.
