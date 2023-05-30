use ndarray::prelude::*;

use crate::node::NodeType;
use crate::node_collection::NodeCollection;

#[derive(Debug, Clone)]
pub struct NodeVecNorm {
    pub v: f64,
    pub i: f64,
}

impl NodeVecNorm {
    pub fn new(nodes: &NodeCollection, v: &Array1<f64>) -> Self {
        let v_nodes = nodes
            .values()
            .filter(|x| matches!(x.ntype, NodeType::Voltage))
            .collect::<Vec<_>>();
        let i_nodes = nodes
            .values()
            .filter(|x| matches!(x.ntype, NodeType::Current))
            .collect::<Vec<_>>();

        let v_norm =
            v_nodes.iter().map(|x| v[x.idx].powi(2)).sum::<f64>().sqrt() / v_nodes.len() as f64;
        let i_norm =
            i_nodes.iter().map(|x| v[x.idx].powi(2)).sum::<f64>().sqrt() / i_nodes.len() as f64;

        NodeVecNorm {
            v: v_norm,
            i: i_norm,
        }
    }

    pub fn infty() -> Self {
        NodeVecNorm {
            v: f64::INFINITY,
            i: f64::INFINITY,
        }
    }
}
