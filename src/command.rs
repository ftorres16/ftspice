#[derive(Debug)]
pub enum Command {
    Op,
    DC(DCParams),
    Tran(TranParams),
}

#[derive(Debug)]
pub struct DCParams {
    pub source: String,
    pub start: f64,
    pub stop: f64,
    pub step: f64,
}

#[derive(Debug, Clone)]
pub struct TranParams {
    pub start: f64,
    pub stop: f64,
    pub step: f64,
}
