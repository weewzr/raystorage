# Crossref lookups; accept a DOI only when volume and first page (or article number) match.
import json, sys, urllib.parse, urllib.request, time
Q = [
 ("vinj_split", "Vinjarapu split flow MEA pilot Separation and Purification Technology", "345", "127311"),
 ("vinj_inter", "Vinjarapu interheating MEA pilot Energy", "320", "134973"),
 ("versteeg", "Versteeg van Swaaij kinetics CO2 alkanolamines aqueous non-aqueous primary secondary amines", "43", "573"),
 ("versteeg_henry", "Versteeg van Swaaij solubility diffusivity acid gases CO2 N2O aqueous alkanolamine", "33", "29"),
 ("carelli", "Carelli economic features of integral modular small-to-medium size reactors", "52", "403"),
 ("soave", "Soave equilibrium constants from a modified Redlich-Kwong equation of state", "27", "1197"),
 ("weiland", "Weiland density and viscosity of some partially carbonated aqueous alkanolamine solutions", "43", "378"),
 ("hikita", "Hikita kinetics of reactions of carbon dioxide with monoethanolamine diethanolamine triethanolamine rapid mixing", "13", "7"),
 ("penny", "Penny Ritter kinetic study of the reaction between carbon dioxide and primary amines", "79", "2103"),
 ("onda", "Onda Takeuchi Okumoto mass transfer coefficients between gas and liquid phases in packed columns", "1", "56"),
 ("hou", "Hou Hughes kinetics of methane steam reforming over Ni alpha-alumina catalyst", "82", "311"),
 ("billet", "Billet Schultes prediction of mass transfer columns with dumped and arranged packings", "77", "498"),
 ("rocha93", "Rocha Bravo Fair distillation columns containing structured packings comprehensive model hydraulics", "32", "641"),
 ("rocha96", "Rocha Bravo Fair distillation columns structured packings comprehensive model mass transfer", "35", "1660"),
 ("wilke", "Wilke Chang correlation of diffusion coefficients in dilute solutions", "1", "264"),
 ("jou", "Jou Mather Otto solubility of CO2 in a 30 mass percent monoethanolamine solution", "73", "140"),
 ("chinchen", "Chinchen Logan Spencer water-gas shift iron oxide chromium oxide mass transport", "12", "69"),
 ("kraussler", "Kraussler Hofbauer water gas shift kinetic model Fe Cr biomass steam gasification", "7", "153"),
 ("du2018", "Du molten salt HITEC shell and tube heat exchanger experimental YD325", None, None),
 ("guo2018", "Guo 2018 MEA CO2 capture hydrogen plant steam methane reforming", None, None),
 ("mea_thomsen", "Thomsen Rasmussen modeling of vapor-liquid-solid equilibrium in gas-aqueous electrolyte systems", "54", "1787"),
 ("nishihara", "Nishihara Mouri Kunitomi GTHTR300C hydrogen cogeneration", None, None),
 ("tochio", "Tochio Nakagawa thermal performance intermediate heat exchanger HTTR", "48", None),
]
out = {}
for key, q, vol, page in Q:
    url = "https://api.crossref.org/works?" + urllib.parse.urlencode({"query.bibliographic": q, "rows": 5,
          "select": "DOI,title,container-title,volume,page,article-number,issued,author"})
    try:
        data = json.load(urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": "refcheck/1.0"}), timeout=30))
    except Exception as e:
        out[key] = {"error": str(e)}; continue
    cands = []
    for it in data["message"]["items"]:
        v = it.get("volume"); p = (it.get("page") or "").split("-")[0]; a = it.get("article-number")
        ok = vol is not None and v == vol and (page in (p, a))
        cands.append({"doi": it["DOI"], "title": (it.get("title") or [""])[0][:120], "journal": (it.get("container-title") or [""])[0],
                      "vol": v, "page": it.get("page"), "art": a, "year": it.get("issued", {}).get("date-parts", [[None]])[0][0],
                      "first_author": ((it.get("author") or [{}])[0]).get("family"), "match": ok})
    out[key] = cands
    time.sleep(0.3)
json.dump(out, open(sys.argv[1], "w", encoding="utf-8"), indent=1, ensure_ascii=False)
for k, c in out.items():
    if isinstance(c, dict): print(k, c); continue
    m = [x for x in c if x["match"]]
    print(k, "MATCH" if m else "none", (m or c)[0])
