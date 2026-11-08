W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
V = "54a7ac4a0553e1b23ad16f5fc90f1bce7bb4bb16"
vp(1440, 900)
out("inputs_ts", ev(r"""Array.from(document.querySelectorAll('[data-param="ts_s_t"]')).map(x=>x.outerHTML.slice(0,220))"""))
out("basis_ctrl", ev(r"""Array.from(document.querySelectorAll('[data-param="basis"]')).map(x=>x.outerHTML.slice(0,300))"""))
# change basis to case_a through the real select element with keyboard
ev(r"""document.querySelector('select[data-param="basis"]').focus()""")
key("ArrowDown")
sleep(2.5)
out("basis_now", ev(r"""document.querySelector('select[data-param="basis"]').value"""))
ev("location.hash = '#dashboard'")
sleep(1)
out("radios", ev("Array.from(document.querySelectorAll('input[name=scenario]')).map(e=>[e.value,e.checked])"))
out("tiles", ev(r"""Array.from(document.querySelectorAll('#kpi-strip [data-kpi^=cost], #kpi-strip [data-kpi=ng_mmscfd], #kpi-strip [data-kpi=h2_product_t_y], #kpi-strip [data-kpi=surplus_mwth]')).map(e=>[e.dataset.kpi,e.innerText.replace(/\s+/g,' ')])"""))
out("note", ev("(document.querySelector('[data-scenario-note]')||{}).innerText"))
out("targets", ev("document.getElementById('targets').innerText"))
out("feas", ev("document.getElementById('feasibility').hidden + ' | ' + document.getElementById('feasibility').innerText"))
out("basisline", ev("document.getElementById('basis-line').innerText"))
shot(W + "/shots/dash_mature_casea_1440.png", False)
