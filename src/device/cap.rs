use crate::device::{GType, Stamp};

pub struct Cap {
    pub name: String,
    pub nodes: Vec<String>,
    pub val: f64,
}

impl Stamp for Cap {
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
}
