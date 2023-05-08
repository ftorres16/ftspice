use std::collections::HashMap;
use std::collections::HashSet;

use crate::device::stamp;
use crate::device::stamp::Stamp;

const GND: &str = "0";

pub struct NodeCollection {
    data: HashMap<String, MNANode>,
}

#[derive(Debug)]
pub struct MNANode {
    pub ntype: NodeType,
    pub idx: usize,
}

#[derive(Debug)]
pub enum NodeType {
    Voltage,
    Current,
}

pub fn parse_elems(elems: &Vec<Box<dyn Stamp>>) -> NodeCollection {
    let mut map = HashMap::new();

    let v_names = elems
        .iter()
        .flat_map(|e| e.get_nodes().iter())
        .filter(|n| n != &GND)
        .collect::<HashSet<_>>();
    map.extend(v_names.iter().enumerate().map(|(i, x)| {
        (
            x.to_string(),
            MNANode {
                ntype: NodeType::Voltage,
                idx: i,
            },
        )
    }));

    let i_names = elems
        .iter()
        .filter(|x| matches!(x.gtype(), stamp::GType::G2))
        .map(|x| x.get_name())
        .collect::<HashSet<_>>();
    map.extend(i_names.iter().enumerate().map(|(i, x)| {
        (
            x.to_string(),
            MNANode {
                ntype: NodeType::Current,
                idx: i + v_names.len(),
            },
        )
    }));

    NodeCollection { data: map }
}

impl NodeCollection {
    pub fn new() -> Self {
        NodeCollection {
            data: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get_idx(&self, name: &str) -> Option<usize> {
        self.data.get(name).map(|x| x.idx)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &MNANode> {
        self.data.values()
    }

    pub fn insert(&mut self, name: &str, node: MNANode) {
        self.data.insert(name.to_string(), node);
    }
}
