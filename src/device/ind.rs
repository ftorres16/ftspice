use crate::device::{GType, Stamp};
use crate::node::NodeCollection;

pub struct Ind {
    pub name: String,
    pub nodes: Vec<String>,
    pub val: f64,
}

impl Stamp for Ind {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_nodes(&self) -> &Vec<String> {
        &self.nodes
    }

    fn gtype(&self) -> GType {
        GType::G1
    }

    fn get_value(&self) -> f64 {
        self.val
    }

    fn set_value(&mut self, value: f64) {
        self.val = value;
    }

    fn init_state(&mut self, _nodes: &NodeCollection, _x: &Vec<f64>) {
        todo!();
    }

    fn update_state(&mut self, _nodes: &NodeCollection, _x: &Vec<f64>, _h: &f64) {
        todo!();
    }

    fn dynamic_stamp(
        &self,
        _nodes: &NodeCollection,
        _x: &Vec<f64>,
        _h: &f64,
        _a: &mut Vec<Vec<f64>>,
        _b: &mut Vec<f64>,
    ) {
        todo!();
    }
}
