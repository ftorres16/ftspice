use ndarray::prelude::*;

use crate::device::Stamp;
use crate::engine::mna::MNA;
use crate::engine::newtons_method;
use crate::engine::node_vec_norm::NodeVecNorm;
use crate::node_collection::NodeCollection;

pub const T_STEP_MIN: f64 = 1e-18;

const TOL_REL: f64 = 0.001;
const TOL_ABS_V: f64 = 1e-3;
const TOL_ABS_A: f64 = 1e-6;

#[derive(Debug)]
pub struct TranStateHistory {
    pub n_iters: u64,
    pub x: Array1<f64>,
    pub t: f64,
}

pub fn step(
    nodes: &NodeCollection,
    elems: &mut Vec<Box<dyn Stamp>>,
    mna: &mut MNA,
    t: &f64,
    h: &f64,
    x: &mut Array1<f64>,
    state_hist: &mut Vec<TranStateHistory>,
    step_max: &f64,
) -> (f64, f64) {
    let mut h = h.to_owned();
    let mut next_h = h;
    let mut step_accepted = false;

    let a_bkp = mna.a.clone();
    let b_bkp = mna.b.clone();

    while !step_accepted {
        for elem in elems.iter_mut() {
            if elem.has_tran() {
                elem.undo_linear_stamp(&nodes, &mut mna.a, &mut mna.b);
                elem.eval_tran(&(t + h));
                elem.linear_stamp(&nodes, &mut mna.a, &mut mna.b);
            }
        }

        for elem in elems.iter() {
            elem.dynamic_stamp(&nodes, &x, &h, &mut mna.a, &mut mna.b);
        }

        let n_iters = newtons_method::solve(nodes, elems, x, &mna);

        state_hist.push(TranStateHistory {
            n_iters,
            t: t + h,
            x: x.to_owned(),
        });

        if n_iters >= newtons_method::MAX_ITERS {
            h /= 2.0;
        } else if state_hist.len() < 4 {
            next_h = h;
            step_accepted = true;
        } else {
            let plte = plte_vec(state_hist, state_hist.len() - 2);

            let plte_norm = NodeVecNorm::new(nodes, &plte);
            let x_norm = NodeVecNorm::new(nodes, &x);

            if plte_is_too_big(&plte_norm, &x_norm) {
                h /= 2.0;
            } else {
                step_accepted = true;

                if plte_can_grow(&plte_norm) && h <= step_max / 2.0 {
                    next_h = h * 2.0;
                } else {
                    next_h = h;
                }
            }
        }

        if !step_accepted {
            for elem in elems.iter_mut() {
                elem.undo_dynamic_stamp(&nodes, &x, &h, &mut mna.a, &mut mna.b);
            }
            state_hist.pop();
        }

        if h < T_STEP_MIN {
            panic!("Timestep too small!");
        }
    }

    mna.a = a_bkp;
    mna.b = b_bkp;

    (h, next_h)
}

fn divided_diff(state_hist: &Vec<TranStateHistory>, n_max: usize, n_min: usize) -> Array1<f64> {
    if n_max == n_min {
        state_hist[n_max].x.clone()
    } else {
        (&divided_diff(state_hist, n_max, n_min + 1) - &divided_diff(state_hist, n_max - 1, n_min))
            / (state_hist[n_max].t - state_hist[n_min].t)
    }
}

fn plte_vec(state_hist: &Vec<TranStateHistory>, n: usize) -> Array1<f64> {
    let c3 = -1.0 / 12.0;
    let h_next = state_hist[n + 1].t - state_hist[n].t;

    &divided_diff(state_hist, n + 1, n - 2) * 6.0 * c3 * h_next.powi(3)
}

fn plte_is_too_big(plte: &NodeVecNorm, x: &NodeVecNorm) -> bool {
    plte.v > x.v * TOL_REL + TOL_ABS_V || plte.i > x.i * TOL_REL + TOL_ABS_A
}

fn plte_can_grow(plte: &NodeVecNorm) -> bool {
    plte.v < 0.1 * TOL_ABS_V && plte.i < 0.1 * TOL_ABS_A
}
