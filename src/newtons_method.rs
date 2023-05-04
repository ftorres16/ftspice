use std::collections::BTreeMap;

use crate::device;
use crate::gauss_lu;
use crate::linalg;
use crate::nonlinear_stamp;

const MAX_ITERS: i64 = 100;

const RELATIVE_TOLERANCE: f64 = 0.001;
const ABSOLUTE_TOLERANCE_V: f64 = 1e-3;
const ABSOLUTE_TOLERANCE_A: f64 = 1e-6;

const DAMPING_GAMMA: f64 = 1.3;
const DAMPING_K: f64 = 16.0;

#[derive(Debug)]
struct Err {
    v: f64,
    i: f64,
}
#[derive(Debug)]
struct Step {
    v: f64,
    i: f64,
}

pub fn solve(
    nodes: &BTreeMap<String, device::NodeType>,
    elems: &Vec<device::SpiceElem>,
    x: &mut Vec<f64>,
    a_mat: &Vec<Vec<f64>>,
    b_vec: &Vec<f64>,
    h_mat: &Vec<Vec<f64>>,
    g_vec: &Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
) -> i64 {
    let mut err = Err {
        v: f64::INFINITY,
        i: f64::INFINITY,
    };
    let mut step = Step {
        v: f64::INFINITY,
        i: f64::INFINITY,
    };

    let mut f0 = get_err_vec(x, a_mat, b_vec, h_mat, g_vec);
    let mut err_old = get_err_norm(nodes, &f0);
    let mut step_old = Step {
        v: err_old.v,
        i: err_old.i,
    };

    let mut n_iters = 0;

    while n_iters < MAX_ITERS && !convergence_condition(err, step, err_old, step_old) {
        let mut jf_mat = a_mat.clone();
        let mut b_temp = b_vec.clone();
        let mut x_proposed = x.clone();

        for elem in elems.iter() {
            nonlinear_stamp::load(&elem, nodes, &x_proposed, &mut jf_mat, &mut b_temp);
        }

        gauss_lu::solve(&mut jf_mat, &mut b_temp, &mut x_proposed);

        let step_proposed = linalg::vec_sub(&x_proposed, &x);
        let step_taken = dampen_step(&step_proposed);
        step = get_step_norm(nodes, &step_taken);

        let x_new = linalg::vec_add(&x, &step_taken);

        f0 = get_err_vec(&x_new, a_mat, b_vec, h_mat, g_vec);
        err = get_err_norm(nodes, &f0);
        if err.v.is_infinite() || err.i.is_infinite() {
            panic!("v_err or i_err diverged");
        }

        for i in 0..x_new.len() {
            x[i] = x_new[i];
        }

        n_iters += 1;
        err_old = Err { v: err.v, i: err.i };
        step_old = Step {
            v: step.v,
            i: step.i,
        };
    }

    n_iters
}

fn dampen_step(step: &Vec<f64>) -> Vec<f64> {
    step.iter()
        .map(|x| DAMPING_GAMMA / DAMPING_K * x.signum() * (DAMPING_K * x.abs()).ln_1p())
        .collect::<Vec<_>>()
}

fn get_err_vec(
    x: &Vec<f64>,
    a_mat: &Vec<Vec<f64>>,
    b_vec: &Vec<f64>,
    h_mat: &Vec<Vec<f64>>,
    g_vec: &Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
) -> Vec<f64> {
    let g_val = g_vec.iter().map(|g| g(x)).collect::<Vec<_>>();
    let h_times_g = linalg::mat_vec_prod(h_mat, &g_val);
    let a_times_x = linalg::mat_vec_prod(a_mat, x);
    let f = linalg::vec_add(&a_times_x, &h_times_g);

    linalg::vec_sub(&f, &b_vec)
}

fn get_err_norm(nodes: &BTreeMap<String, device::NodeType>, err_vec: &Vec<f64>) -> Err {
    let mut err = Err { v: 0.0, i: 0.0 };

    for (node_type, err_item) in nodes.values().zip(err_vec) {
        match node_type {
            device::NodeType::Voltage => {
                if err_item.abs() > err.v {
                    err.v = err_item.abs();
                }
            }
            device::NodeType::Current => {
                if err_item.abs() > err.i {
                    err.i = err_item.abs();
                }
            }
        }
    }

    err
}

fn get_step_norm(nodes: &BTreeMap<String, device::NodeType>, step_vec: &Vec<f64>) -> Step {
    let err = get_err_norm(nodes, step_vec);
    Step { v: err.v, i: err.i }
}

fn convergence_condition(err: Err, step: Step, err_old: Err, step_old: Step) -> bool {
    step.v < RELATIVE_TOLERANCE * step_old.v + ABSOLUTE_TOLERANCE_V
        && step.i < RELATIVE_TOLERANCE * step_old.i + ABSOLUTE_TOLERANCE_A
        && err.v < RELATIVE_TOLERANCE * err_old.v + ABSOLUTE_TOLERANCE_V
        && err.i < RELATIVE_TOLERANCE * err_old.i + ABSOLUTE_TOLERANCE_A
}
