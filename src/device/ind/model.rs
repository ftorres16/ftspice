#[derive(Debug)]
pub struct Model {
    pub vpos: f64,
    pub vneg: f64,
    pub val: f64,
    pub u_old: f64,
    pub i_old: f64,
}

impl Model {
    pub fn g_eq(&self, h: &f64) -> f64 {
        h / (2.0 * self.val)
    }

    pub fn i_eq(&self, h: &f64) -> f64 {
        self.i_old + self.g_eq(h) * self.u_old
    }

    pub fn u_new(&self) -> f64 {
        self.vpos - self.vneg
    }

    pub fn i_new(&self, h: &f64) -> f64 {
        self.g_eq(h) * (self.u_new() + self.u_old) + self.i_old
    }
}
