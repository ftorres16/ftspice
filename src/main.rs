use std::collections::BTreeMap;
use std::env;

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod device;
mod gauss_lu;
mod linalg;
mod newtons_method;
mod parser;

const GND: &str = "0";

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = &args[1];

    let elems = parser::parse_spice_file(file);

    let nodes = find_nodes(&elems);
    let num_nonlinear_funcs = elems.iter().map(|x| x.num_nonlinear_funcs()).sum();

    let mut x_vec = vec![0.0; nodes.len()];

    // Linear elements
    let mut a_mat = vec![vec![0.0; nodes.len()]; nodes.len()];
    let mut b_vec = vec![0.0; nodes.len()];

    // Nonlinear elements
    let mut h_mat = vec![vec![0.0; nodes.len()]; num_nonlinear_funcs];
    let mut g_vec: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

    for elem in elems.iter() {
        elem.linear_stamp(&nodes, &mut a_mat, &mut b_vec);
        elem.nonlinear_func(&nodes, &mut h_mat, &mut g_vec);
    }

    // gauss_lu::solve(&mut a_mat, &mut b_vec, &mut x_vec);
    newtons_method::solve(&nodes, &elems, &mut x_vec, &a_mat, &b_vec, &h_mat, &g_vec);

    for ((node, type_), val) in nodes.iter().zip(x_vec.iter()) {
        println!("{node} ({type_:?}): {val}");
    }
}

fn find_nodes(elems: &Vec<device::SpiceElem>) -> BTreeMap<String, device::NodeType> {
    let mut nodes: BTreeMap<String, device::NodeType> = BTreeMap::new();

    for elem in elems.iter() {
        for node in elem.nodes.iter() {
            nodes.insert(node.to_string(), device::NodeType::G1);
        }

        if let device::DType::Vdd = elem.dtype {
            nodes.insert(elem.name.to_string(), device::NodeType::G2);
        }
    }

    if !nodes.contains_key(GND) {
        panic!("GND not found!");
    }
    nodes.remove(GND);

    nodes
}
