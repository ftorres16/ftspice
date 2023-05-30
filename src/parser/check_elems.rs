use std::collections::HashSet;

use crate::device::Stamp;
use crate::node::GND;

pub fn check_elems(elems: &[Box<dyn Stamp>]) {
    check_duplicate_names(elems);
    check_gnd(elems);
}

fn check_duplicate_names(elems: &[Box<dyn Stamp>]) {
    let names = elems.iter().map(|x| x.get_name()).collect::<HashSet<_>>();
    assert_eq!(names.len(), elems.len(), "Duplicate elems found!");
}

fn check_gnd(elems: &[Box<dyn Stamp>]) {
    elems
        .iter()
        .flat_map(|e| e.get_nodes().iter())
        .position(|n| n == &GND)
        .expect("GND node not found!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device;

    #[test]
    #[should_panic(expected = "Duplicate elems found!")]
    fn test_duplicate_names_fail() {
        let elems: Vec<Box<dyn Stamp>> = vec![
            Box::new(device::res::Res {
                name: String::from("R1"),
                nodes: vec![String::from("0"), String::from("1")],
                val: 1e3,
            }),
            Box::new(device::res::Res {
                name: String::from("R1"),
                nodes: vec![String::from("0"), String::from("3")],
                val: 1e3,
            }),
        ];

        check_duplicate_names(&elems);
    }

    #[test]
    fn test_duplicate_names_succeed() {
        let elems: Vec<Box<dyn Stamp>> = vec![
            Box::new(device::res::Res {
                name: String::from("R1"),
                nodes: vec![String::from("0"), String::from("1")],
                val: 1e3,
            }),
            Box::new(device::res::Res {
                name: String::from("R2"),
                nodes: vec![String::from("0"), String::from("3")],
                val: 1e3,
            }),
        ];

        check_duplicate_names(&elems);
    }

    #[test]
    #[should_panic(expected = "GND node not found!")]
    fn test_no_gnd_fail() {
        let elems: Vec<Box<dyn Stamp>> = vec![Box::new(device::res::Res {
            name: String::from("R1"),
            nodes: vec![String::from("1"), String::from("2")],
            val: 1e3,
        })];

        check_gnd(&elems);
    }

    #[test]
    fn test_no_gnd_succeed() {
        let elems: Vec<Box<dyn Stamp>> = vec![Box::new(device::res::Res {
            name: String::from("R1"),
            nodes: vec![String::from("0"), String::from("2")],
            val: 1e3,
        })];

        check_gnd(&elems);
    }
}
