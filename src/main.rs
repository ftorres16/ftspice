use std::collections::BTreeSet;
use std::env;

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod device;
mod gauss_lu;
mod parser;

const GND: &str = "0";

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = &args[1];

    let elems = parser::parse_spice_file(file);

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
