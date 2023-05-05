pub mod diode;
pub mod npn;

#[derive(Debug)]
pub enum DType {
    Vdd,
    Idd,
    Res,
    Diode,
    NPN,
}

#[derive(Debug)]
pub enum RowType {
    Voltage,
    Current,
}

#[derive(Debug)]
pub struct SpiceElem {
    pub dtype: DType,
    pub name: String,
    pub nodes: Vec<String>,
    pub value: Option<f64>,
}
