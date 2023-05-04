pub mod diode;

#[derive(Debug)]
pub enum DType {
    Vdd,
    Idd,
    Res,
    Diode,
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
