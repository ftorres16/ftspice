use std::iter::successors;

use crate::command;
use crate::device::Stamp;
use crate::node;

mod gauss_lu;
mod linalg;
mod newtons_method;

pub struct Engine {
    a: Vec<Vec<f64>>,
    b: Vec<f64>,
    h: Vec<Vec<f64>>,
    g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
    pub elems: Vec<Box<dyn Stamp>>,
    pub nodes: node::NodeCollection,
    pub op_cmd: Option<command::Command>,
    pub dc_cmd: Option<command::Command>,
}

impl Engine {
    pub fn new(elems: Vec<Box<dyn Stamp>>, mut cmds: Vec<command::Command>) -> Self {
        let nodes = node::parse_elems(&elems);

        let mut a = vec![vec![0.0; nodes.len()]; nodes.len()];
        let mut b = vec![0.0; nodes.len()];

        let num_nonlinear_funcs = elems
            .iter()
            .map(|e| e.count_nonlinear_funcs())
            .sum::<usize>();
        let mut h = vec![vec![0.0; num_nonlinear_funcs]; nodes.len()];
        let mut g = Vec::new();

        for elem in elems.iter() {
            elem.linear_stamp(&nodes, &mut a, &mut b);
            elem.nonlinear_funcs(&nodes, &mut h, &mut g);
        }

        let op_cmd = match cmds.iter().position(|x| matches!(x, command::Command::Op)) {
            Some(i) => Some(cmds.remove(i)),
            None => None,
        };
        let dc_cmd = match cmds
            .iter()
            .position(|x| matches!(x, command::Command::DC(_)))
        {
            Some(i) => Some(cmds.remove(i)),
            None => None,
        };

        Engine {
            a: a,
            b: b,
            h: h,
            g: g,
            elems: elems,
            nodes: nodes,
            op_cmd: op_cmd,
            dc_cmd: dc_cmd,
        }
    }

    pub fn run_op(&mut self) -> (u64, Vec<f64>) {
        let mut x: Vec<f64> = vec![0.0; self.nodes.len()];
        let n_iters = newtons_method::solve(
            &self.nodes,
            &mut self.elems,
            &mut x,
            &self.a,
            &self.b,
            &self.h,
            &self.g,
        );
        (n_iters, x)
    }

    pub fn run_dc(&mut self) -> (Vec<u64>, Vec<Vec<f64>>) {
        let dc_params = match &self.dc_cmd {
            Some(command::Command::DC(x)) => x,
            _ => panic!("DC simulation wrongly configured."),
        };

        let sweep_idx = self
            .elems
            .iter()
            .position(|e| e.get_name() == dc_params.source)
            .expect("Sweep source not found");

        let mut x_hist = Vec::new();
        let mut n_iters_hist = Vec::new();

        let mut x: Vec<f64> = vec![0.0; self.a.len()];
        let mut a_temp = self.a.clone();
        let mut b_temp = self.b.clone();

        let sweep_iter = successors(Some(dc_params.start), |x| {
            let next = x + dc_params.step;
            (next < dc_params.stop).then_some(next)
        });

        for sweep_val in sweep_iter {
            self.elems[sweep_idx].undo_linear_stamp(&self.nodes, &mut a_temp, &mut b_temp);
            self.elems[sweep_idx].set_value(sweep_val);
            self.elems[sweep_idx].linear_stamp(&self.nodes, &mut a_temp, &mut b_temp);

            let n_iters = newtons_method::solve(
                &self.nodes,
                &self.elems,
                &mut x,
                &a_temp,
                &b_temp,
                &self.h,
                &self.g,
            );

            n_iters_hist.push(n_iters);
            x_hist.push(x.clone());
        }

        (n_iters_hist, x_hist)
    }
}
