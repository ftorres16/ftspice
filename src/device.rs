use crate::node;

pub mod cap;
pub mod diode;
pub mod idd;
pub mod ind;
pub mod nmos;
pub mod npn;
pub mod res;
pub mod spice_fn;
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

    fn get_value(&self) -> f64;

    fn set_value(&mut self, value: f64);

    fn has_tran(&self) -> bool {
        false
    }

    fn eval_tran(&mut self, _t: &f64) {}

    fn linear_stamp(
        &self,
        _nodes: &node::NodeCollection,
        _a: &mut Vec<Vec<f64>>,
        _b: &mut Vec<f64>,
    ) {
    }

    fn undo_linear_stamp(
        &self,
        _nodes: &node::NodeCollection,
        _a: &mut Vec<Vec<f64>>,
        _b: &mut Vec<f64>,
    ) {
    }

    fn init_state(&mut self, _nodes: &node::NodeCollection, _x: &Vec<f64>) {}

    fn update_state(&mut self, _nodes: &node::NodeCollection, _x: &Vec<f64>, _h: &f64) {}

    fn dynamic_stamp(
        &self,
        _nodes: &node::NodeCollection,
        _x: &Vec<f64>,
        _h: &f64,
        _a: &mut Vec<Vec<f64>>,
        _b: &mut Vec<f64>,
    ) {
    }

    fn undo_dynamic_stamp(
        &self,
        _nodes: &node::NodeCollection,
        _x: &Vec<f64>,
        _h: &f64,
        _a: &mut Vec<Vec<f64>>,
        _b: &mut Vec<f64>,
    ) {
    }

    fn count_nonlinear_funcs(&self) -> usize {
        0
    }

    fn nonlinear_funcs(
        &self,
        _nodes: &node::NodeCollection,
        _h_mat: &mut Vec<Vec<f64>>,
        _g_vec: &mut Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
    ) {
    }

    fn nonlinear_stamp(
        &self,
        _nodes: &node::NodeCollection,
        _x: &Vec<f64>,
        _a: &mut Vec<Vec<f64>>,
        _b: &mut Vec<f64>,
    ) {
    }
}
