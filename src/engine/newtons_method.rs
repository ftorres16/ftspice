use ndarray::prelude::*;

use crate::device::Stamp;
use crate::engine::error::NotConvergedError;
use crate::engine::gauss_lu;
use crate::engine::mna::MNA;
use crate::engine::node_vec_norm::NodeVecNorm;
use crate::node_collection::NodeCollection;

pub const MAX_ITERS: u64 = 100;

const TOL_REL: f64 = 0.001;
const TOL_ABS_V: f64 = 1e-6;
const TOL_ABS_A: f64 = 1e-9;

const DAMPING_GAMMA: f64 = 1.3;
const DAMPING_K: f64 = 16.0;

pub fn solve(
    nodes: &NodeCollection,
    elems: &Vec<Box<dyn Stamp>>,
    x: &mut Array1<f64>,
    mna: &MNA,
) -> Result<u64, NotConvergedError> {
    let mut err = NodeVecNorm::infty();
    let mut step = NodeVecNorm::infty();

    let mut f0 = mna.get_err(x);
    let mut err_old = NodeVecNorm::new(nodes, &f0);
    let mut step_old = err_old.clone();

    let mut n_iters = 0;

    while n_iters < MAX_ITERS && !converged(&err, &step, &err_old, &step_old) {
        let mut jf_mat = mna.a.clone();
        let mut b_temp = mna.b.clone();
        let mut x_proposed = x.clone();

        for elem in elems.iter() {
            elem.nonlinear_stamp(&nodes, &x_proposed, &mut jf_mat, &mut b_temp);
        }

        gauss_lu::solve(&mut jf_mat, &mut b_temp, &mut x_proposed);

        let step_proposed = &x_proposed.view() - &x.view();
        let step_taken = dampen_step(&step_proposed);
        step = NodeVecNorm::new(nodes, &step_taken);

        let x_new = &x.view() + &step_taken;

        f0 = mna.get_err(&x_new);
        err = NodeVecNorm::new(nodes, &f0);
        if err.v.is_infinite() || err.i.is_infinite() {
            panic!("v_err or i_err diverged");
        }

        for i in 0..x_new.len() {
            x[i] = x_new[i];
        }

        n_iters += 1;
        err_old = err.clone();
        step_old = step.clone();
    }

    if n_iters < MAX_ITERS {
        Ok(n_iters)
    } else {
        Err(NotConvergedError)
    }
}

fn dampen_step(step: &Array1<f64>) -> Array1<f64> {
    step.iter()
        .map(|x| DAMPING_GAMMA / DAMPING_K * x.signum() * (DAMPING_K * x.abs()).ln_1p())
        .collect()
}

fn converged(
    err: &NodeVecNorm,
    step: &NodeVecNorm,
    err_old: &NodeVecNorm,
    step_old: &NodeVecNorm,
) -> bool {
    step.v < TOL_REL * step_old.v + TOL_ABS_V
        && step.i < TOL_REL * step_old.i + TOL_ABS_A
        && err.v < TOL_REL * err_old.v + TOL_ABS_V
        && err.i < TOL_REL * err_old.i + TOL_ABS_A
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dampen_step() {
        let step = array![1.0, 2.0, 3.0];
        let small_step = dampen_step(&step);

        for (old, new) in step.iter().zip(&small_step) {
            assert!(old > new);
        }
    }

    #[test]
    fn test_converged_success() {
        let err = NodeVecNorm { v: 1e-9, i: 1e-9 };
        let step = NodeVecNorm { v: 1e-9, i: 1e-9 };
        let err_old = NodeVecNorm { v: 1.0, i: 1.0 };
        let step_old = NodeVecNorm { v: 1.0, i: 1.0 };

        assert!(converged(&err, &step, &err_old, &step_old));
    }

    #[test]
    fn test_converged_fail_err_v() {
        let err = NodeVecNorm { v: 1.0, i: 1e-9 };
        let step = NodeVecNorm { v: 1e-9, i: 1e-9 };
        let err_old = NodeVecNorm { v: 1.0, i: 1.0 };
        let step_old = NodeVecNorm { v: 1.0, i: 1.0 };

        assert!(!converged(&err, &step, &err_old, &step_old));
    }

    #[test]
    fn test_converged_fail_err_i() {
        let err = NodeVecNorm { v: 1e-9, i: 1.0 };
        let step = NodeVecNorm { v: 1e-9, i: 1e-9 };
        let err_old = NodeVecNorm { v: 1.0, i: 1.0 };
        let step_old = NodeVecNorm { v: 1.0, i: 1.0 };

        assert!(!converged(&err, &step, &err_old, &step_old));
    }

    #[test]
    fn test_converged_fail_step_v() {
        let err = NodeVecNorm { v: 1e-9, i: 1e-9 };
        let step = NodeVecNorm { v: 1.0, i: 1e-9 };
        let err_old = NodeVecNorm { v: 1.0, i: 1.0 };
        let step_old = NodeVecNorm { v: 1.0, i: 1.0 };

        assert!(!converged(&err, &step, &err_old, &step_old));
    }

    #[test]
    fn test_converged_fail_step_i() {
        let err = NodeVecNorm { v: 1e-9, i: 1e-9 };
        let step = NodeVecNorm { v: 1e-9, i: 1.0 };
        let err_old = NodeVecNorm { v: 1.0, i: 1.0 };
        let step_old = NodeVecNorm { v: 1.0, i: 1.0 };

        assert!(!converged(&err, &step, &err_old, &step_old));
    }
}
