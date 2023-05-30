use ndarray::prelude::*;

use crate::device::Stamp;
use crate::engine::mna::MNA;
use crate::engine::newtons_method;
use crate::engine::node_vec_norm::NodeVecNorm;
use crate::node_collection::NodeCollection;

pub mod state_history;

pub const T_STEP_MIN: f64 = 1e-18;

const TOL_REL: f64 = 0.001;
const TOL_ABS_V: f64 = 1e-3;
const TOL_ABS_A: f64 = 1e-6;

pub fn step(
    nodes: &NodeCollection,
    elems: &mut Vec<Box<dyn Stamp>>,
    mna: &mut MNA,
    t: &f64,
    h: &f64,
    x: &mut Array1<f64>,
    state_hist: &mut state_history::StateHistory,
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

        state_hist.push(n_iters, &x, t + h);

        if n_iters >= newtons_method::MAX_ITERS {
            h /= 2.0;
        } else if state_hist.len() < 4 {
            next_h = h;
            step_accepted = true;
        } else {
            let plte = state_hist.plte(state_hist.len() - 2);

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

fn plte_is_too_big(plte: &NodeVecNorm, x: &NodeVecNorm) -> bool {
    plte.v > x.v * TOL_REL + TOL_ABS_V || plte.i > x.i * TOL_REL + TOL_ABS_A
}

fn plte_can_grow(plte: &NodeVecNorm) -> bool {
    plte.v < 0.1 * TOL_ABS_V && plte.i < 0.1 * TOL_ABS_A
}
