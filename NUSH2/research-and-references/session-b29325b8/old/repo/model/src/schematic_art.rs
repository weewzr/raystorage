//! Deterministic clean-room TikZ primitives for static engineering schematics.
//! Geometry is presentation-only: no equipment dimensions are design claims.

use std::fmt::Write;

#[derive(Clone, Copy)]
pub struct Pt { pub x: f64, pub y: f64 }
#[derive(Clone, Copy)]
pub struct Equip { pub c: Pt, pub w: f64, pub h: f64 }
impl Equip {
    pub fn left(self)->Pt { Pt{x:self.c.x-self.w/2.0,y:self.c.y} }
    pub fn right(self)->Pt { Pt{x:self.c.x+self.w/2.0,y:self.c.y} }
    pub fn top(self)->Pt { Pt{x:self.c.x,y:self.c.y+self.h/2.0} }
    pub fn bottom(self)->Pt { Pt{x:self.c.x,y:self.c.y-self.h/2.0} }
    pub fn at_left(self, frac:f64)->Pt { Pt{x:self.c.x-self.w/2.0,y:self.c.y+frac*self.h/2.0} }
    pub fn at_right(self, frac:f64)->Pt { Pt{x:self.c.x+self.w/2.0,y:self.c.y+frac*self.h/2.0} }
}
fn p(p:Pt)->String { format!("({:.2},{:.2})",p.x,p.y) }
pub fn begin(s:&mut String) {
    writeln!(s,r"\definecolor{{equipfill}}{{RGB}}{{238,241,244}}\definecolor{{equipstroke}}{{RGB}}{{70,78,86}}\definecolor{{coreaccent}}{{RGB}}{{232,151,72}}\definecolor{{primaryhe}}{{RGB}}{{196,63,54}}\definecolor{{secondaryhe}}{{RGB}}{{225,139,45}}\definecolor{{processgas}}{{RGB}}{{68,105,166}}\definecolor{{co2stream}}{{RGB}}{{95,99,104}}\definecolor{{h2stream}}{{RGB}}{{32,139,111}}\definecolor{{waterstream}}{{RGB}}{{54,137,189}}\definecolor{{electricstream}}{{RGB}}{{184,145,31}}").unwrap();
    writeln!(s,r"\begin{{tikzpicture}}[x=1cm,y=1cm,font=\sffamily\scriptsize,line cap=round,line join=round]").unwrap();
}
pub fn end(s:&mut String){ writeln!(s,r"\end{{tikzpicture}}").unwrap(); }
pub fn reactor(s:&mut String,e:Equip,label:&str){
    let x=e.c.x; let y=e.c.y; let w=e.w; let h=e.h;
    writeln!(s,r"\draw[fill=equipfill,draw=equipstroke,line width=.7pt,rounded corners=5pt] ({:.2},{:.2}) rectangle ({:.2},{:.2});",x-w/2.0,y-h/2.0,x+w/2.0,y+h/2.0).unwrap();
    writeln!(s,r"\draw[fill=coreaccent!45,draw=coreaccent!80!black] ({:.2},{:.2}) rectangle ({:.2},{:.2});",x-w*0.18,y-h*0.18,x+w*0.18,y+h*0.18).unwrap();
    writeln!(s,r"\node[align=center,font=\sffamily\bfseries\scriptsize] at ({:.2},{:.2}) {{{}}};",x,y-h*0.34,label).unwrap();
}
pub fn exchanger(s:&mut String,e:Equip,label:&str){
    let x=e.c.x; let y=e.c.y; let w=e.w; let h=e.h;
    writeln!(s,r"\draw[fill=equipfill,draw=equipstroke,line width=.7pt,rounded corners=2pt] ({:.2},{:.2}) rectangle ({:.2},{:.2});",x-w/2.0,y-h/2.0,x+w/2.0,y+h/2.0).unwrap();
    for k in [-0.25_f64,0.0,0.25] { writeln!(s,r"\draw[equipstroke] ({:.2},{:.2}) -- ({:.2},{:.2});",x-w*0.32,y+k*h,x+w*0.32,y+k*h).unwrap(); }
    writeln!(s,r"\node[font=\sffamily\bfseries\scriptsize] at ({:.2},{:.2}) {{{}}};",x,y,label).unwrap();
}
pub fn reformer(s:&mut String,e:Equip,label:&str){
    let x=e.c.x;let y=e.c.y;let w=e.w;let h=e.h;
    writeln!(s,r"\draw[fill=equipfill,draw=equipstroke,line width=.7pt] ({:.2},{:.2}) rectangle ({:.2},{:.2});",x-w/2.0,y-h/2.0,x+w/2.0,y+h/2.0).unwrap();
    for k in [-0.24_f64,0.0,0.24] { writeln!(s,r"\draw[coreaccent,line width=1pt] ({:.2},{:.2}) -- ({:.2},{:.2});",x+k*w,y-h*0.30,x+k*w,y+h*0.30).unwrap(); }
    writeln!(s,r"\node[font=\sffamily\bfseries\scriptsize] at ({:.2},{:.2}) {{{}}};",x,y-h*0.38,label).unwrap();
}
pub fn vessel(s:&mut String,e:Equip,label:&str){
    let x=e.c.x;let y=e.c.y;let w=e.w;let h=e.h;
    writeln!(s,r"\draw[fill=equipfill,draw=equipstroke,line width=.7pt,rounded corners=8pt] ({:.2},{:.2}) rectangle ({:.2},{:.2});",x-w/2.0,y-h/2.0,x+w/2.0,y+h/2.0).unwrap();
    writeln!(s,r"\node[align=center,font=\sffamily\bfseries\scriptsize] at ({:.2},{:.2}) {{{}}};",x,y,label).unwrap();
}
pub fn capture(s:&mut String,e:Equip,label:&str){
    let dx=e.w*0.20;
    for x in [e.c.x-dx,e.c.x+dx] { writeln!(s,r"\draw[fill=equipfill,draw=equipstroke,line width=.7pt,rounded corners=6pt] ({:.2},{:.2}) rectangle ({:.2},{:.2});",x-e.w*0.13,e.c.y-e.h/2.0,x+e.w*0.13,e.c.y+e.h/2.0).unwrap(); }
    writeln!(s,r"\node[align=center,font=\sffamily\bfseries\scriptsize] at ({:.2},{:.2}) {{{}}};",e.c.x,e.c.y-e.h*0.68,label).unwrap();
}
pub fn psa(s:&mut String,e:Equip,label:&str){
    for dx in [-0.28_f64,0.0,0.28] { let x=e.c.x+dx*e.w; writeln!(s,r"\draw[fill=equipfill,draw=equipstroke,line width=.7pt,rounded corners=5pt] ({:.2},{:.2}) rectangle ({:.2},{:.2});",x-e.w*0.09,e.c.y-e.h/2.0,x+e.w*0.09,e.c.y+e.h/2.0).unwrap(); }
    writeln!(s,r"\node[align=center,font=\sffamily\bfseries\scriptsize] at ({:.2},{:.2}) {{{}}};",e.c.x,e.c.y-e.h*0.68,label).unwrap();
}
pub fn compressor(s:&mut String,e:Equip,label:&str){
    writeln!(s,r"\draw[fill=equipfill,draw=equipstroke,line width=.7pt] ({:.2},{:.2}) -- ({:.2},{:.2}) -- ({:.2},{:.2}) -- cycle;",e.c.x-e.w/2.0,e.c.y-e.h*0.35,e.c.x+e.w/2.0,e.c.y-e.h/2.0,e.c.x+e.w/2.0,e.c.y+e.h/2.0).unwrap();
    writeln!(s,r"\node[align=center,font=\sffamily\bfseries\scriptsize] at ({:.2},{:.2}) {{{}}};",e.c.x,e.c.y-e.h*0.75,label).unwrap();
}
pub fn turbine(s:&mut String,e:Equip,label:&str){
    writeln!(s,r"\draw[fill=equipfill,draw=equipstroke,line width=.7pt] ({:.2},{:.2}) -- ({:.2},{:.2}) -- ({:.2},{:.2}) -- ({:.2},{:.2}) -- cycle;",e.c.x-e.w/2.0,e.c.y+e.h*0.25,e.c.x+e.w/2.0,e.c.y+e.h/2.0,e.c.x+e.w/2.0,e.c.y-e.h/2.0,e.c.x-e.w/2.0,e.c.y-e.h*0.25).unwrap();
    writeln!(s,r"\node[font=\sffamily\bfseries\scriptsize] at ({:.2},{:.2}) {{{}}};",e.c.x,e.c.y,label).unwrap();
}
pub fn generator(s:&mut String,e:Equip,label:&str){
    writeln!(s,r"\draw[fill=equipfill,draw=equipstroke,line width=.7pt] ({:.2},{:.2}) circle ({:.2});",e.c.x,e.c.y,e.w*0.35).unwrap();
    writeln!(s,r"\node[font=\sffamily\bfseries\scriptsize] at ({:.2},{:.2}) {{{}}};",e.c.x,e.c.y,label).unwrap();
}
pub fn condenser(s:&mut String,e:Equip,label:&str){
    writeln!(s,r"\draw[fill=equipfill,draw=equipstroke,line width=.7pt,rounded corners=2pt] ({:.2},{:.2}) rectangle ({:.2},{:.2});",e.c.x-e.w/2.0,e.c.y-e.h/2.0,e.c.x+e.w/2.0,e.c.y+e.h/2.0).unwrap();
    for dy in [-0.2_f64,0.0,0.2] { writeln!(s,r"\draw[waterstream] ({:.2},{:.2}) -- ({:.2},{:.2});",e.c.x-e.w*0.35,e.c.y+dy*e.h,e.c.x+e.w*0.35,e.c.y+dy*e.h).unwrap(); }
    writeln!(s,r"\node[font=\sffamily\bfseries\scriptsize] at ({:.2},{:.2}) {{{}}};",e.c.x,e.c.y-e.h*0.72,label).unwrap();
}
pub fn pump(s:&mut String,e:Equip,label:&str){
    writeln!(s,r"\draw[fill=equipfill,draw=equipstroke,line width=.7pt] ({:.2},{:.2}) circle ({:.2});",e.c.x,e.c.y,e.w*0.28).unwrap();
    writeln!(s,r"\draw[equipstroke] ({:.2},{:.2}) -- ({:.2},{:.2});",e.c.x,e.c.y,e.c.x+e.w*0.22,e.c.y+e.h*0.16).unwrap();
    writeln!(s,r"\node[font=\sffamily\scriptsize] at ({:.2},{:.2}) {{{}}};",e.c.x,e.c.y-e.h*0.65,label).unwrap();
}
pub fn pipe(s:&mut String,a:Pt,b:Pt,color:&str,label:Option<&str>){
    writeln!(s,r"\draw[{},line width=1.5pt,-{{Latex[length=2mm]}}] {} -- {};",color,p(a),p(b)).unwrap();
    if let Some(t)=label { writeln!(s,r"\node[fill=white,inner sep=1pt,font=\sffamily\scriptsize] at ({:.2},{:.2}) {{{}}};",(a.x+b.x)/2.0,(a.y+b.y)/2.0+0.18,t).unwrap(); }
}
pub fn ortho(s:&mut String,a:Pt,mid_x:f64,b:Pt,color:&str,label:Option<&str>){
    writeln!(s,r"\draw[{},line width=1.5pt,-{{Latex[length=2mm]}}] {} -- ({:.2},{:.2}) -- ({:.2},{:.2}) -- {};",color,p(a),mid_x,a.y,mid_x,b.y,p(b)).unwrap();
    if let Some(t)=label { writeln!(s,r"\node[fill=white,inner sep=1pt,font=\sffamily\scriptsize] at ({:.2},{:.2}) {{{}}};",mid_x+0.12,(a.y+b.y)/2.0,t).unwrap(); }
}
pub fn shaft(s:&mut String,a:Pt,b:Pt){ writeln!(s,r"\draw[equipstroke,line width=2.2pt] {} -- {};",p(a),p(b)).unwrap(); }
pub fn callout(s:&mut String,p0:Pt,text:&str){ writeln!(s,r"\node[align=left,font=\sffamily\scriptsize] at {} {{{}}};",p(p0),text).unwrap(); }
pub fn legend(s:&mut String,x:f64,y:f64,items:&[(&str,&str)]){
    let h=0.42*(items.len() as f64)+0.25;
    writeln!(s,r"\draw[fill=white,draw=equipstroke!55,rounded corners=2pt] ({:.2},{:.2}) rectangle ({:.2},{:.2});",x,y,x+2.8,y+h).unwrap();
    for (i,(color,name)) in items.iter().enumerate(){ let yy=y+h-0.35-(i as f64)*0.42; writeln!(s,r"\draw[{},line width=1.5pt] ({:.2},{:.2}) -- ({:.2},{:.2}); \node[anchor=west,font=\sffamily\scriptsize] at ({:.2},{:.2}) {{{}}};",color,x+0.18,yy,x+0.75,yy,x+0.88,yy,name).unwrap(); }
}
