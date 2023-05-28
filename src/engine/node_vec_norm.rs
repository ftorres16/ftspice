use crate::node::NodeType;
use crate::node_collection::NodeCollection;

#[derive(Debug, Clone)]
pub struct NodeVecNorm {
    pub v: f64,
    pub i: f64,
}

impl NodeVecNorm {
    pub fn new(nodes: &NodeCollection, v: &Vec<f64>) -> Self {
        let mut norm = NodeVecNorm { v: 0.0, i: 0.0 };

        for node in nodes.values() {
            let norm_it = v[node.idx].abs();

            match node.ntype {
                NodeType::Voltage => {
                    if norm_it > norm.v {
                        norm.v = norm_it;
                    }
                }
                NodeType::Current => {
                    if norm_it > norm.i {
                        norm.i = norm_it;
                    }
                }
            }
        }
        norm
    }

    pub fn infty() -> Self {
        NodeVecNorm {
            v: f64::INFINITY,
            i: f64::INFINITY,
        }
    }
}
