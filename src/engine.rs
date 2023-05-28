use std::iter::successors;

use crate::command;
use crate::device::Stamp;
use crate::engine::mna::MNA;
use crate::engine::transient::T_STEP_MIN;
use crate::node_collection::NodeCollection;

mod gauss_lu;
mod linalg;
mod mna;
mod newtons_method;
mod node_vec_norm;
mod transient;

pub struct Engine {
    pub elems: Vec<Box<dyn Stamp>>,
    pub op_cmd: Option<command::Command>,
    pub dc_cmd: Option<command::Command>,
    pub tran_cmd: Option<command::Command>,
    num_nonlinear_funcs: usize,
}

impl Engine {
    pub fn new(mut elems: Vec<Box<dyn Stamp>>, mut cmds: Vec<command::Command>) -> Self {
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

        let num_nonlinear_funcs = elems.iter().map(|e| e.count_nonlinear_funcs()).sum();

        for elem in elems.iter_mut() {
            if elem.has_tran() {
                elem.eval_tran(&0.0);
            }
        }

        Engine {
            elems: elems,
            op_cmd: op_cmd,
            dc_cmd: dc_cmd,
            tran_cmd: tran_cmd,
            num_nonlinear_funcs,
        }
    }

    pub fn run_op(&mut self) -> (u64, NodeCollection, Vec<f64>) {
        let nodes = NodeCollection::from_startup_elems(&self.elems);

        let mut mna = MNA::new(nodes.len(), self.num_nonlinear_funcs);

        for elem in self.elems.iter() {
            elem.linear_startup_stamp(&nodes, &mut mna.a, &mut mna.b);
            elem.nonlinear_funcs(&nodes, &mut mna.h, &mut mna.g);
        }

        let mut x = mna.get_x();
        let n_iters = newtons_method::solve(&nodes, &self.elems, &mut x, &mna);

        for elem in self.elems.iter_mut() {
            elem.init_state(&nodes, &x);
        }

        (n_iters, nodes, x)
    }

    pub fn run_dc(&mut self) -> (Vec<u64>, NodeCollection, Vec<Vec<f64>>) {
        let dc_params = match &self.dc_cmd {
            Some(command::Command::DC(x)) => x,
            _ => panic!("DC simulation wrongly configured."),
        };

        let sweep_idx = self
            .elems
            .iter()
            .position(|e| e.get_name() == dc_params.source)
            .expect("Sweep source not found");

        let nodes = NodeCollection::from_elems(&self.elems);
        let mut mna = MNA::new(nodes.len(), self.num_nonlinear_funcs);

        for elem in self.elems.iter() {
            elem.linear_stamp(&nodes, &mut mna.a, &mut mna.b);
            elem.nonlinear_funcs(&nodes, &mut mna.h, &mut mna.g);
        }

        let mut x_hist = Vec::new();
        let mut n_iters_hist = Vec::new();

        let mut x = mna.get_x();

        let val_bkp = self.elems[sweep_idx].get_value();

        let sweep_iter = successors(Some(dc_params.start), |x| {
            let next = x + dc_params.step;
            (next < dc_params.stop).then_some(next)
        });

        for sweep_val in sweep_iter {
            self.elems[sweep_idx].undo_linear_stamp(&nodes, &mut mna.a, &mut mna.b);
            self.elems[sweep_idx].set_value(sweep_val);
            self.elems[sweep_idx].linear_stamp(&nodes, &mut mna.a, &mut mna.b);

            let n_iters = newtons_method::solve(&nodes, &self.elems, &mut x, &mna);

            n_iters_hist.push(n_iters);
            x_hist.push(x.clone());
        }

        self.elems[sweep_idx].set_value(val_bkp);

        (n_iters_hist, nodes, x_hist)
    }

    pub fn run_tran(&mut self) -> (Vec<u64>, NodeCollection, Vec<f64>, Vec<Vec<f64>>) {
        let tran_params = match &self.tran_cmd {
            Some(command::Command::Tran(x)) => x.to_owned(),
            _ => panic!("DC simulation wrongly configured."),
        };

        let nodes = NodeCollection::from_elems(&self.elems);
        let mut mna = MNA::new(nodes.len(), self.num_nonlinear_funcs);
        let mut x = mna.get_x();

        for elem in self.elems.iter() {
            elem.linear_stamp(&nodes, &mut mna.a, &mut mna.b);
            elem.nonlinear_funcs(&nodes, &mut mna.h, &mut mna.g);
        }

        // Load Start Up solutions
        let (_, startup_nodes, startup_x) = self.run_op();
        for (name, node) in nodes.iter() {
            let old_idx = startup_nodes
                .get_idx(name)
                .expect("New node not found in startup nodes.");
            x[node.idx] = startup_x[old_idx];
        }

        let mut state_hist = Vec::new();

        let mut t = tran_params.start;
        let mut h = T_STEP_MIN;
        let mut next_h;

        while t < tran_params.stop {
            (h, next_h) = transient::step(
                &nodes,
                &mut self.elems,
                &mut mna,
                &t,
                &h,
                &mut x,
                &mut state_hist,
                &tran_params.step,
            );

            t += h;

            for elem in self.elems.iter_mut() {
                elem.update_state(&nodes, &x, &h);
            }

            h = next_h;
        }

        (
            state_hist.iter().map(|s| s.n_iters).collect(),
            nodes,
            state_hist.iter().map(|s| s.t).collect(),
            state_hist.iter().map(|s| s.x.clone()).collect(),
        )
    }
}
