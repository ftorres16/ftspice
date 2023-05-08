#[derive(Debug)]
pub struct Model {
    pub vpos: f64,
    pub vneg: f64,
}

const ISAT: f64 = 1.0e-12;
const ETA: f64 = 1.0;
const VT: f64 = 26e-3;

impl Model {
    pub fn i(&self) -> f64 {
        ISAT * ((self.vpos - self.vneg) / (ETA * VT)).exp_m1()
    }

    pub fn g_eq(&self) -> f64 {
        ISAT / (ETA * VT) * ((self.vpos - self.vneg) / (ETA * VT)).exp()
    }

    pub fn i_eq(&self) -> f64 {
        self.i() - self.g_eq() * (self.vpos - self.vneg)
    }
}
