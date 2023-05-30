use ndarray::prelude::*;

use crate::node_collection::NodeCollection;

pub mod cap;
pub mod diode;
pub mod idd;
pub mod ind;
pub mod nmos;
pub mod npn;
pub mod res;
pub mod vdd;

// MNA sgroups from Circuit Simulation Book
pub enum GType {
    G1,
    G2,
}

pub trait Stamp {
    fn get_name(&self) -> &str;

    fn get_nodes(&self) -> &Vec<String>;

    fn gtype(&self) -> GType;

    // For elements whose GType changes like Inductors
    fn gtype_startup(&self) -> GType {
        self.gtype()
    }

    fn get_value(&self) -> f64;

    fn set_value(&mut self, value: f64);

    fn has_tran(&self) -> bool {
        false
    }

    fn eval_tran(&mut self, _t: &f64) {}

    fn linear_stamp(&self, _nodes: &NodeCollection, _a: &mut Array2<f64>, _b: &mut Array1<f64>) {}

    fn undo_linear_stamp(
        &self,
        _nodes: &NodeCollection,
        _a: &mut Array2<f64>,
        _b: &mut Array1<f64>,
    ) {
    }

    fn linear_startup_stamp(
        &self,
        nodes: &NodeCollection,
        a: &mut Array2<f64>,
        b: &mut Array1<f64>,
    ) {
        self.linear_stamp(nodes, a, b);
    }

    fn undo_linear_startup_stamp(
        &self,
        nodes: &NodeCollection,
        a: &mut Array2<f64>,
        b: &mut Array1<f64>,
    ) {
        self.undo_linear_stamp(nodes, a, b);
    }

    fn init_state(&mut self, _nodes: &NodeCollection, _x: &Array1<f64>) {}

    fn update_state(&mut self, _nodes: &NodeCollection, _x: &Array1<f64>, _h: &f64) {}

    fn dynamic_stamp(
        &self,
        _nodes: &NodeCollection,
        _x: &Array1<f64>,
        _h: &f64,
        _a: &mut Array2<f64>,
        _b: &mut Array1<f64>,
    ) {
    }

    fn undo_dynamic_stamp(
        &self,
        _nodes: &NodeCollection,
        _x: &Array1<f64>,
        _h: &f64,
        _a: &mut Array2<f64>,
        _b: &mut Array1<f64>,
    ) {
    }

    fn count_nonlinear_funcs(&self) -> usize {
        0
    }

    fn nonlinear_funcs(
        &self,
        _nodes: &NodeCollection,
        _h_mat: &mut Array2<f64>,
        _g_vec: &mut Vec<Box<dyn Fn(&Array1<f64>) -> f64>>,
    ) {
    }

    fn nonlinear_stamp(
        &self,
        _nodes: &NodeCollection,
        _x: &Array1<f64>,
        _a: &mut Array2<f64>,
        _b: &mut Array1<f64>,
    ) {
    }
}
