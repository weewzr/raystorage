use nuclear_assisted_smr::final_design::{HE_RETURN_C, HE_SUPPLY_C, HEAT_MWTH, H2_MMSCFD, INL_ROT_C, PSA_RECOVERY, REACTOR_MWTH, REFORMER_C};
use std::{fmt::Write, fs};
fn main() {
 let remaining = REACTOR_MWTH - HEAT_MWTH;
 let mut s = String::new();
 writeln!(s, r"\begin{{tikzpicture}}[>=Latex,font=\small,equip/.style={{draw,rounded corners,align=center,minimum width=20mm,minimum height=11mm}},flow/.style={{->,thick}},heat/.style={{->,very thick,dashed}},stream/.style={{align=center,font=\scriptsize}}]").unwrap();
 writeln!(s, r"\node[equip] (rx) at (0,3.2) {{HTGR\\{:.0} MWth}};", REACTOR_MWTH).unwrap();
 writeln!(s, r"\node[equip] (ihx) at (3.0,3.2) {{IHX}};").unwrap();
 writeln!(s, r"\node[equip] (smr) at (6.2,3.2) {{SMR\\{:.0}$^\circ$C outlet\\{:.1} MWth}};", REFORMER_C, HEAT_MWTH).unwrap();
 writeln!(s, r"\node[equip] (wgs) at (9.2,3.2) {{WGS}};").unwrap();
 writeln!(s, r"\node[equip] (ccs) at (12.2,3.2) {{CO$_2$ capture}};").unwrap();
 writeln!(s, r"\node[equip] (psa) at (9.2,0.8) {{PSA\\{:.0}\% H$_2$ recovery}};", PSA_RECOVERY*100.0).unwrap();
 writeln!(s, r"\node[equip] (h2) at (6.2,0.8) {{H$_2$ product\\{:.0} MMSCFD}};", H2_MMSCFD).unwrap();
 writeln!(s, r"\node[equip] (co2) at (12.2,0.8) {{Compression / conditioning\\$\rightarrow$ conditional T\&S}};").unwrap();
 writeln!(s, r"\node[stream] (feed) at (6.2,5.0) {{Natural gas + steam\\S/C = 3.0}};").unwrap();
 writeln!(s, r"\node[stream] (remain) at (0,0.8) {{Remaining thermal capacity\\{:.1} MWth\\not electricity output}};", remaining).unwrap();
 writeln!(s, r"\draw[heat] (rx)-- node[above,stream]{{primary He\\{:.0}$^\circ$C}} (ihx);", INL_ROT_C).unwrap();
 writeln!(s, r"\draw[heat] (ihx)-- node[above,stream]{{secondary He\\{:.0}$^\circ$C; 78.49 kg/s}} (smr);", HE_SUPPLY_C).unwrap();
 writeln!(s, r"\draw[heat] (smr.south west) to[out=225,in=315] node[below,stream]{{secondary-He return $\approx${:.0}$^\circ$C}} (ihx.south east);", HE_RETURN_C).unwrap();
 writeln!(s, r"\draw[heat] (ihx.north west) to[out=135,in=45] node[above,stream]{{primary-He return}} (rx.north east);").unwrap();
 writeln!(s, r"\draw[flow] (feed)--(smr); \draw[flow] (smr)--(wgs); \draw[flow] (wgs)--(ccs); \draw[flow] (ccs) |- (psa); \draw[flow] (psa)--(h2); \draw[flow] (ccs)--(co2);").unwrap();
 writeln!(s, r"\draw[dashed] (1.55,-0.35) -- (1.55,5.25);").unwrap();
 writeln!(s, r"\node[stream] at (0,5.35) {{\textbf{{NUCLEAR PRIMARY}}}};").unwrap();
 writeln!(s, r"\node[stream] at (7.2,5.35) {{\textbf{{SECONDARY HEAT LOOP + CHEMICAL PROCESS}}}};").unwrap();
 writeln!(s, r"\end{{tikzpicture}}").unwrap();
 fs::write("../results/final_design/generated/final_integrated_process_figure.tex", s).unwrap();
}