use ndarray::prelude::*;

use crate::node::NodeType;
use crate::node_collection::NodeCollection;

#[derive(Debug, Clone)]
pub struct NodeVecNorm {
    pub v: f64,
    pub i: f64,
}

fn norm(v: &Array1<f64>) -> f64 {
    v.mapv(|x| x.powi(2)).sum().sqrt() / v.len() as f64
}

impl NodeVecNorm {
    pub fn new(nodes: &NodeCollection, v: &Array1<f64>) -> Self {
        let v_nodes = nodes
            .values()
            .filter(|x| matches!(x.ntype, NodeType::Voltage))
            .map(|x| v[x.idx])
            .collect::<Array1<_>>();
        let i_nodes = nodes
            .values()
            .filter(|x| matches!(x.ntype, NodeType::Current))
            .map(|x| v[x.idx])
            .collect::<Array1<_>>();

        NodeVecNorm {
            v: norm(&v_nodes),
            i: norm(&i_nodes),
        }
    }

    pub fn infty() -> Self {
        NodeVecNorm {
            v: f64::INFINITY,
            i: f64::INFINITY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;

    #[test]
    fn test_norm_zero() {
        let v = array![0.0, 0.0, 0.0];

        assert_eq!(norm(&v), 0.0);
    }

    #[test]
    fn test_norm_pos() {
        let v = array![1.0, 1.0, 1.0];

        assert!(norm(&v) > 0.0);
    }

    #[test]
    fn test_norm_scale() {
        let v = array![1.0, 1.0, 1.0];
        let a = 3.0;

        assert!((a * norm(&v) - norm(&(&v * a))).abs() < EPS);
    }

    #[test]
    fn test_norm_triangular() {
        let v1 = array![1.0, 1.0, 1.0];
        let v2 = array![2.0, 2.0, 2.0];

        assert!(norm(&(&v1 + &v2)) >= norm(&v1) + norm(&v2));
    }
}
