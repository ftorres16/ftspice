use std::collections::BTreeSet;
use std::fs;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::iterators::Pairs;
use pest::Parser;

#[derive(Parser)]
#[grammar = "spice.pest"]
pub struct SpiceParser;

mod device;
mod gauss_lu;

const GND: &str = "0";

fn main() {
    let elems = parse_spice_file("test/test.sp");

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

fn parse_spice_file(file: &str) -> Vec<device::SpiceElem> {
    let mut elems = Vec::new();

    let unparsed_file = fs::read_to_string(file).expect("Cannot read file.");

    let file = SpiceParser::parse(Rule::file, &unparsed_file)
        .expect("Unsuccessful parse")
        .next()
        .unwrap(); // unwrap `file` rule, never fails

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::node => {
                let node = line.into_inner().next().unwrap();

                match node.as_rule() {
                    Rule::r_node => {
                        let mut node_details = node.into_inner();
                        let name = node_details.next().unwrap().as_str();
                        let node_0 = node_details.next().unwrap().as_str();
                        let node_1 = node_details.next().unwrap().as_str();

                        let value = parse_value(node_details.next().unwrap().into_inner());

                        elems.push(device::SpiceElem {
                            dtype: device::DType::Res,
                            name: String::from(name),
                            nodes: vec![String::from(node_0), String::from(node_1)],
                            value: value,
                        });
                    }
                    Rule::v_node => {
                        let mut node_details = node.into_inner();
                        let name = node_details.next().unwrap().as_str();
                        let node_1 = node_details.next().unwrap().as_str();
                        let node_0 = node_details.next().unwrap().as_str();
                        let value = parse_value(
                            node_details
                                .next()
                                .unwrap()
                                .into_inner()
                                .next()
                                .unwrap()
                                .into_inner(),
                        );

                        elems.push(device::SpiceElem {
                            dtype: device::DType::Vdd,
                            name: String::from(name),
                            nodes: vec![String::from(node_0), String::from(node_1)],
                            value: value,
                        });
                    }
                    _ => unreachable!(),
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    elems
}

fn parse_value(mut value_details: Pairs<Rule>) -> f64 {
    let mut value = value_details
        .next()
        .unwrap()
        .as_str()
        .parse::<f64>()
        .unwrap();

    if let Some(prefix) = value_details.next() {
        let mult = match prefix.as_str() {
            "G" => 1e9,
            "M" => 1e6,
            "k" => 1e3,
            "h" => 1e2,
            "da" => 1e1,
            "d" => 1e-1,
            "c" => 1e-2,
            "m" => 1e-3,
            "u" => 1e-6,
            "n" => 1e-9,
            "p" => 1e-12,
            "f" => 1e-18,
            _ => unreachable!(),
        };
        value *= mult;
    }

    value
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
