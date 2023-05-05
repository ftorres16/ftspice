#[derive(Debug)]
pub struct NPN {
    pub vc: f64,
    pub vb: f64,
    pub ve: f64,
}

const IES: f64 = 2e-14;
const ICS: f64 = 99e-14;
const VTE: f64 = 26e-3;
const VTC: f64 = 26e-3;
const AR: f64 = 0.02;
const AF: f64 = 0.99;

impl NPN {
    pub fn vbe(&self) -> f64 {
        self.vb - self.ve
    }

    pub fn vbc(&self) -> f64 {
        self.vb - self.vc
    }

    pub fn ie(&self) -> f64 {
        -IES * (self.vbe() / VTE).exp_m1() + AR * ICS * (self.vbc() / VTC).exp_m1()
    }

    pub fn ic(&self) -> f64 {
        AF * IES * (self.vbe() / VTE).exp_m1() - ICS * (self.vbc() / VTC).exp_m1()
    }

    pub fn ib(&self) -> f64 {
        -(self.ie() + self.ic())
    }

    pub fn gee(&self) -> f64 {
        IES / VTE * (self.vbe() / VTE).exp()
    }
    pub fn gec(&self) -> f64 {
        AR * ICS / VTC * (self.vbc() / VTC).exp()
    }
    pub fn gce(&self) -> f64 {
        AF * IES / VTE * (self.vbe() / VTE).exp()
    }
    pub fn gcc(&self) -> f64 {
        ICS / VTC * (self.vbc() / VTC).exp()
    }

    pub fn ie_eq(&self) -> f64 {
        self.ie() + self.gee() * self.vbe() - self.gec() * self.vbc()
    }
    pub fn ic_eq(&self) -> f64 {
        self.ic() - self.gce() * self.vbe() + self.gcc() * self.vbc()
    }
}
