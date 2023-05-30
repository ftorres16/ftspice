use ndarray::prelude::*;

use crate::device::Stamp;
use crate::engine::error::NotConvergedError;
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
) -> Result<(f64, f64), NotConvergedError> {
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

        match n_iters {
            Err(NotConvergedError) => {
                h /= 2.0;
                step_accepted = false;
            }
            Ok(n_iters) if state_hist.len() < 3 => {
                state_hist.push(n_iters, &x, t + h);
                next_h = h;
                step_accepted = true;
            }
            Ok(n_iters) => {
                state_hist.push(n_iters, &x, t + h);

                let plte = state_hist.plte(state_hist.len() - 2);
                let plte_norm = NodeVecNorm::new(nodes, &plte);
                let x_norm = NodeVecNorm::new(nodes, &x);

                step_accepted = !plte_is_too_big(&plte_norm, &x_norm);

                if !step_accepted {
                    h /= 2.0;
                    state_hist.pop();
                } else {
                    next_h = if plte_can_grow(&plte_norm) && h <= step_max / 2.0 {
                        h * 2.0
                    } else {
                        h
                    }
                }
            }
        }

        if !step_accepted {
            for elem in elems.iter_mut() {
                elem.undo_dynamic_stamp(&nodes, &x, &h, &mut mna.a, &mut mna.b);
            }
        }

        if h < T_STEP_MIN {
            return Err(NotConvergedError);
        }
    }

    mna.a = a_bkp;
    mna.b = b_bkp;

    Ok((h, next_h))
}

fn plte_is_too_big(plte: &NodeVecNorm, x: &NodeVecNorm) -> bool {
    plte.v > x.v * TOL_REL + TOL_ABS_V || plte.i > x.i * TOL_REL + TOL_ABS_A
}

fn plte_can_grow(plte: &NodeVecNorm) -> bool {
    plte.v < 0.1 * TOL_ABS_V && plte.i < 0.1 * TOL_ABS_A
}
