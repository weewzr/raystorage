"""Count binary documents in the remote HEAD tree from the saved tree listing."""
import collections
import json

T = r"C:\Users\Alf\AppData\Local\Temp\claude\C--Ray-Claude-Ray-Debug-VS-Code\b29325b8-d1b9-49cd-bfe6-262261d7ffef\scratchpad\findings\old_repo_remote_tree.json"
t = json.load(open(T, encoding="utf-8-sig"))
c = collections.Counter()
s = collections.Counter()
for e in t["tree"]:
    if e["type"] != "blob":
        continue
    ext = e["path"].rsplit(".", 1)[-1].lower()
    top = e["path"].split("/")[0]
    if ext in ("pdf", "pptx", "docx", "xlsx", "xlsm", "xls", "png"):
        c[(top, ext)] += 1
        s[(top, ext)] += e.get("size", 0)
for k in sorted(c):
    print(k, c[k], round(s[k] / 1e6, 1), "MB")
print("total blobs", sum(1 for e in t["tree"] if e["type"] == "blob"), "total MB", round(sum(e.get("size", 0) for e in t["tree"] if e["type"] == "blob") / 1e6, 1))
