f="C:/Users/Alf/AppData/Local/Temp/claude/f97511ab42c3e41a/dee70430-6901-4bdf-a7a6-7fbb5c9a2058/scratchpad/phase3/fix2carbon/edit_log.py"
s=open(f,encoding="utf-8").read()
reps=[
("the floor abatement 481,083.69 t/y against 557,767.415516 (7.850175 against 8.995997 kg/kg)",
 "the net abatement 481,083.692586 t/y against 557,767.415516 (reference intensity 7.850175 against 8.995997 kg/kg)"),
("at the floor the target is missed at every stage, and definition A cannot reach it with any share.",
 "at the floor the cost without a grant is above S$100/t at every stage and life, and definition A cannot reach the target with any share."),
]
for a,b in reps:
    assert s.count(a)==1,a
    s=s.replace(a,b,1)
open(f,"w",encoding="utf-8").write(s)
print("ok")
