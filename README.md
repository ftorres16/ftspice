# ftspice

Rust implementation of a basic SPICE simulator.

Intended as a learning excercise.
Based on my work for ECE1777H taught by Prof. Farid Najm at the University of Toronto and his book titled ["Circuit Simulation"](https://onlinelibrary.wiley.com/doi/book/10.1002/9780470561218).

In its current version it supports the following:

- Simulation modes:
  - Operation point (`.op`)
  - DC Sweep (`.dc <source_name> <start> <stop> <step>`)
  - Transient (`.tran <stop> <step>`)
- Devices:
  - Independent voltage/current sources
    - Constant values
    - Functional: Sine, Pulse, Exp
  - Arbitrary linear Resistors
  - Arbitrary linear Capacitors
  - Arbitrary linear Inductors
  - One specific Diode model
  - One specific NPN model
  - One specific NMOS model

## Usage

The project is managed with `cargo`, the rust package manager.

You can run the simulator with `cargo run -- <path to a SPICE netlist>`.

Example netlists are inlcuded in the `tests/` folder.
