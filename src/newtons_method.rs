use std::collections::BTreeMap;

use crate::device;
use crate::gauss_lu;
use crate::linalg;

const MAX_ITERS: i64 = 100;

const RELATIVE_TOLERANCE: f64 = 0.001;
const ABSOLUTE_TOLERANCE_V: f64 = 1e-3;
const ABSOLUTE_TOLERANCE_A: f64 = 1e-6;

const DAMPING_GAMMA: f64 = 1.3;
const DAMPING_K: f64 = 16.0;

pub fn solve(
    nodes: &BTreeMap<String, device::NodeType>,
    elems: &Vec<device::SpiceElem>,
    x: &mut Vec<f64>,
    a_mat: &Vec<Vec<f64>>,
    b_vec: &Vec<f64>,
    h_mat: &Vec<Vec<f64>>,
    g_vec: &Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
) -> i64 {
    let mut v_err = f64::INFINITY;
    let mut i_err = f64::INFINITY;
    let mut v_step_size = f64::INFINITY;
    let mut i_step_size = f64::INFINITY;

    let mut f0 = get_err_vec(x, a_mat, b_vec, h_mat, g_vec);
    let (mut v_err_old, mut i_err_old) = get_err_norms(nodes, &f0);
    let mut v_step_size_old = v_err_old;
    let mut i_step_size_old = i_err_old;

    let mut n_iters = 0;

    while n_iters < MAX_ITERS
        && (v_step_size >= RELATIVE_TOLERANCE * v_step_size_old + ABSOLUTE_TOLERANCE_V
            || i_step_size >= RELATIVE_TOLERANCE * i_step_size_old + ABSOLUTE_TOLERANCE_A
            || v_err >= RELATIVE_TOLERANCE * v_err_old + ABSOLUTE_TOLERANCE_V
            || i_err >= RELATIVE_TOLERANCE * i_err_old + ABSOLUTE_TOLERANCE_A)
    {
        let mut jf_mat = a_mat.clone();
        let mut b_temp = b_vec.clone();
        let mut x_proposed = x.clone();

        for elem in elems.iter() {
            elem.taylor_stamp(nodes, &x_proposed, &mut jf_mat, &mut b_temp);
        }

        gauss_lu::solve(&mut jf_mat, &mut b_temp, &mut x_proposed);

        let step_proposed = linalg::vec_sub(&x_proposed, &x);
        let step_taken = dampen_step(&step_proposed);
        (v_step_size, i_step_size) = get_err_norms(nodes, &step_taken);

        let x_new = linalg::vec_add(&x, &step_taken);

        f0 = get_err_vec(&x_new, a_mat, b_vec, h_mat, g_vec);
        (v_err, i_err) = get_err_norms(nodes, &f0);

        if v_err.is_infinite() || i_err.is_infinite() {
            panic!("v_err or i_err diverged");
        }

        for i in 0..x_new.len() {
            x[i] = x_new[i];
        }

        n_iters += 1;

        v_err_old = v_err;
        i_err_old = i_err;
        v_step_size_old = v_step_size;
        i_step_size_old = i_step_size;
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

fn get_err_norms(nodes: &BTreeMap<String, device::NodeType>, err_vec: &Vec<f64>) -> (f64, f64) {
    let mut v_err = 0.0;
    let mut i_err = 0.0;

    for (node_type, err) in nodes.values().zip(err_vec) {
        match node_type {
            device::NodeType::G1 => {
                if err.abs() > v_err {
                    v_err = err.abs();
                }
            }
            device::NodeType::G2 => {
                if err.abs() > i_err {
                    i_err = err.abs();
                }
            }
        }
    }
    (v_err, i_err)
}
