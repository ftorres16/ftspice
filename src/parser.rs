use std::fs;

use crate::command;
use crate::device;
use crate::device::Stamp;
use crate::spice_fn::{ExpParams, PulseParams, SineParams, SpiceFn};

use pest::iterators::Pair;
use pest::Parser;

pub mod check_elems;

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

    let val = parse_value(node_details.next().unwrap());

    device::res::Res {
        name: String::from(name),
        nodes: vec![String::from(node_0), String::from(node_1)],
        val,
    }
}

fn parse_vdd(node: Pair<Rule>) -> device::vdd::Vdd {
    let mut node_details = node.into_inner();
    let name = node_details.next().unwrap().as_str();
    let node_1 = node_details.next().unwrap().as_str();
    let node_0 = node_details.next().unwrap().as_str();

    let val_details = node_details.next().unwrap().into_inner().next().unwrap();
    let val;
    let tran_fn;

    match val_details.as_rule() {
        Rule::v_dc_value => {
            val = parse_value(val_details.into_inner().next().unwrap());
            tran_fn = None;
        }
        Rule::fn_value => {
            let spice_fn = parse_spice_fn(val_details.into_inner().next().unwrap());
            val = spice_fn.eval(&0.0).clone();
            tran_fn = Some(spice_fn);
        }
        _ => unreachable!(),
    };

    device::vdd::Vdd {
        name: String::from(name),
        nodes: vec![String::from(node_0), String::from(node_1)],
        val,
        tran_fn,
    }
}

fn parse_idd(node: Pair<Rule>) -> device::idd::Idd {
    let mut node_details = node.into_inner();
    let name = node_details.next().unwrap().as_str();
    let node_1 = node_details.next().unwrap().as_str();
    let node_0 = node_details.next().unwrap().as_str();

    let val_details = node_details.next().unwrap().into_inner().next().unwrap();
    let val;
    let tran_fn;

    match val_details.as_rule() {
        Rule::i_dc_value => {
            val = parse_value(val_details.into_inner().next().unwrap());
            tran_fn = None;
        }
        Rule::fn_value => {
            let spice_fn = parse_spice_fn(val_details.into_inner().next().unwrap());
            val = spice_fn.eval(&0.0).clone();
            tran_fn = Some(spice_fn);
        }
        _ => unreachable!(),
    };

    device::idd::Idd {
        name: String::from(name),
        nodes: vec![String::from(node_0), String::from(node_1)],
        val,
        tran_fn,
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
        u_curr: None,
        i_curr: None,
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
        u_curr: None,
        i_curr: None,
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

    let stop = parse_value(cmd_details.next().unwrap());
    let step = parse_value(cmd_details.next().unwrap());

    command::Command::Tran(command::TranParams {
        start: 0.0,
        stop,
        step,
    })
}

fn parse_spice_fn(fn_value: Pair<Rule>) -> SpiceFn {
    match fn_value.as_rule() {
        Rule::sine_fn => {
            let mut fn_details = fn_value.into_inner();
            let offset = parse_value(fn_details.next().unwrap());
            let amplitude = parse_value(fn_details.next().unwrap());
            let freq = parse_value(fn_details.next().unwrap());

            SpiceFn::Sine(SineParams {
                offset,
                amplitude,
                freq,
            })
        }
        Rule::pulse_fn => {
            let mut fn_details = fn_value.into_inner();
            let v1 = parse_value(fn_details.next().unwrap());
            let v2 = parse_value(fn_details.next().unwrap());
            let delay = parse_value(fn_details.next().unwrap());
            let t_rise = parse_value(fn_details.next().unwrap());
            let t_fall = parse_value(fn_details.next().unwrap());
            let pulse_width = parse_value(fn_details.next().unwrap());
            let period = parse_value(fn_details.next().unwrap());

            SpiceFn::Pulse(PulseParams {
                v1,
                v2,
                delay,
                t_rise,
                t_fall,
                pulse_width,
                period,
            })
        }
        Rule::exp_fn => {
            let mut fn_details = fn_value.into_inner();
            let v1 = parse_value(fn_details.next().unwrap());
            let v2 = parse_value(fn_details.next().unwrap());
            let rise_delay = parse_value(fn_details.next().unwrap());
            let rise_tau = parse_value(fn_details.next().unwrap());
            let fall_delay = parse_value(fn_details.next().unwrap());
            let fall_tau = parse_value(fn_details.next().unwrap());

            SpiceFn::Exp(ExpParams {
                v1,
                v2,
                rise_delay,
                rise_tau,
                fall_delay,
                fall_tau,
            })
        }
        _ => unreachable!(),
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
    fn parse_spice_file_rc_tran_test() {
        let (elems, cmds) = parse_spice_file("test/rc.sp");

        assert_eq!(elems.len(), 3);
        assert_eq!(elems[0].get_name(), "V01");
        assert_eq!(elems[1].get_name(), "R12");
        assert_eq!(elems[2].get_name(), "C20");

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], command::Command::Tran(_)));
    }

    #[test]
    fn parse_spice_file_rc_sine_tran_test() {
        let (elems, cmds) = parse_spice_file("test/rc_sine.sp");

        assert_eq!(elems.len(), 3);
        assert_eq!(elems[0].get_name(), "V01");
        assert_eq!(elems[1].get_name(), "R12");
        assert_eq!(elems[2].get_name(), "C20");

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], command::Command::Tran(_)));
    }

    #[test]
    fn parse_spice_file_rc_pulse_tran_test() {
        let (elems, cmds) = parse_spice_file("test/rc_pulse.sp");

        assert_eq!(elems.len(), 3);
        assert_eq!(elems[0].get_name(), "V01");
        assert_eq!(elems[1].get_name(), "R12");
        assert_eq!(elems[2].get_name(), "C20");

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], command::Command::Tran(_)));
    }

    #[test]
    fn parse_spice_file_rc_exp_tran_test() {
        let (elems, cmds) = parse_spice_file("test/rc_exp.sp");

        assert_eq!(elems.len(), 3);
        assert_eq!(elems[0].get_name(), "V01");
        assert_eq!(elems[1].get_name(), "R12");
        assert_eq!(elems[2].get_name(), "C20");

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], command::Command::Tran(_)));
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
        assert!(matches!(elem.tran_fn, None));
    }

    #[test]
    fn parse_vdd_sine() {
        let pair = SpiceParser::parse(Rule::v_node, "V1 1 0 SIN(0.0 1.0 10k)")
            .unwrap()
            .next()
            .unwrap();
        let elem = parse_vdd(pair);

        assert_eq!(elem.name, "V1");
        assert_eq!(elem.nodes, ["0", "1"]);
        assert_eq!(elem.val, 0.0);

        let tran_fn = elem.tran_fn.expect("Tran Fn not set");
        if let SpiceFn::Sine(params) = tran_fn {
            assert_eq!(params.offset, 0.0);
            assert_eq!(params.amplitude, 1.0);
            assert_eq!(params.freq, 10e3);
        } else {
            panic!("Tran Function is not Sine");
        }
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
    fn parse_idd_sine() {
        let pair = SpiceParser::parse(Rule::i_node, "I1 1 0 SIN(0.0 1.0 10k)")
            .unwrap()
            .next()
            .unwrap();
        let elem = parse_idd(pair);

        assert_eq!(elem.name, "I1");
        assert_eq!(elem.nodes, ["0", "1"]);
        assert_eq!(elem.val, 0.0);

        let tran_fn = elem.tran_fn.expect("Tran Fn not set");
        if let SpiceFn::Sine(params) = tran_fn {
            assert_eq!(params.offset, 0.0);
            assert_eq!(params.amplitude, 1.0);
            assert_eq!(params.freq, 10e3);
        } else {
            panic!("Tran Function is not Sine");
        }
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
        assert_eq!(elem.u_curr, None);
        assert_eq!(elem.i_curr, None);
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
        assert_eq!(elem.u_curr, None);
        assert_eq!(elem.i_curr, None);
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
    fn parse_tran_cmd_generic() {
        let pair = SpiceParser::parse(Rule::tran_cmd, ".TRAN 1 1m")
            .unwrap()
            .next()
            .unwrap();

        let cmd = parse_tran_cmd(pair);

        assert!(matches!(cmd, command::Command::Tran(_)));
        if let command::Command::Tran(params) = cmd {
            assert_eq!(params.start, 0.0);
            assert_eq!(params.stop, 1.0);
            assert_eq!(params.step, 1e-3);
        }
    }

    #[test]
    fn parse_spice_fn_sine() {
        let pair = SpiceParser::parse(Rule::fn_value, "SIN(0.0 1.0 10k)")
            .unwrap()
            .next()
            .unwrap()
            .into_inner()
            .next()
            .unwrap();

        let fn_ = parse_spice_fn(pair);

        assert!(matches!(fn_, SpiceFn::Sine(_)));
        if let SpiceFn::Sine(params) = fn_ {
            assert_eq!(params.offset, 0.0);
            assert_eq!(params.amplitude, 1.0);
            assert_eq!(params.freq, 10e3);
        } else {
            panic!("Tran Function is not Sine");
        }
    }

    #[test]
    fn parse_spice_fn_pulse() {
        let pair = SpiceParser::parse(Rule::fn_value, "PULSE(0.0 1.0 0.0 1p 1p 5n 10n)")
            .unwrap()
            .next()
            .unwrap()
            .into_inner()
            .next()
            .unwrap();

        let fn_ = parse_spice_fn(pair);

        assert!(matches!(fn_, SpiceFn::Pulse(_)));
        if let SpiceFn::Pulse(params) = fn_ {
            assert_eq!(params.v1, 0.0);
            assert_eq!(params.v2, 1.0);
            assert_eq!(params.delay, 0.0);
            assert_eq!(params.t_rise, 1e-12);
            assert_eq!(params.t_fall, 1e-12);
            assert_eq!(params.pulse_width, 5e-9);
            assert_eq!(params.period, 10e-9);
        } else {
            panic!("Tran Function is not Pulse");
        }
    }

    #[test]
    fn parse_spice_fn_exp() {
        let pair = SpiceParser::parse(Rule::fn_value, "EXP(0.0 1.0 0.0 1n 5n 1n)")
            .unwrap()
            .next()
            .unwrap()
            .into_inner()
            .next()
            .unwrap();

        let fn_ = parse_spice_fn(pair);

        assert!(matches!(fn_, SpiceFn::Exp(_)));
        if let SpiceFn::Exp(params) = fn_ {
            assert_eq!(params.v1, 0.0);
            assert_eq!(params.v2, 1.0);
            assert_eq!(params.rise_delay, 0.0);
            assert_eq!(params.rise_tau, 1e-9);
            assert_eq!(params.fall_delay, 5e-9);
            assert_eq!(params.fall_tau, 1e-9);
        } else {
            panic!("Tran Function is not Exp");
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
