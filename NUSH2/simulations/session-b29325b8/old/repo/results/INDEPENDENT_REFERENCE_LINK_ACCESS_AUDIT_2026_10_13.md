# Independent bibliography link-access test (2026-10-13)

## Method and limits
Examined all 56 BibTeX records in `paper/references.bib` and attempted to open every supplied distinct direct URL and DOI through the available web retrieval system (duplicates reused). For sources with a retrieval error, the result is **unconfirmed**, not automatically a broken link. Additional search confirmed official IEAGHG and MTI landing pages. This is a browser-level retrieval and identity check, **not** a raw HTTP status-code crawl: the local runtime has no outbound DNS, DOI redirects and some large PDFs are blocked by the web retrieval tool.

## Result summary
- 56 entries examined.
- 23 source landing pages or original documents confirmed by accessible content or matching official search results.
- 10 URL links attempted but not confirmed.
- 15 DOI-backed links attempted but their redirect/content was not confirmed.
- 6 links returned fetch restrictions, size limits or cache misses.
- 2 entries lack any explicit URL/DOI.

## Individual results
| Citation key | Result | Supplied link |
|---|---|---|
| `ieaghg2017smr` | SOURCE PAGE CONFIRMED | https://ieaghg.org/publications/techno-economic-evaluation-of-smr-based-standalone-merchant-hydrogen-plant-with-ccs/ |
| `jaeahttr` | SOURCE PAGE CONFIRMED | https://www.jaea.go.jp/04/o-arai/nhc/jp/use_htgr/production.html |
| `nistwebbook` | URL UNCONFIRMED | https://webbook.nist.gov/chemistry/ |
| `iea2024hydrogen` | SOURCE PAGE CONFIRMED | https://www.iea.org/reports/global-hydrogen-review-2024 |
| `jaeri2004gthtr300c` | FETCH RESTRICTED | https://jopss.jaea.go.jp/pdfdata/JAERI-Review-2004-026.pdf |
| `mti2025nuclear` | URL UNCONFIRMED | https://www.mti.gov.sg/newsroom/singapore-energy-lecture-by-min-est--tan-see-leng-at-the-2025-singapore-international-energy-week/ |
| `mti2026inir` | SOURCE PAGE CONFIRMED | https://www.mti.gov.sg/newsroom/singapore-to-undertake-iaea-review-by-international-experts--to-assess-its-ability-to-make-informed-decision-on-nuclear-energy-deployment/ |
| `mti2025carbon` | URL UNCONFIRMED | https://www.mti.gov.sg/energy-and-carbon/carbon/ |
| `mti2025indonesiaccs` | URL UNCONFIRMED | https://www.mti.gov.sg/newsroom/singapore-and-indonesia-sign-three-memoranda-of-understanding-with-indonesia-on-cross-border-electricity-trade--carbon-capture-and-storage--and-sustainable-industrial-zone/ |
| `iea2026lng` | SOURCE PAGE CONFIRMED | https://www.iea.org/reports/assessing-emissions-from-lng-supply-and-abatement-options |
| `unece2022lca` | FETCH RESTRICTED | https://unece.org/sed/documents/2021/10/reports/life-cycle-assessment-electricity-generation-options |
| `emagrid2024` | SOURCE PAGE CONFIRMED | https://www.ema.gov.sg/resources/singapore-energy-statistics/chapter2 |
| `emaenergyprices` | SOURCE PAGE CONFIRMED | https://www.ema.gov.sg/resources/singapore-energy-statistics/chapter5 |
| `jaea2014economics` | SOURCE PAGE CONFIRMED | https://rdreview.jaea.go.jp/review_en/2014/e2014_6_3.html |
| `ema2024context` | SOURCE PAGE CONFIRMED | https://www.ema.gov.sg/resources/singapore-energy-statistics/chapter2 |
| `ema2024futurefund` | SOURCE PAGE CONFIRMED | https://www.ema.gov.sg/news-events/news/media-releases/2024/establishment-of-future-energy-fund-to-support-singapore-infrastructure-investments |
| `inlgain2024` | SOURCE PAGE CONFIRMED | https://gain.inl.gov/content/uploads/4/2024/08/AdvancedReactorCost_GAIN_2024.pdf |
| `takei2006gthtr` | DOI UNCONFIRMED | https://doi.org/10.3327/taesj2002.5.109 |
| `neacarbon2026` | SOURCE PAGE CONFIRMED | https://www.nea.gov.sg/our-services/climate-change-energy-efficiency/climate-change/carbon-tax |
| `mticcs2025cost` | URL UNCONFIRMED | https://www.mti.gov.sg/newsroom/written-reply-to-pq-on-projected-cost-per-tonne-of-carbon-dioxide-abated-through-carbon-capture-and-storage-in-singapore/ |
| `nishihara2007potential` | DOI UNCONFIRMED | https://doi.org/10.1299/jsmeicone.2007.15._ICONE1510_66 |
| `jaea2018excellent` | DOI UNCONFIRMED | https://doi.org/10.11484/jaea-technology-2018-004 |
| `jaea2007mockup` | URL UNCONFIRMED | https://doi.org/10.11484/jaea-technology-2007-022 |
| `worldbankdeflator` | SOURCE PAGE CONFIRMED | https://data.worldbank.org/indicator/NY.GDP.DEFL.ZS |
| `emausep2025` | SOURCE PAGE CONFIRMED | https://www.ema.gov.sg/resources/singapore-energy-statistics/chapter5 |
| `inltev961` | SOURCE PAGE CONFIRMED | https://art.inl.gov/NGNP/INL%20Documents/Year%202010/Sensitivity%20of%20Hydrogen%20Production%20via%20Steam%20Methane%20Reforming%20to%20High%20Temp%20Gas-Cooled%20Reactor%20Outlet%20Temp%20Process%20Analysis.pdf |
| `inltev953` | SOURCE PAGE CONFIRMED | https://art.inl.gov/NGNP/INL%20Documents/Year%202010/HTGR-Integrated%20Hydrogen%20Production%20via%20Steam%20Methane%20Reforming%20(SMR)%20Process%20Analysis%20rev%200.pdf |
| `jaeahttr2026ops` | SOURCE PAGE CONFIRMED | https://www.jaea.go.jp/04/o-arai/nhc/en/research/httr/index.html |
| `takamatsu2011lofc` | DOI UNCONFIRMED | https://doi.org/10.1299/jsmeicone.2011.19._ICONE1943_92 |
| `hamamoto2006ihx` | NO DIRECT LINK | (none) |
| `httrihx1995` | DOI UNCONFIRMED | https://doi.org/10.3327/jaesj.37.316 |
| `inlagrrelease` | URL UNCONFIRMED | https://art.inl.gov/report/agr-triso-fuel-fission-product-release-data-summary/ |
| `nrcngnppirt2010` | FETCH RESTRICTED | https://www.nrc.gov/regulations-legislation/nureg-series-publications/publications-prepared-by-nrc-contractors/next-generation-nuclear-plant-phenomena-identification-and-ranking-tables-pirts-nuregcr-6944/v6 |
| `iaeanuclearhydrogen2013` | FETCH RESTRICTED | https://www-pub.iaea.org/MTCD/Publications/PDF/Pub1577_web.pdf |
| `iaeacoupling2009` | NO DIRECT LINK | (none) |
| `mti2026safetycriteria` | URL UNCONFIRMED | https://www.mti.gov.sg/newsroom/written-reply-to-pq-on-safety-criteria-framework-study-on-decision-making-process-for-nuclear-energy-deployment/ |
| `ema2026safety` | SOURCE PAGE CONFIRMED | https://www.ema.gov.sg/news-events/news/forum-replies/2026/safety-in-using-nuclear-energy-is-overriding-priority |
| `mti2026carbon` | SOURCE PAGE CONFIRMED | https://www.mti.gov.sg/energy-and-carbon/carbon/ |
| `sawa2001httrfuel` | DOI UNCONFIRMED | https://doi.org/10.1016/S0029-5493(01)00358-2 |
| `sawaueta2004` | DOI UNCONFIRMED | https://doi.org/10.1016/j.nucengdes.2004.08.006 |
| `morris2016agr1` | DOI UNCONFIRMED | https://doi.org/10.1016/j.nucengdes.2016.04.031 |
| `demkowicz2015facs` | DOI UNCONFIRMED | https://doi.org/10.1016/j.jnucmat.2015.05.037 |
| `heliumsmr2022` | DOI UNCONFIRMED | https://doi.org/10.1016/j.pnucene.2022.104435 |
| `nrctriso6844` | FETCH RESTRICTED | https://www.nrc.gov/regulations-legislation/nureg-series-publications/publications-prepared-by-nrc-contractors/triso-coated-particle-fuel-phenomenon-identification-and-ranking-tables-pirts-for-fission-product/v1 |
| `nrcngnpfpt2008` | SOURCE PAGE CONFIRMED | https://www.nrc.gov/reading-rm/doc-collections/nuregs/contract/cr6944/v3/index |
| `dipu2016tritium` | DOI UNCONFIRMED | https://doi.org/10.1016/j.anucene.2015.10.028 |
| `takeda2000permeation` | DOI UNCONFIRMED | https://doi.org/10.3327/jaesj.42.204 |
| `ding2018epz` | DOI UNCONFIRMED | https://doi.org/10.1016/j.anucene.2017.08.039 |
| `mee2022htrpmepz` | FETCH RESTRICTED | https://www.mee.gov.cn/ywdt/gsgg/gongshi/wqgs_1/202206/W020220629349695162541.pdf |
| `mee2009htrpmreview` | SOURCE PAGE CONFIRMED | https://www.mee.gov.cn/gkml/hbb/bgsh/200910/t20091022_175356.htm |
| `jaea2024heatload` | URL UNCONFIRMED | https://www.jaea.go.jp/02/press2024/p24061201/ |
| `jaea2026heatuse` | SOURCE PAGE CONFIRMED | https://www.jaea.go.jp/04/o-arai/nhc/jp/use_htgr/production.html |
| `pubseawater2026` | SOURCE PAGE CONFIRMED | https://www.pub.gov.sg/Public/WaterLoop/Water-Conservation/Industrial-Water-Solutions |
| `devanand2019siting` | DOI UNCONFIRMED | https://doi.org/10.1016/j.compchemeng.2019.03.024 |
| `govsg2026western` | URL UNCONFIRMED | https://www.gov.sg/ndr2026/future-ready/ |
| `inl2023advancedcost` | DOI UNCONFIRMED | https://www.osti.gov/biblio/1986466 |

## Priority issues
1. **`jaeri2004gthtr300c`: bibliographic identity discrepancy.** The linked JAERI-Review 2004-026 appears in JAEA search as **HTTR Workshop, Workshop on Hydrogen Production Technology**, 206 pages, December 2004, rather than the BibTeX title *GTHTR300 for Hydrogen Cogeneration*. Verify the exact source used for 600 MWth/170 MWth before retaining this citation. The direct PDF was not retrieved.
2. **`hamamoto2006ihx` and `iaeacoupling2009`:** no link fields; find exact official publication/DOI and verify metadata.
3. **`iaeanuclearhydrogen2013` and `mee2022htrpmepz`:** PDF URLs returned 'content length too large'; this does NOT prove dead links.
4. **`unece2022lca`:** returned 403, possibly access policy rather than missing publication.
5. **`inl2023advancedcost`:** OSTI landing page not confirmed in this test; a different, clearly accessible 2024 GAIN INL report is linked under `inlgain2024` and must not be substituted silently for the cited 2023 work.
6. DOI redirects often failed within the retrieval tool. Resolve and match publication titles via Crossref/publisher before marking them valid or broken.

## Verified examples
- INL TEV-953 and TEV-961 original PDFs: opened successfully.
- IEAGHG 2017-02 SMR+CCS report: official publisher landing page independently found, with process and cost summary.
- IEA Global Hydrogen Review 2024: official page opened and title/year matched.
- JAEA HTTR heat-use page: opened and contents matched.
- Singapore EMA energy statistics and 2026 safety statement: opened.

## Next step
Repair only links whose correct publication identity is independently established; do not replace a citation with a vaguely similar document. For every unresolved item record publication title/year/DOI and direct access, or explicitly mark access-restricted. Rebuild bibliography after corrections.
