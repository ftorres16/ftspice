#[derive(Debug)]
pub struct Model {
    pub vd: f64,
    pub vg: f64,
    pub vs: f64,
}

#[derive(Debug)]
pub enum State {
    CutOff,
    Linear,
    Saturated,
}

const BETA: f64 = 0.5e-3;
const VT: f64 = 0.6;
const LAMBDA: f64 = 0.01;

impl Model {
    pub fn vgs(&self) -> f64 {
        self.vg - self.vs
    }

    pub fn vds(&self) -> f64 {
        self.vd - self.vs
    }

    pub fn state(&self) -> State {
        if self.vgs() <= VT {
            State::CutOff
        } else if 0.0 <= self.vds() && self.vds() <= self.vgs() - VT {
            State::Linear
        } else if 0.0 <= self.vgs() - VT && self.vgs() - VT <= self.vds() {
            State::Saturated
        } else {
            unreachable!()
        }
    }

    pub fn id(&self) -> f64 {
        match self.state() {
            State::CutOff => 0.0,
            State::Linear => BETA * ((self.vgs() - VT) * self.vds() - 0.5 * self.vds().powi(2)),
            State::Saturated => {
                0.5 * BETA * (self.vgs() - VT) * (self.vgs() - VT) * (1.0 + LAMBDA * self.vds())
            }
        }
    }

    pub fn is(&self) -> f64 {
        -self.id()
    }

    pub fn ig(&self) -> f64 {
        0.0
    }

    pub fn gds(&self) -> f64 {
        match self.state() {
            State::CutOff => 0.0,
            State::Linear => BETA * (self.vgs() - VT - self.vds()),
            State::Saturated => 0.5 * BETA * LAMBDA * (self.vgs() - VT).powi(2),
        }
    }

    pub fn gm(&self) -> f64 {
        match self.state() {
            State::CutOff => 0.0,
            State::Linear => BETA * self.vds(),
            State::Saturated => BETA * (self.vgs() - VT) * (1.0 + LAMBDA * self.vds()),
        }
    }

    pub fn ieq(&self) -> f64 {
        self.id() - self.gds() * self.vds() - self.gm() * self.vgs()
    }
}
