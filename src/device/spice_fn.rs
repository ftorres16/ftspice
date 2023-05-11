#[derive(Debug, Clone)]
pub enum SpiceFn {
    Sine(SineParams),
}

impl SpiceFn {
    pub fn eval(&self, t: &f64) -> f64 {
        match &self {
            SpiceFn::Sine(x) => x.eval(t),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SineParams {
    pub offset: f64,
    pub amplitude: f64,
    pub freq: f64,
}

impl SineParams {
    fn eval(&self, t: &f64) -> f64 {
        self.offset + self.amplitude * (2.0 * std::f64::consts::PI * self.freq * t).sin()
    }
}
