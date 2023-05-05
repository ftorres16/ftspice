#[derive(Debug)]
pub enum Command {
    Op,
    DC(DCParams),
}

#[derive(Debug)]
pub struct DCParams {
    pub source: String,
    pub start: f64,
    pub stop: f64,
    pub step: f64,
}
