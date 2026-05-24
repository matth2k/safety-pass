![](https://github.com/matth2k/safety-pass/actions/workflows/rust.yml/badge.svg)
[![Docs](https://img.shields.io/badge/docs-github--pages-blue)](https://matth2k.github.io/safety-pass/)
[![crates.io](https://img.shields.io/badge/crates.io-github--pages-blue)](https://crates.io/crates/safety-pass)

# `nl_opt`: Verilog Compiler Driver for Netlist Optimization Development

## Description

This Verilog frontend tool can drive netlist optimizations on a pass-by-pass basis.

## Getting Started

Try running an `nl_opt` pattern on this example verilog:

```verilog
module top (
  a,
  b,
  y
);
  input a;
  wire a;
  input b;
  wire b;
  output y;
  wire y;
  wire inst_0_ZN;
  wire inst_1_ZN;
  AND2 inst_0 (
    .A1(a),
    .A2(b),
    .ZN(inst_0_ZN)
  );
  AND2 inst_1 (
    .A1(inst_0_ZN),
    .A2(inst_0_ZN),
    .ZN(inst_1_ZN)
  );
  assign y = inst_1_ZN;
endmodule
```

Save it to `ex.v`

Then run `nl_opt ex.v --passes clean,print-verilog`

## Help

```
Netlist optimization debugging tool

Usage: nl_opt [OPTIONS] [INPUT]

Arguments:
  [INPUT]
          Verilog file to read from (or use stdin)

Options:
  -x, --no-xilinx
          Do not parse with Xilinx-specific port names

      --verify
          Verify after every pass (not just the last)

  -v, --verbose
          Verbose logging

  -p, --passes <PASSES>
          A list of passes to run in order

          Possible values:
          - print-verilog: A dummy pass that emits the Verilog of the netlist
          - dot-graph:     A pass that prints the dot graph of the netlist
          - clean:         A pass that cleans the netlist
          - rename-nets:   A pass that renames wires and instances sequentially

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
