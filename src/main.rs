use std::collections::BTreeSet;

mod device;
mod gauss_lu;
use crate::device::Stamp;

const GND: &str = "0";

fn main() {
    let mut elems: Vec<device::SpiceElem> = Vec::new();

    elems.push(device::SpiceElem {
        dtype: device::DType::Vdd,
        name: "V0".to_string(),
        nodes: vec!["0".to_string(), "1".to_string()],
        value: 3.0,
    });
    elems.push(device::SpiceElem {
        dtype: device::DType::Res,
        name: "R1".to_string(),
        nodes: vec!["1".to_string(), "2".to_string()],
        value: 1e3,
    });
    elems.push(device::SpiceElem {
        dtype: device::DType::Res,
        name: "R1".to_string(),
        nodes: vec!["2".to_string(), "0".to_string()],
        value: 1e3,
    });

    let nodes = find_nodes(&elems);

    let mut a_mat = vec![vec![0.0; nodes.len()]; nodes.len()];
    let mut b_vec = vec![0.0; nodes.len()];
    let mut x_vec = vec![0.0; nodes.len()];

    for elem in elems.iter() {
        elem.linear_stamp(&nodes, &mut a_mat, &mut b_vec);
    }

    gauss_lu::solve(&mut a_mat, &mut b_vec, &mut x_vec);

    for (node, val) in nodes.iter().zip(x_vec.iter()) {
        println!("{node}: {val}");
    }
}

fn find_nodes(elems: &Vec<device::SpiceElem>) -> BTreeSet<String> {
    let mut nodes: BTreeSet<String> = BTreeSet::new();

    for elem in elems.iter() {
        for node in elem.nodes.iter() {
            nodes.insert(node.to_string());
        }

        if let device::DType::Vdd = elem.dtype {
            nodes.insert(elem.name.to_string());
        }
    }

    if !nodes.contains(GND) {
        panic!("GND not found!");
    }
    nodes.remove(GND);

    nodes
}
