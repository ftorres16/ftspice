use std::collections::BTreeMap;
use std::env;

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod command;
mod device;
mod gauss_lu;
mod linalg;
mod linear_stamp;
mod newtons_method;
mod nonlinear_func;
mod nonlinear_stamp;
mod parser;

const GND: &str = "0";

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = &args
        .get(1)
        .expect("Insufficient arguments. Specify spice file to simulate.");

    let (elems, _) = parser::parse_spice_file(file);

    let nodes = find_nodes(&elems);
    let num_nonlinear_funcs = elems.iter().map(|x| nonlinear_func::count(x)).sum();

    let mut x_vec = vec![0.0; nodes.len()];

    // Linear elements
    let mut a_mat = vec![vec![0.0; nodes.len()]; nodes.len()];
    let mut b_vec = vec![0.0; nodes.len()];

    // Nonlinear elements
    let mut h_mat = vec![vec![0.0; num_nonlinear_funcs]; nodes.len()];
    let mut g_vec: Vec<Box<dyn Fn(&Vec<f64>) -> f64>> = Vec::new();

    for elem in elems.iter() {
        linear_stamp::load(&elem, &nodes, &mut a_mat, &mut b_vec);
        nonlinear_func::load(&elem, &nodes, &mut h_mat, &mut g_vec);
    }

    // gauss_lu::solve(&mut a_mat, &mut b_vec, &mut x_vec);
    let n_iters = newtons_method::solve(&nodes, &elems, &mut x_vec, &a_mat, &b_vec, &h_mat, &g_vec);

    // Display as CSV
    println!(
        "n_iters{}",
        nodes.keys().fold(String::new(), |a, b| a + "," + b)
    );
    println!(
        "{}{}",
        n_iters,
        x_vec
            .iter()
            .map(|x| x.to_string())
            .fold(String::new(), |a, b| a + "," + &b)
    );
}

fn find_nodes(elems: &Vec<device::SpiceElem>) -> BTreeMap<String, device::RowType> {
    let mut nodes: BTreeMap<String, device::RowType> = BTreeMap::new();

    for elem in elems.iter() {
        for node in elem.nodes.iter() {
            nodes.insert(node.to_string(), device::RowType::Voltage);
        }

        if let device::DType::Vdd = elem.dtype {
            nodes.insert(elem.name.to_string(), device::RowType::Current);
        }
    }

    if !nodes.contains_key(GND) {
        panic!("GND not found!");
    }
    nodes.remove(GND);

    nodes
}
