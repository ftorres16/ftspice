use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::device::{GType, Stamp};
use crate::node::{MNANode, NodeType, GND};

#[derive(Debug)]
pub struct NodeCollection {
    data: BTreeMap<String, MNANode>,
}

impl NodeCollection {
    pub fn from_elems(elems: &[Box<dyn Stamp>]) -> Self {
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

    pub fn from_startup_elems(elems: &[Box<dyn Stamp>]) -> Self {
        let mut nc = NodeCollection::from_elems(elems);
        let nc_len = nc.data.len();

        let i_names = elems
            .iter()
            .filter(|e| matches!((e.gtype(), e.gtype_startup()), (GType::G1, GType::G2)))
            .map(|e| e.get_name())
            .collect::<BTreeSet<_>>();
        nc.data.extend(i_names.iter().enumerate().map(|(i, n)| {
            (
                n.to_string(),
                MNANode {
                    ntype: NodeType::Current,
                    idx: i + nc_len,
                },
            )
        }));

        nc
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get_idx(&self, name: &str) -> Option<usize> {
        self.data.get(name).map(|x| x.idx)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &MNANode)> {
        self.data.iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &MNANode> {
        self.data.values()
    }
}
