use crate::device::Stamp;
use crate::engine::linalg;
use crate::engine::mna::MNA;
use crate::engine::newtons_method;
use crate::engine::node_vec_norm::NodeVecNorm;
use crate::node::NodeCollection;

pub const T_STEP_MIN: f64 = 1e-12;

const TOL_REL: f64 = 0.001;
const TOL_ABS_V: f64 = 1e-3;
const TOL_ABS_A: f64 = 1e-6;

pub fn step(
    nodes: &NodeCollection,
    elems: &mut Vec<Box<dyn Stamp>>,
    t: &f64,
    h: &f64,
    x: &mut Vec<f64>,
    mna: &mut MNA,
    in_src_idx: &usize,
    x_hist: &mut Vec<Vec<f64>>,
    t_hist: &mut Vec<f64>,
    step_max: &f64,
) -> (f64, f64, u64) {
    let mut n_iters = 0;
    let mut h = h.to_owned();
    let mut next_h = h;
    let mut step_accepted = false;

    let in_src_idx = in_src_idx.to_owned();
    let a_bkp = mna.a.clone();
    let b_bkp = mna.b.clone();
    let in_src_bkp = elems[in_src_idx].get_value();

    while !step_accepted {
        elems[in_src_idx].undo_linear_stamp(&nodes, &mut mna.a, &mut mna.b);
        elems[in_src_idx].set_value(in_src_voltage(&(t + h)));
        elems[in_src_idx].linear_stamp(&nodes, &mut mna.a, &mut mna.b);

        for elem in elems.iter() {
            elem.dynamic_stamp(&nodes, &x, &h, &mut mna.a, &mut mna.b);
        }

        n_iters = newtons_method::solve(nodes, elems, x, &mna);

        if n_iters >= newtons_method::MAX_ITERS {
            for elem in elems.iter() {
                elem.undo_dynamic_stamp(&nodes, &x, &h, &mut mna.a, &mut mna.b);
            }
            h /= 2.0;
        } else if x_hist.len() < 4 {
            next_h = h;
            step_accepted = true;
        } else {
            t_hist.push(t + h);
            x_hist.push(x.to_owned());
            let plte = plte_vec(x_hist, t_hist, x_hist.len() - 2);
            t_hist.pop();
            x_hist.pop();

            let plte_norm = NodeVecNorm::new(nodes, &plte);
            let x_norm = NodeVecNorm::new(nodes, &x);

            if plte_is_too_big(&plte_norm, &x_norm) {
                for elem in elems.iter() {
                    elem.undo_dynamic_stamp(&nodes, &x, &h, &mut mna.a, &mut mna.b);
                }

                h /= 2.0;
            } else {
                step_accepted = true;

                if plte_is_small(&plte_norm) && h <= step_max / 2.0 {
                    next_h = h * 2.0;
                } else {
                    next_h = h;
                }
            }
        }

        if h < T_STEP_MIN {
            panic!("Timestep too small!");
        }
    }

    mna.a = a_bkp;
    mna.b = b_bkp;
    elems[in_src_idx].set_value(in_src_bkp);

    (h, next_h, n_iters)
}

fn divided_diff(x_hist: &Vec<Vec<f64>>, t_hist: &Vec<f64>, n_max: usize, n_min: usize) -> Vec<f64> {
    if n_max == n_min {
        x_hist[n_max].clone()
    } else {
        linalg::vec_scalar_prod(
            &linalg::vec_sub(
                &divided_diff(x_hist, t_hist, n_max, n_min + 1),
                &divided_diff(x_hist, t_hist, n_max - 1, n_min),
            ),
            1.0 / (t_hist[n_max] - t_hist[n_min]),
        )
    }
}

fn plte_vec(x_hist: &Vec<Vec<f64>>, t_hist: &Vec<f64>, n: usize) -> Vec<f64> {
    let c3 = -1.0 / 12.0;
    let h_next = t_hist[n + 1] - t_hist[n];

    linalg::vec_scalar_prod(
        &divided_diff(x_hist, t_hist, n + 1, n - 2),
        6.0 * c3 * h_next.powi(3),
    )
}

fn in_src_voltage(t: &f64) -> f64 {
    let tau = 2e-9;
    let v0 = 3.0;
    v0 * (-t / tau).exp()
}

fn plte_is_too_big(plte: &NodeVecNorm, x: &NodeVecNorm) -> bool {
    plte.v > x.v * TOL_REL + TOL_ABS_V || plte.i > x.i * TOL_REL + TOL_ABS_A
}

fn plte_is_small(plte: &NodeVecNorm) -> bool {
    plte.v < 0.1 * TOL_ABS_V && plte.i < 0.1 * TOL_ABS_A
}
