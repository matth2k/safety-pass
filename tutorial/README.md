# safety-pass Beginner Tutorial

Tutorial walking through inspecting, repairing, transforming, and testing a circuit using safety-pass

## Prerequisites

- **Git** - download the repository
- **Rust and Cargo** - compile and run safety-pass
- **Graphviz** - turn a DOT graph into an image
- **Verilator** - simulate & test the Verilog circuit

Installation commands (macOS, Homebrew):

```bash
brew install graphviz
brew install verilator
```

Verify everything is available:

```bash
git --version
cargo --version
dot -V
verilator --version
```

### Common setup errors

If you see:

```text
dot: command not found
```

install Graphviz. The Rust `Broken pipe` message is only a consequence of the missing `dot` command.

If you see:

```text
make: verilator: No such file or directory
```

install Verilator and run the command again.

## 1. Download the tutorial

Clone the `tutorial` branch of the repository:

```bash
git clone --branch tutorial --single-branch https://github.com/matth2k/safety-pass.git
```

Enter the tutorial directory:

```bash
cd safety-pass/tutorial
```

The entire repo is required because the tutorial files depend on `safety-net`, `safety-pass`, and `nl_opt` code elsewhere in the project.

## 2. Visualize the broken circuit

Generate an image of the ripple-carry adder:

```bash
make rca.png
```

Open the image on macOS:

```bash
open rca.png
```

This command:

1. Parses `rca.v`
2. Converts the Verilog into a safety-net netlist
3. Runs the existing `dot-graph` pass
4. Uses Graphviz to create `rca.png`

The image has four full-adder cells `fa_0` through `fa_3` and the carry wires correctly connect `fa_0` to `fa_1` and `fa_1` to `fa_2`. 
`fa_3` has no wire entering its `CI` port -- the missing connection is `carry[2]`.

## 3. Repair the missing carry connection

Open `rca.v` and find the `fa_3` full adder. Its carry-input connection is commented out:

```verilog
// .CI(carry[2]),
```

Remove the two slashes:

```verilog
.CI(carry[2]),
```

This connects the carry output from `fa_2` to the carry input of `fa_3`.

Regenerate the image:

```bash
make -B rca.png
```

The `-B` forces `make` to rebuild the image. Open it again:

```bash
open rca.png
```

The graph should now show `carry[2]` connecting `fa_2` to the `CI` port of `fa_3`

## 4. Test the repaired circuit

Run a provided Verilator test:

```bash
make test
```

A four-bit input can represent the numbers 0 through 15; the test tries every pair of inputs giving:

```text
16 × 16 = 256 test cases
```

Every line should say `OK` and the final line should be:

```text
OK: 15 + 15 = 30
```

This shows the repaired circuit works before applying a transformation. If a later test fails we can then say the transformation caused the problem.

## 5. Create a work branch

Move to the repository root:

```bash
cd ..
```

Create a separate branch for your changes:

```bash
git switch -c tutorial-pass-work
```

Return to the tutorial directory:

```bash
cd tutorial
```

This is for you to develop your own pass.

## 6. Apply the starter pass

Run:

```bash
make patch
```

This applies `pass_template.patch` which adds a new pass named `MyPass` to:

```text
safety-pass/src/passes.rs
```

Only run `make patch` once. Running it again will fail because the changes have already been applied.

The important unfinished portion is:

```rust
for cell in netlist.matches(|p| p.get_type() == CellType::FA) {
    todo!("Do something with this full adder cell! Swap A and B?")
}
```

> Find every cell in the netlist whose type is `FA` then perform some operation on it.

The patch also registers `MyPass` with the command-line program. This allows it to be selected using

```bash
-p my-pass
```

## 7. Implement the transformation

Replace unfinished `Pass` implementation with:

```rust
impl Pass for MyPass {
    type I = Cell;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        use crate::CellType;

        let mut swapped = 0;

        for cell in netlist.matches(|p| p.get_type() == CellType::FA) {
            let Some(a) = cell.find_input(&"A".into()) else {
                continue;
            };
            let Some(b) = cell.find_input(&"B".into()) else {
                continue;
            };
            let Some(a_driver) = a.get_driver() else {
                continue;
            };
            let Some(b_driver) = b.get_driver() else {
                continue;
            };

            a.connect(b_driver);
            b.connect(a_driver);
            swapped += 1;
        }

        Ok(format!(
            "Swapped A and B inputs on {swapped} full adders"
        ))
    }
}
```

For every full adder the pass retrieves `A` and `B` input ports and remembers their original driver nets. 
It reconnects each port to the other port's driver. Cells with missing ports / drivers are skipped.

Check and run the implementation:

```bash
cargo check
cargo run --release --quiet -- rca.v -p my-pass
```

Should report:

```text
Swapped A and B inputs on 4 full adders
```

## 8. Verify the structural change

Run `MyPass` followed by `dot-graph` in the same pipeline:

```bash
cargo run --release --quiet -- rca.v \
    -p my-pass \
    -p dot-graph |
    dot -Tpng > rca_swapped.png
```

`MyPass` first modifies the in memory netlist and `dot-graph` then visualizes the modified version

In `rca_swapped.png` verify:

- Each `b[i]` wire now connects to port `A`
- Each `a[i]` wire now connects to port `B`
- The carry chain remains unchanged

## 9. Verify that behavior is preserved

Emit the transformed netlist as Verilog:

```bash
cargo run --release --quiet -- rca.v \
    -p my-pass \
    -p print-verilog \
    > rca_swapped.v
```

Compile the transformed circuit in a separate Verilator build directory:

```bash
verilator --cc --exe --build \
    -Wno-PINMISSING \
    --top-module rca \
    --Mdir obj_dir_swapped \
    rca_main.cpp cells.v rca_swapped.v
```

`cells.v` is included because `rca_swapped.v` contains the transformed `rca` module but still relies on the separate `FA` module definition

Run the transformed simulation:

```bash
./obj_dir_swapped/Vrca
```

All 256 cases should pass.

## Summary of the workflow 

Congratulations! Here's what you accomplished: 

1. Compile Verilog into a safety-net netlist
2. Find cells of a specific type
3. Inspect and modify their connections
4. Visualize the transformed structure
5. Emit the transformed netlist as Verilog
6. Verify that the transformation preserves behavior