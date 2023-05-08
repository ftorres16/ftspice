use std::collections::HashMap;
use std::collections::HashSet;

use crate::device;

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

pub fn parse_elems(elems: &Vec<device::SpiceElem>) -> HashMap<String, MNANode> {
    let mut nodes = HashMap::new();
    let v_names = elems
        .iter()
        .flat_map(|e| e.nodes.iter())
        .filter(|n| n != &GND)
        .collect::<HashSet<_>>();
    let i_names = elems
        .iter()
        .filter(|x| matches!(x.dtype, device::DType::Vdd))
        .map(|x| &x.name)
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
