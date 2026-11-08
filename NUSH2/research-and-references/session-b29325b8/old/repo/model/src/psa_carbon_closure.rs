//! Evidence-gated elemental carbon closure for PSA and CCS process boundaries.
//! Unknown streams must be represented explicitly; no missing carbon is set to zero.
#[derive(Debug,Clone,Copy)]
pub struct CarbonStream {
    pub total_kmol_h:f64,
    pub co2:f64, pub co:f64, pub ch4:f64,
    pub c2h6:f64, pub c3h8:f64,
}
impl CarbonStream {
    pub fn carbon_kmol_h(self)->f64 {
        self.total_kmol_h*(self.co2+self.co+self.ch4+2.0*self.c2h6+3.0*self.c3h8)
    }
}
#[derive(Debug,Clone,Copy)]
pub struct CarbonClosure {
    pub input_kmol_c_h:f64,
    pub output_kmol_c_h:f64,
    pub residual_kmol_c_h:f64,
    pub relative_residual:f64,
}
/// None means an essential inlet or outlet is unknown, not zero.
pub fn close_carbon(inputs:&[Option<CarbonStream>],outputs:&[Option<CarbonStream>])->Option<CarbonClosure>{
    if inputs.is_empty() || outputs.is_empty() ||
       inputs.iter().chain(outputs.iter()).any(|x|x.is_none()) {return None;}
    let inp:f64=inputs.iter().map(|x|x.unwrap().carbon_kmol_h()).sum();
    let out:f64=outputs.iter().map(|x|x.unwrap().carbon_kmol_h()).sum();
    if !inp.is_finite() || !out.is_finite() || inp<=0.0 || out<0.0{return None;}
    Some(CarbonClosure{input_kmol_c_h:inp,output_kmol_c_h:out,
        residual_kmol_c_h:inp-out,relative_residual:(inp-out)/inp})
}
/// Only the ratio of two reported CO2 streams; NOT plant-wide capture.
pub fn reported_co2_stream_ratio(captured:f64,emitted:f64)->Option<f64>{
    if !captured.is_finite() || !emitted.is_finite() || captured<0.0 || emitted<0.0 ||
       captured+emitted<=0.0 {return None;}
    Some(captured/(captured+emitted))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn s(total:f64,co2:f64,co:f64,ch4:f64)->CarbonStream {
        CarbonStream{total_kmol_h:total,co2,co,ch4,c2h6:0.0,c3h8:0.0}
    }
    #[test]fn exact_closed_synthetic_case(){
        let r=close_carbon(&[Some(s(100.0,0.0,0.0,1.0))],
          &[Some(s(60.0,1.0,0.0,0.0)),Some(s(40.0,0.0,1.0,0.0))]).unwrap();
        assert!(r.residual_kmol_c_h.abs()<1e-12);
    }
    #[test]fn unclosed_synthetic_case_retains_residual(){
        let r=close_carbon(&[Some(s(100.0,0.0,0.0,1.0))],
          &[Some(s(90.0,1.0,0.0,0.0))]).unwrap();
        assert!((r.residual_kmol_c_h-10.0).abs()<1e-12);
        assert!((r.relative_residual-0.1).abs()<1e-12);
    }
    #[test]fn missing_tail_gas_never_assumed_zero(){
        assert!(close_carbon(&[Some(s(100.0,0.0,0.0,1.0))],
           &[Some(s(90.0,1.0,0.0,0.0)),None]).is_none());
    }
    #[test]fn reported_ratio_is_not_a_capture_efficiency(){
        let r=reported_co2_stream_ratio(1927.0,142.0).unwrap();
        assert!((r-1927.0/2069.0).abs()<1e-12);
    }
}
