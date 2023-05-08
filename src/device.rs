pub mod diode;
pub mod nmos;
pub mod npn;

#[derive(Debug, Clone)]
pub enum DType {
    Vdd,
    Idd,
    Res,
    Diode,
    NPN,
    NMOS,
}

#[derive(Debug, Clone)]
pub struct SpiceElem {
    pub dtype: DType,
    pub name: String,
    pub nodes: Vec<String>,
    pub value: Option<f64>,
}
