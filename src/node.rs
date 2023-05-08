use std::collections::HashMap;
use std::collections::HashSet;

use crate::device::stamp;
use crate::device::stamp::Stamp;

const GND: &str = "0";

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

pub fn parse_elems(elems: &Vec<Box<dyn Stamp>>) -> HashMap<String, MNANode> {
    let mut nodes = HashMap::new();
    let v_names = elems
        .iter()
        .flat_map(|e| e.get_nodes().iter())
        .filter(|n| n != &GND)
        .collect::<HashSet<_>>();
    nodes.extend(v_names.iter().enumerate().map(|(i, x)| {
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
    nodes.extend(i_names.iter().enumerate().map(|(i, x)| {
        (
            x.to_string(),
            MNANode {
                ntype: NodeType::Current,
                idx: i + v_names.len(),
            },
        )
    }));

    nodes
}
