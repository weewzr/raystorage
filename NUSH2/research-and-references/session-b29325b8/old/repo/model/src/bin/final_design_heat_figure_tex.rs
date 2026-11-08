use nuclear_assisted_smr::final_design::{HEAT_MWTH,REACTOR_MWTH,NISHIHARA_IHX_MWTH};
fn main(){
 let residual=REACTOR_MWTH-HEAT_MWTH;
 println!(r#"\begin{{figure}}[htbp]\centering"#);
 println!(r#"\begin{{tikzpicture}}[x=0.014cm,y=1cm]"#);
 println!(r#"\draw[very thick] (0,2.4)--({:.1},2.4);"#,REACTOR_MWTH);
 println!(r#"\node[anchor=west] at (0,2.72) {{Reactor rating: {:.0} MWth}};"#,REACTOR_MWTH);
 println!(r#"\draw[very thick] (0,1.45)--({:.1},1.45);"#,NISHIHARA_IHX_MWTH);
 println!(r#"\node[anchor=west] at (0,1.77) {{Source heat branch: {:.0} MWth}};"#,NISHIHARA_IHX_MWTH);
 println!(r#"\draw[very thick] (0,0.5)--({:.1},0.5);"#,HEAT_MWTH);
 println!(r#"\node[anchor=west] at (0,0.82) {{Process duty: {:.1} MWth}};"#,HEAT_MWTH);
 println!(r#"\node[anchor=west,align=left] at (0,-0.35) {{Remaining reactor capacity: {:.1} MWth thermal\\No project electricity output is inferred from this remainder.}};"#,residual);
 println!(r#"\end{{tikzpicture}}"#);
 println!(r#"\caption{{Rust-generated thermal-capacity comparison from canonical final-design constants. Bar length represents thermal capacity/duty on a common MWth scale.}}"#);
 println!(r#"\label{{fig:rust-heat-flow}}\end{{figure}}"#);
}