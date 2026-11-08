"""Write docs/ABATEMENT_SUBSIDY_REFERENCES.md from the Phase 2 researcher results."""
import json
import re
import sys

SRC, OUT = sys.argv[1], sys.argv[2]
res = json.load(open(SRC, encoding="utf-8"))
AREAS = [
    ("research:policy", "P", "Government co-funding and carbon price"),
    ("research:emission-factors", "E", "Emission factors and physical constants"),
    ("research:transport-storage", "T", "CO2 conditioning, shipping and storage"),
    ("research:finance-conventions", "F", "Financial conventions"),
]
QUARTER = re.compile(r"\bQ([1-4])\b")
PATH = re.compile(r"(?<![A-Za-z])[A-Za-z]:[/\\][^\s,;)\"']*")


def clean(text, letter=None):
    text = (text or "").replace("—", ", ").replace("–", "-")
    text = PATH.sub("<scratch>", text)
    text = QUARTER.sub(lambda m: ["first", "second", "third", "fourth"][int(m.group(1)) - 1] + " quarter", text)
    text = text.replace("user decision", "owner decision")
    text = text.replace("&amp;", "&").replace("|", "/").replace("\r", " ").replace("\n", " ")
    if letter:
        text = re.sub(r"\bR(\d+)\b", lambda m: f"{letter}{m.group(1)}", text)
    return re.sub(r"\s+", " ", text).strip()


L = []
w = L.append
w("# References for the CO2 accounting and the co-funding model")
w("")
w("Every entry was checked by a researcher with web access during Phase 2. The column Verified says how: `fetched_and_read` means the source document or page was opened and the cited place was read; `search_snippet_only` means only a search result was seen; `not_verified` means the existence or the content could not be confirmed. Values are paraphrased. An entry marked `search_snippet_only` or `not_verified` supports no parameter until it is read in full. Ids are prefixed by area: P for policy, E for emission factors, T for transport and storage, F for finance.")
w("")
for key, letter, title in AREAS:
    r = res[key]
    w(f"## {title}")
    w("")
    w(clean(r["summary"], letter))
    w("")
    w("| Id | Title | Issuer | Year | Link | Locator | Supports | Value | Verified | Note |")
    w("|---|---|---|---|---|---|---|---|---|---|")
    for x in r["refs"]:
        w("| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |".format(
            clean(x["id"], letter), clean(x["title"]), clean(x["issuer"]), clean(x["year"]),
            clean(x["link"]), clean(x["locator"]), clean(x["supports"]), clean(x.get("value", ""), letter),
            x["verified"], clean(x.get("note", ""), letter)))
    w("")
    w("Not found or not read:")
    w("")
    for n in r["not_found"]:
        w(f"- {clean(n, letter)}")
    w("")
    w("Recommendations from the researcher:")
    w("")
    for n in r["recommendations"]:
        w(f"- {clean(n, letter)}")
    w("")

text = "\n".join(L) + "\n"
assert "—" not in text and "–" not in text
open(OUT, "w", encoding="utf-8", newline="\n").write(text)
print("written", OUT, len(text), "chars")
