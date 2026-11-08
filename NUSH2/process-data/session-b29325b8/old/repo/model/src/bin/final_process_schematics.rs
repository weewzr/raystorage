use nuclear_assisted_smr::final_design::{HE_RETURN_C,HE_SUPPLY_C,HEAT_MWTH,H2_MMSCFD,INL_ROT_C,PSA_RECOVERY,REACTOR_MWTH,REFORMER_C};
use nuclear_assisted_smr::schematic_art::*;
use std::{fmt::Write,fs};

fn fig3()->String {
 let mut s=String::new(); begin(&mut s);
 let rx=Equip{c:Pt{x:1.2,y:4.7},w:1.7,h:2.6}; let ihx=Equip{c:Pt{x:4.2,y:4.7},w:1.6,h:2.0}; let smr=Equip{c:Pt{x:7.4,y:4.7},w:2.0,h:2.6};
 let wgs=Equip{c:Pt{x:10.1,y:4.7},w:1.55,h:1.9}; let cap=Equip{c:Pt{x:12.6,y:4.7},w:1.8,h:2.15}; let psae=Equip{c:Pt{x:12.6,y:1.75},w:1.9,h:1.8}; let comp=Equip{c:Pt{x:9.5,y:1.35},w:1.6,h:1.35};
 reactor(&mut s,rx,"HTGR"); exchanger(&mut s,ihx,"IHX"); reformer(&mut s,smr,"SMR"); vessel(&mut s,wgs,"WGS"); capture(&mut s,cap,"CO$_2$ capture"); psa(&mut s,psae,"PSA"); compressor(&mut s,comp,"CO$_2$ conditioning");
 let rx_hot=rx.at_right(0.42); let rx_cold=rx.at_right(-0.42); let ihx_pi=ihx.at_left(0.38); let ihx_po=ihx.at_left(-0.38); let ihx_so=ihx.at_right(0.38); let ihx_si=ihx.at_right(-0.38); let smr_hi=smr.at_left(0.42); let smr_ho=smr.at_left(-0.42);
 pipe(&mut s,rx_hot,ihx_pi,"primaryhe",Some(&format!("{:.0}$^\\circ$C",INL_ROT_C))); ortho(&mut s,ihx_po,2.75,rx_cold,"primaryhe",Some("primary-He return"));
 pipe(&mut s,ihx_so,smr_hi,"secondaryhe",Some(&format!("{:.0}$^\\circ$C; 78.49 kg/s",HE_SUPPLY_C))); ortho(&mut s,smr_ho,5.75,ihx_si,"secondaryhe",Some(&format!("return $\\approx${:.0}$^\\circ$C",HE_RETURN_C)));
 let feed=Pt{x:7.4,y:7.15}; pipe(&mut s,feed,smr.top(),"processgas",Some("NG + steam; S/C=3.0")); pipe(&mut s,smr.right(),wgs.left(),"processgas",None); pipe(&mut s,wgs.right(),cap.left(),"processgas",None);
 ortho(&mut s,cap.right(),14.25,psae.right(),"processgas",Some("H$_2$-rich gas")); let h2end=Pt{x:15.25,y:1.75}; pipe(&mut s,psae.right(),h2end,"h2stream",Some(&format!("H$_2$ {:.0} MMSCFD",H2_MMSCFD)));
 ortho(&mut s,cap.bottom(),11.1,comp.right(),"co2stream",Some("captured CO$_2$")); let ts=Pt{x:7.65,y:1.35}; pipe(&mut s,comp.left(),ts,"co2stream",Some("conditional T\\&S"));
 for q in [rx_hot,rx_cold,ihx_pi,ihx_po,ihx_so,ihx_si,smr_hi,smr_ho,smr.top(),smr.right(),wgs.left(),wgs.right(),cap.left(),cap.right(),cap.bottom(),psae.right(),comp.left(),comp.right()] { writeln!(&mut s,r"\fill[equipstroke] ({:.2},{:.2}) circle (1.15pt);",q.x,q.y).unwrap(); }
 callout(&mut s,Pt{x:1.2,y:6.55},&format!("600 MWth basis\\\\Remaining thermal capacity: {:.1} MWth\\\\(not electricity output)",REACTOR_MWTH-HEAT_MWTH));
 callout(&mut s,Pt{x:7.4,y:6.35},&format!("871$^\\circ$C outlet\\\\{:.1} MWth process heat",HEAT_MWTH)); callout(&mut s,Pt{x:12.6,y:0.35},&format!("{:.0}\\% H$_2$ recovery",PSA_RECOVERY*100.0));
 writeln!(&mut s,r"\draw[dashed,equipstroke!65] (5.25,0.1)--(5.25,7.5); \node[font=\sffamily\bfseries\scriptsize] at (2.55,7.55) {{NUCLEAR PRIMARY + IHX INTERFACE}}; \node[font=\sffamily\bfseries\scriptsize] at (9.8,7.55) {{SECONDARY HEAT LOOP + CHEMICAL PROCESS}};").unwrap();
 legend(&mut s,12.25,-0.25,&[("primaryhe","Primary He"),("secondaryhe","Secondary He"),("processgas","Process gas"),("co2stream","CO$_2$"),("h2stream","H$_2$")]); end(&mut s); s
}

fn fig1()->String {
 let mut s=String::new(); begin(&mut s); let furnace=Equip{c:Pt{x:1.1,y:4.4},w:1.6,h:1.6}; let smr=Equip{c:Pt{x:4.0,y:4.4},w:2.0,h:2.4}; let wgs=Equip{c:Pt{x:6.8,y:4.4},w:1.55,h:1.8}; let cap=Equip{c:Pt{x:9.4,y:4.4},w:1.8,h:2.1}; let psae=Equip{c:Pt{x:12.0,y:4.4},w:1.9,h:1.8}; let comp=Equip{c:Pt{x:9.4,y:1.45},w:1.6,h:1.3};
 vessel(&mut s,furnace,"Fired furnace"); reformer(&mut s,smr,"SMR"); vessel(&mut s,wgs,"WGS"); capture(&mut s,cap,"CO$_2$ capture"); psa(&mut s,psae,"PSA"); compressor(&mut s,comp,"CO$_2$ conditioning");
 pipe(&mut s,furnace.right(),smr.left(),"processgas",Some("fossil-fired heat + combustion CO$_2$")); let feed=Pt{x:4.0,y:6.8}; pipe(&mut s,feed,smr.top(),"processgas",Some("natural gas + steam")); pipe(&mut s,smr.right(),wgs.left(),"processgas",None); pipe(&mut s,wgs.right(),cap.left(),"processgas",None); pipe(&mut s,cap.right(),psae.left(),"processgas",None);
 let h2=Pt{x:14.1,y:4.4}; pipe(&mut s,psae.right(),h2,"h2stream",Some("H$_2$ product")); ortho(&mut s,cap.bottom(),9.4,comp.top(),"co2stream",Some("captured CO$_2$")); let ts=Pt{x:6.8,y:1.45}; pipe(&mut s,comp.left(),ts,"co2stream",Some("T\\&S")); legend(&mut s,11.1,0.15,&[("processgas","Process / heat"),("co2stream","CO$_2$"),("h2stream","H$_2$")]); end(&mut s); s
}

fn fig2()->String {
 let mut s=String::new(); begin(&mut s); let rx=Equip{c:Pt{x:1.3,y:4.6},w:1.7,h:2.6}; let sg=Equip{c:Pt{x:4.4,y:4.6},w:1.7,h:2.1}; let turb=Equip{c:Pt{x:7.7,y:4.6},w:2.0,h:1.5}; let gen=Equip{c:Pt{x:10.5,y:4.6},w:1.7,h:1.7}; let cond=Equip{c:Pt{x:7.7,y:1.55},w:2.3,h:1.45}; let pum=Equip{c:Pt{x:4.4,y:1.55},w:1.4,h:1.4};
 reactor(&mut s,rx,"HTGR"); exchanger(&mut s,sg,"Steam generator"); turbine(&mut s,turb,"Turbine"); generator(&mut s,gen,"G"); condenser(&mut s,cond,"Condenser"); pump(&mut s,pum,"Pump");
 pipe(&mut s,rx.at_right(0.42),sg.at_left(0.38),"primaryhe",Some("primary He")); ortho(&mut s,sg.at_left(-0.38),2.85,rx.at_right(-0.42),"primaryhe",Some("He return")); pipe(&mut s,sg.right(),turb.left(),"waterstream",Some("steam")); shaft(&mut s,turb.right(),gen.left());
 let elec=Pt{x:12.8,y:4.6}; pipe(&mut s,gen.right(),elec,"electricstream",Some("electricity")); ortho(&mut s,turb.bottom(),7.7,cond.top(),"waterstream",Some("exhaust steam")); pipe(&mut s,cond.left(),pum.right(),"waterstream",Some("condensate")); ortho(&mut s,pum.top(),4.4,sg.bottom(),"waterstream",Some("feedwater"));
 callout(&mut s,Pt{x:8.9,y:6.55},"Reference electricity-generation pathway\\\\not project electricity output"); legend(&mut s,9.8,0.15,&[("primaryhe","Primary He"),("waterstream","Steam / water"),("electricstream","Electricity")]); end(&mut s); s
}

fn main(){ let dir="../results/final_design/generated"; fs::create_dir_all(dir).unwrap(); fs::write(format!("{dir}/figure1_conventional_smr_ccs.tex"),fig1()).unwrap(); fs::write(format!("{dir}/figure2_htgr_power.tex"),fig2()).unwrap(); fs::write(format!("{dir}/figure3_proposed_htgr_smr_ccs.tex"),fig3()).unwrap(); }