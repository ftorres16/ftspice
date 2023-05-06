use std::collections::BTreeMap;
use std::iter::successors;

use crate::command;
use crate::device;
use crate::linear_stamp;
use crate::nonlinear_func;

mod gauss_lu;
mod linalg;
mod newtons_method;

const GND: &str = "0";

pub struct Engine {
    a: Vec<Vec<f64>>,
    b: Vec<f64>,
    h: Vec<Vec<f64>>,
    g: Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
    pub elems: Vec<device::SpiceElem>,
    pub nodes: BTreeMap<String, device::RowType>,
    pub op_cmd: Option<command::Command>,
    pub dc_cmd: Option<command::Command>,
}

impl Engine {
    pub fn new(elems: Vec<device::SpiceElem>, mut cmds: Vec<command::Command>) -> Self {
        let mut nodes = BTreeMap::new();

        nodes.extend(
            elems
                .iter()
                .flat_map(|e| e.nodes.iter())
                .map(|x| (x.to_string(), device::RowType::Voltage)),
        );
        nodes.extend(
            elems
                .iter()
                .filter(|x| matches!(x.dtype, device::DType::Vdd))
                .map(|x| (x.name.to_string(), device::RowType::Current)),
        );

        if !nodes.contains_key(GND) {
            panic!("GND not found!");
        }
        nodes.remove(GND);

        let mut a = vec![vec![0.0; nodes.len()]; nodes.len()];
        let mut b = vec![0.0; nodes.len()];

        let num_nonlinear_funcs = elems
            .iter()
            .map(|x| nonlinear_func::count(x))
            .sum::<usize>();
        let mut h = vec![vec![0.0; num_nonlinear_funcs]; nodes.len()];
        let mut g = Vec::new();

        for elem in elems.iter() {
            linear_stamp::load(&elem, &nodes, &mut a, &mut b);
            nonlinear_func::load(&elem, &nodes, &mut h, &mut g);
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

    pub fn run_op(&self) -> (u64, Vec<f64>) {
        let mut x: Vec<f64> = vec![0.0; self.nodes.len()];
        let n_iters = newtons_method::solve(
            &self.nodes,
            &self.elems,
            &mut x,
            &self.a,
            &self.b,
            &self.h,
            &self.g,
        );
        (n_iters, x)
    }

    pub fn run_dc(&self) -> (Vec<u64>, Vec<Vec<f64>>) {
        let dc_params = match &self.dc_cmd {
            Some(command::Command::DC(x)) => x,
            _ => panic!("DC simulation wrongly configured."),
        };

        let mut sweep_elem = self
            .elems
            .iter()
            .find(|e| e.name == dc_params.source)
            .expect("Sweep source not found")
            .to_owned();

        let mut x_hist = Vec::new();
        let mut n_iters_hist = Vec::new();

        let mut x: Vec<f64> = vec![0.0; self.a.len()];
        let mut b_temp = self.b.clone();

        let sweep_iter = successors(Some(dc_params.start), |x| {
            let next = x + dc_params.step;
            (next < dc_params.stop).then_some(next)
        });

        for sweep_val in sweep_iter {
            // Undo previous stamp, and add new one with the swept value
            // Ignore `a` updates, connectivity doesn't change
            let mut a_temp = self.a.clone();
            sweep_elem.value = Some(-sweep_elem.value.unwrap());
            linear_stamp::load(&sweep_elem, &self.nodes, &mut a_temp, &mut b_temp);
            sweep_elem.value = Some(sweep_val);
            linear_stamp::load(&sweep_elem, &self.nodes, &mut a_temp, &mut b_temp);

            let n_iters = newtons_method::solve(
                &self.nodes,
                &self.elems,
                &mut x,
                &self.a,
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
