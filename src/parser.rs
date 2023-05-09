use std::fs;

use crate::command;
use crate::device;
use crate::device::Stamp;

use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "spice.pest"]
pub struct SpiceParser;

pub fn parse_spice_file(file: &str) -> (Vec<Box<dyn Stamp>>, Vec<command::Command>) {
    let mut elems = Vec::new();
    let mut cmds = Vec::new();

    let unparsed_file = fs::read_to_string(file).expect("Cannot read file.");

    let file = SpiceParser::parse(Rule::file, &unparsed_file)
        .expect("Unsuccessful parse")
        .next()
        .unwrap(); // unwrap `file` rule, never fails

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::node => {
                let node = line.into_inner().next().unwrap();

                let e: Box<dyn Stamp> = match node.as_rule() {
                    Rule::r_node => Box::new(parse_res(node)),
                    Rule::v_node => Box::new(parse_vdd(node)),
                    Rule::i_node => Box::new(parse_idd(node)),
                    Rule::ind_node => Box::new(parse_ind(node)),
                    Rule::cap_node => Box::new(parse_cap(node)),
                    Rule::dio_node => Box::new(parse_dio(node)),
                    Rule::bjt_node => Box::new(parse_bjt(node)),
                    Rule::mos_node => Box::new(parse_mos(node)),
                    _ => unreachable!(),
                };
                elems.push(e);
            }
            Rule::command => {
                let cmd = line.into_inner().next().unwrap();

                match cmd.as_rule() {
                    Rule::op_cmd => cmds.push(parse_op_cmd()),
                    Rule::dc_cmd => cmds.push(parse_dc_cmd(cmd)),
                    Rule::tran_cmd => cmds.push(parse_tran_cmd(cmd)),
                    _ => unreachable!(),
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    (elems, cmds)
}

fn parse_res(node: Pair<Rule>) -> device::res::Res {
    let mut node_details = node.into_inner();

    let name = node_details.next().unwrap().as_str();
    let node_0 = node_details.next().unwrap().as_str();
    let node_1 = node_details.next().unwrap().as_str();

    let value = parse_value(node_details.next().unwrap());

    device::res::Res {
        name: String::from(name),
        nodes: vec![String::from(node_0), String::from(node_1)],
        val: value,
    }
}

fn parse_vdd(node: Pair<Rule>) -> device::vdd::Vdd {
    let mut node_details = node.into_inner();
    let name = node_details.next().unwrap().as_str();
    let node_1 = node_details.next().unwrap().as_str();
    let node_0 = node_details.next().unwrap().as_str();
    let val = parse_value(node_details.next().unwrap().into_inner().next().unwrap());

    device::vdd::Vdd {
        name: String::from(name),
        nodes: vec![String::from(node_0), String::from(node_1)],
        val: val,
    }
}

fn parse_idd(node: Pair<Rule>) -> device::idd::Idd {
    let mut node_details = node.into_inner();
    let name = node_details.next().unwrap().as_str();
    let node_1 = node_details.next().unwrap().as_str();
    let node_0 = node_details.next().unwrap().as_str();
    let val = parse_value(node_details.next().unwrap().into_inner().next().unwrap());

    device::idd::Idd {
        name: String::from(name),
        nodes: vec![String::from(node_0), String::from(node_1)],
        val: val,
    }
}

fn parse_ind(node: Pair<Rule>) -> device::ind::Ind {
    let mut node_details = node.into_inner();
    let name = node_details.next().unwrap().as_str();
    let node_1 = node_details.next().unwrap().as_str();
    let node_0 = node_details.next().unwrap().as_str();
    let value = parse_value(node_details.next().unwrap());

    device::ind::Ind {
        name: String::from(name),
        nodes: vec![String::from(node_0), String::from(node_1)],
        val: value,
    }
}

fn parse_cap(node: Pair<Rule>) -> device::cap::Cap {
    let mut node_details = node.into_inner();
    let name = node_details.next().unwrap().as_str();
    let node_1 = node_details.next().unwrap().as_str();
    let node_0 = node_details.next().unwrap().as_str();
    let value = parse_value(node_details.next().unwrap());

    device::cap::Cap {
        name: String::from(name),
        nodes: vec![String::from(node_0), String::from(node_1)],
        val: value,
    }
}

fn parse_dio(node: Pair<Rule>) -> device::diode::Diode {
    let mut node_details = node.into_inner();
    let name = node_details.next().unwrap().as_str();
    let node_1 = node_details.next().unwrap().as_str();
    let node_0 = node_details.next().unwrap().as_str();

    device::diode::Diode {
        name: String::from(name),
        nodes: vec![String::from(node_0), String::from(node_1)],
    }
}

fn parse_bjt(node: Pair<Rule>) -> device::npn::NPN {
    let mut node_details = node.into_inner();
    let name = node_details.next().unwrap().as_str();
    let node_0 = node_details.next().unwrap().as_str();
    let node_1 = node_details.next().unwrap().as_str();
    let node_2 = node_details.next().unwrap().as_str();

    device::npn::NPN {
        name: String::from(name),
        nodes: vec![
            String::from(node_0),
            String::from(node_1),
            String::from(node_2),
        ],
    }
}

fn parse_mos(node: Pair<Rule>) -> device::nmos::NMOS {
    let mut node_details = node.into_inner();

    let name = node_details.next().unwrap().as_str();
    let node_0 = node_details.next().unwrap().as_str();
    let node_1 = node_details.next().unwrap().as_str();
    let node_2 = node_details.next().unwrap().as_str();

    device::nmos::NMOS {
        name: String::from(name),
        nodes: vec![
            String::from(node_0),
            String::from(node_1),
            String::from(node_2),
        ],
    }
}

fn parse_op_cmd() -> command::Command {
    command::Command::Op
}

fn parse_dc_cmd(cmd: Pair<Rule>) -> command::Command {
    let mut cmd_details = cmd.into_inner();

    let source = cmd_details.next().unwrap().as_str();
    let start = parse_value(cmd_details.next().unwrap());
    let stop = parse_value(cmd_details.next().unwrap());
    let step = parse_value(cmd_details.next().unwrap());

    command::Command::DC(command::DCParams {
        source: String::from(source),
        start: start,
        stop: stop,
        step: step,
    })
}

fn parse_tran_cmd(cmd: Pair<Rule>) -> command::Command {
    let mut cmd_details = cmd.into_inner();

    let start = parse_value(cmd_details.next().unwrap());
    let stop = parse_value(cmd_details.next().unwrap());
    let step = parse_value(cmd_details.next().unwrap());

    command::Command::Tran(command::TranParams {
        start: start,
        stop: stop,
        step: step,
    })
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
            "f" => 1e-15,
            _ => unreachable!(),
        };
        value *= mult;
    }

    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_spice_file_v_divider() {
        let (elems, cmds) = parse_spice_file("test/v_divider.sp");

        assert_eq!(elems.len(), 3);
        assert_eq!(elems[0].get_name(), "V1");
        assert_eq!(elems[1].get_name(), "R12");
        assert_eq!(elems[2].get_name(), "R20");

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], command::Command::Op));
    }

    #[test]
    fn parse_spice_file_i_divider() {
        let (elems, cmds) = parse_spice_file("test/i_divider.sp");

        assert_eq!(elems.len(), 3);
        assert_eq!(elems[0].get_name(), "I1");
        assert_eq!(elems[1].get_name(), "R10a");
        assert_eq!(elems[2].get_name(), "R10b");

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], command::Command::Op));
    }

    #[test]
    fn parse_spice_file_r_d_direct() {
        let (elems, cmds) = parse_spice_file("test/r_d_direct.sp");

        assert_eq!(elems.len(), 3);
        assert_eq!(elems[0].get_name(), "V1");
        assert_eq!(elems[1].get_name(), "R12");
        assert_eq!(elems[2].get_name(), "D20");

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], command::Command::Op));
    }

    #[test]
    fn parse_spice_file_npn_test() {
        let (elems, cmds) = parse_spice_file("test/npn_test.sp");

        assert_eq!(elems.len(), 5);
        assert_eq!(elems[0].get_name(), "V01");
        assert_eq!(elems[1].get_name(), "V02");
        assert_eq!(elems[2].get_name(), "R23");
        assert_eq!(elems[3].get_name(), "R14");
        assert_eq!(elems[4].get_name(), "Q310");

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], command::Command::Op));
    }

    #[test]
    fn parse_spice_file_nmos_test() {
        let (elems, cmds) = parse_spice_file("test/nmos_test.sp");

        assert_eq!(elems.len(), 4);
        assert_eq!(elems[0].get_name(), "V01");
        assert_eq!(elems[1].get_name(), "V02");
        assert_eq!(elems[2].get_name(), "R23");
        assert_eq!(elems[3].get_name(), "M310");

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], command::Command::Op));
    }

    #[test]
    fn parse_spice_file_v_divider_sweep_test() {
        let (elems, cmds) = parse_spice_file("test/v_divider_sweep.sp");

        assert_eq!(elems.len(), 3);
        assert_eq!(elems[0].get_name(), "V1");
        assert_eq!(elems[1].get_name(), "R12");
        assert_eq!(elems[2].get_name(), "R20");

        assert_eq!(cmds.len(), 2);
        assert!(matches!(cmds[0], command::Command::Op));
        assert!(matches!(cmds[1], command::Command::DC(_)));
    }

    #[test]
    fn parse_spice_file_i_divider_sweep_test() {
        let (elems, cmds) = parse_spice_file("test/i_divider_sweep.sp");

        assert_eq!(elems.len(), 3);
        assert_eq!(elems[0].get_name(), "I1");
        assert_eq!(elems[1].get_name(), "R10a");
        assert_eq!(elems[2].get_name(), "R10b");

        assert_eq!(cmds.len(), 2);
        assert!(matches!(cmds[0], command::Command::Op));
        assert!(matches!(cmds[1], command::Command::DC(_)));
    }

    #[test]
    fn parse_res_generic() {
        let pair = SpiceParser::parse(Rule::r_node, "R1 1 0 R=2.2k")
            .unwrap()
            .next()
            .unwrap();
        let elem = parse_res(pair);

        assert_eq!(elem.name, "R1");
        assert_eq!(elem.nodes, ["1", "0"]);
        assert_eq!(elem.val, 2.2e3);
    }

    #[test]
    fn parse_vdd_generic() {
        let pair = SpiceParser::parse(Rule::v_node, "V1 1 0 4.0V")
            .unwrap()
            .next()
            .unwrap();
        let elem = parse_vdd(pair);

        assert_eq!(elem.name, "V1");
        assert_eq!(elem.nodes, ["0", "1"]);
        assert_eq!(elem.val, 4.0);
    }

    #[test]
    fn parse_idd_generic() {
        let pair = SpiceParser::parse(Rule::i_node, "I1 1 0 4.0mA")
            .unwrap()
            .next()
            .unwrap();
        let elem = parse_idd(pair);

        assert_eq!(elem.name, "I1");
        assert_eq!(elem.nodes, ["0", "1"]);
        assert_eq!(elem.val, 4.0e-3);
    }

    #[test]
    fn parse_ind_generic() {
        let pair = SpiceParser::parse(Rule::ind_node, "L1 1 0 L=1u")
            .unwrap()
            .next()
            .unwrap();
        let elem = parse_ind(pair);

        assert_eq!(elem.name, "L1");
        assert_eq!(elem.nodes, ["0", "1"]);
        assert_eq!(elem.val, 1e-6);
    }

    #[test]
    fn parse_cap_generic() {
        let pair = SpiceParser::parse(Rule::cap_node, "C1 1 0 C=1u")
            .unwrap()
            .next()
            .unwrap();
        let elem = parse_cap(pair);

        assert_eq!(elem.name, "C1");
        assert_eq!(elem.nodes, ["0", "1"]);
        assert_eq!(elem.val, 1e-6);
    }

    #[test]
    fn parse_dio_generic() {
        let pair = SpiceParser::parse(Rule::dio_node, "D1 1 0 d_model")
            .unwrap()
            .next()
            .unwrap();
        let elem = parse_dio(pair);

        assert_eq!(elem.name, "D1");
        assert_eq!(elem.nodes, ["0", "1"]);
    }

    #[test]
    fn parse_bjt_generic() {
        let pair = SpiceParser::parse(Rule::bjt_node, "Q1 1 2 3 0 q_model")
            .unwrap()
            .next()
            .unwrap();
        let elem = parse_bjt(pair);

        assert_eq!(elem.name, "Q1");
        assert_eq!(elem.nodes, ["1", "2", "3"]);
    }

    #[test]
    fn parse_nmos_generic() {
        let pair = SpiceParser::parse(Rule::mos_node, "M1 1 2 3 0 t_model")
            .unwrap()
            .next()
            .unwrap();
        let elem = parse_mos(pair);

        assert_eq!(elem.name, "M1");
        assert_eq!(elem.nodes, ["1", "2", "3"]);
    }

    #[test]
    fn parse_op_cmd_generic() {
        let _pair = SpiceParser::parse(Rule::op_cmd, ".op")
            .unwrap()
            .next()
            .unwrap();

        let cmd = parse_op_cmd();

        assert!(matches!(cmd, command::Command::Op));
    }

    #[test]
    fn parse_dc_cmd_v_generic() {
        let pair = SpiceParser::parse(Rule::dc_cmd, ".DC V1 0 1 1m")
            .unwrap()
            .next()
            .unwrap();

        let cmd = parse_dc_cmd(pair);

        assert!(matches!(cmd, command::Command::DC(_)));
        if let command::Command::DC(params) = cmd {
            assert_eq!(params.source, "V1");
            assert_eq!(params.start, 0.0);
            assert_eq!(params.stop, 1.0);
            assert_eq!(params.step, 1e-3);
        }
    }

    #[test]
    fn parse_dc_cmd_i_generic() {
        let pair = SpiceParser::parse(Rule::dc_cmd, ".DC I1 0 1 1m")
            .unwrap()
            .next()
            .unwrap();

        let cmd = parse_dc_cmd(pair);

        assert!(matches!(cmd, command::Command::DC(_)));
        if let command::Command::DC(params) = cmd {
            assert_eq!(params.source, "I1");
            assert_eq!(params.start, 0.0);
            assert_eq!(params.stop, 1.0);
            assert_eq!(params.step, 1e-3);
        }
    }

    #[test]
    fn parse_value_generic() {
        let test_vals = [("1.23", 1.23), ("-50", -50.0), ("1.3k", 1300.0)];

        for (tgt_str, tgt_val) in test_vals.iter() {
            let pair = SpiceParser::parse(Rule::value, tgt_str)
                .unwrap()
                .next()
                .unwrap();
            assert_eq!(&parse_value(pair), tgt_val);
        }
    }

    #[test]
    fn parse_value_prefixes() {
        let test_vals = [
            ("1.0f", 1e-15),
            ("1.0p", 1e-12),
            ("1.0n", 1.0e-9),
            ("1.0u", 1.0e-6),
            ("1.0m", 1.0e-3),
            ("1.0c", 1.0e-2),
            ("1.0d", 1.0e-1),
            ("1.0da", 1.0e1),
            ("1.0h", 1.0e2),
            ("1.0k", 1.0e3),
            ("1.0M", 1.0e6),
            ("1.0G", 1.0e9),
        ];
        for (tgt_str, tgt_val) in test_vals.iter() {
            let pair = SpiceParser::parse(Rule::value, tgt_str)
                .unwrap()
                .next()
                .unwrap();
            assert_eq!(&parse_value(pair), tgt_val);
        }
    }
}
