use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::device::{GType, Stamp};

const GND: &str = "0";

pub struct NodeCollection {
    data: BTreeMap<String, MNANode>,
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
    let mut map = BTreeMap::new();

    let v_names = elems
        .iter()
        .flat_map(|e| e.get_nodes().iter())
        .filter(|n| n != &GND)
        .collect::<BTreeSet<_>>();
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
        .filter(|x| matches!(x.gtype(), GType::G2))
        .map(|x| x.get_name())
        .collect::<BTreeSet<_>>();
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
}
