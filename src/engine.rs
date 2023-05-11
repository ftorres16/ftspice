use std::iter::successors;

use crate::command;
use crate::device::Stamp;
use crate::engine::mna::MNA;
use crate::engine::transient::T_STEP_MIN;
use crate::node;

mod gauss_lu;
mod linalg;
mod mna;
mod newtons_method;
mod node_vec_norm;
mod transient;

pub struct Engine {
    pub mna: MNA,
    pub elems: Vec<Box<dyn Stamp>>,
    pub nodes: node::NodeCollection,
    pub op_cmd: Option<command::Command>,
    pub dc_cmd: Option<command::Command>,
    pub tran_cmd: Option<command::Command>,
}

impl Engine {
    pub fn new(elems: Vec<Box<dyn Stamp>>, mut cmds: Vec<command::Command>) -> Self {
        let nodes = node::parse_elems(&elems);
        let num_nonlinear_funcs = elems
            .iter()
            .map(|e| e.count_nonlinear_funcs())
            .sum::<usize>();
        let mut mna = MNA::new(nodes.len(), num_nonlinear_funcs);

        for elem in elems.iter() {
            elem.linear_stamp(&nodes, &mut mna.a, &mut mna.b);
            elem.nonlinear_funcs(&nodes, &mut mna.h, &mut mna.g);
        }

        let op_cmd = cmds
            .iter()
            .position(|x| matches!(x, command::Command::Op))
            .map(|i| cmds.remove(i));
        let dc_cmd = cmds
            .iter()
            .position(|x| matches!(x, command::Command::DC(_)))
            .map(|i| cmds.remove(i));
        let tran_cmd = cmds
            .iter()
            .position(|x| matches!(x, command::Command::Tran(_)))
            .map(|i| cmds.remove(i));

        Engine {
            mna: mna,
            elems: elems,
            nodes: nodes,
            op_cmd: op_cmd,
            dc_cmd: dc_cmd,
            tran_cmd: tran_cmd,
        }
    }

    pub fn run_op(&mut self) -> (u64, Vec<f64>) {
        let mut x = self.mna.get_x();
        let n_iters = newtons_method::solve(&self.nodes, &self.elems, &mut x, &self.mna);
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

        let mut x = self.mna.get_x();

        let a_bkp = self.mna.a.clone();
        let b_bkp = self.mna.b.clone();
        let val_bkp = self.elems[sweep_idx].get_value();

        let sweep_iter = successors(Some(dc_params.start), |x| {
            let next = x + dc_params.step;
            (next < dc_params.stop).then_some(next)
        });

        for sweep_val in sweep_iter {
            self.elems[sweep_idx].undo_linear_stamp(&self.nodes, &mut self.mna.a, &mut self.mna.b);
            self.elems[sweep_idx].set_value(sweep_val);
            self.elems[sweep_idx].linear_stamp(&self.nodes, &mut self.mna.a, &mut self.mna.b);

            let n_iters = newtons_method::solve(&self.nodes, &self.elems, &mut x, &self.mna);

            n_iters_hist.push(n_iters);
            x_hist.push(x.clone());
        }

        self.mna.a = a_bkp;
        self.mna.b = b_bkp;
        self.elems[sweep_idx].set_value(val_bkp);

        (n_iters_hist, x_hist)
    }

    pub fn run_tran(&mut self) -> (Vec<u64>, Vec<f64>, Vec<Vec<f64>>) {
        let tran_params = match &self.tran_cmd {
            Some(command::Command::Tran(x)) => x.to_owned(),
            _ => panic!("DC simulation wrongly configured."),
        };

        let (_, mut x) = self.run_op();

        for elem in self.elems.iter_mut() {
            elem.init_state(&self.nodes, &x);
        }

        let mut state_hist = Vec::new();

        let in_src_idx = self
            .elems
            .iter()
            .position(|x| x.get_name().starts_with("V"))
            .expect("No V source found");

        let mut t = tran_params.start;
        let mut h = T_STEP_MIN;
        let mut next_h;

        while t < tran_params.stop {
            (h, next_h) = transient::step(
                &self.nodes,
                &mut self.elems,
                &mut self.mna,
                &t,
                &h,
                &mut x,
                &in_src_idx,
                &mut state_hist,
                &tran_params.step,
            );

            t += h;

            for elem in self.elems.iter_mut() {
                elem.update_state(&self.nodes, &x, &h);
            }

            h = next_h;
        }

        (
            state_hist.iter().map(|s| s.n_iters).collect(),
            state_hist.iter().map(|s| s.t).collect(),
            state_hist.iter().map(|s| s.x.clone()).collect(),
        )
    }
}
