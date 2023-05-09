use crate::node;

pub mod diode;
pub mod idd;
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

    fn get_value(&self) -> f64;

    fn set_value(&mut self, value: f64);

    fn linear_stamp(&self, nodes: &node::NodeCollection, a: &mut Vec<Vec<f64>>, b: &mut Vec<f64>);

    fn undo_linear_stamp(
        &self,
        nodes: &node::NodeCollection,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    );

    fn count_nonlinear_funcs(&self) -> usize;

    fn nonlinear_funcs(
        &self,
        nodes: &node::NodeCollection,
        h_mat: &mut Vec<Vec<f64>>,
        g_vec: &mut Vec<Box<dyn Fn(&Vec<f64>) -> f64>>,
    );

    fn nonlinear_stamp(
        &self,
        nodes: &node::NodeCollection,
        x: &Vec<f64>,
        a: &mut Vec<Vec<f64>>,
        b: &mut Vec<f64>,
    );
}
