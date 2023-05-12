#[derive(Debug, Clone)]
pub enum SpiceFn {
    Pulse(PulseParams),
    Sine(SineParams),
}

impl SpiceFn {
    pub fn eval(&self, t: &f64) -> f64 {
        match &self {
            SpiceFn::Sine(p) => p.eval(t),
            SpiceFn::Pulse(p) => p.eval(t),
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

#[derive(Debug, Clone)]
pub struct PulseParams {
    pub v1: f64,
    pub v2: f64,
    pub delay: f64,
    pub t_rise: f64,
    pub t_fall: f64,
    pub pulse_width: f64,
    pub period: f64,
}

#[derive(Debug)]
enum PulseState {
    Waiting,
    Rising(f64),
    High,
    Falling(f64),
    Low,
}

impl PulseParams {
    fn get_state(&self, t: &f64) -> PulseState {
        let t_norm = (t % self.period).abs();

        if t_norm < self.delay {
            PulseState::Waiting
        } else if t_norm < self.delay + self.t_rise {
            let frac = (t_norm - self.delay) / self.t_rise;
            PulseState::Rising(frac)
        } else if t_norm < self.delay + self.pulse_width - self.t_fall {
            PulseState::High
        } else if t_norm < self.delay + self.pulse_width {
            let frac = (t_norm - (self.delay + self.pulse_width - self.t_fall)) / self.t_fall;
            PulseState::Falling(frac)
        } else {
            PulseState::Low
        }
    }

    fn eval(&self, t: &f64) -> f64 {
        let state = self.get_state(t);

        match state {
            PulseState::Waiting | PulseState::Low => self.v1,
            PulseState::Rising(frac) => self.v1 + frac * (self.v2 - self.v1),
            PulseState::High => self.v2,
            PulseState::Falling(frac) => self.v2 + frac * (self.v1 - self.v2),
        }
    }
}
