use crate::device;

use std::fs;

use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "spice.pest"]
pub struct SpiceParser;

pub fn parse_spice_file(file: &str) -> Vec<device::SpiceElem> {
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
                    Rule::r_node => elems.push(parse_res(node)),
                    Rule::v_node => elems.push(parse_vdd(node)),
                    _ => unreachable!(),
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    elems
}

fn parse_res(node: Pair<Rule>) -> device::SpiceElem {
    let mut node_details = node.into_inner();

    let name = node_details.next().unwrap().as_str();
    let node_0 = node_details.next().unwrap().as_str();
    let node_1 = node_details.next().unwrap().as_str();

    let value = parse_value(node_details.next().unwrap());

    device::SpiceElem {
        dtype: device::DType::Res,
        name: String::from(name),
        nodes: vec![String::from(node_0), String::from(node_1)],
        value: value,
    }
}

fn parse_vdd(node: Pair<Rule>) -> device::SpiceElem {
    let mut node_details = node.into_inner();
    let name = node_details.next().unwrap().as_str();
    let node_1 = node_details.next().unwrap().as_str();
    let node_0 = node_details.next().unwrap().as_str();
    let value = parse_value(node_details.next().unwrap().into_inner().next().unwrap());

    device::SpiceElem {
        dtype: device::DType::Vdd,
        name: String::from(name),
        nodes: vec![String::from(node_0), String::from(node_1)],
        value: value,
    }
}

fn parse_value(value: Pair<Rule>) -> f64 {
    let mut value_details = value.into_inner();

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
