# safety-pass Beginner Tutorial

Safety Pass is a toolchain that allows you to build, inspect, and rewrite Verilog netlists with composable compiler passes written in Rust. Safety Pass gives hardware developers a fast path from an optimization idea to a testable implementation without needing to build an entire compiler framework first.

Broadly speaking, circuit transformations are easy to describe but surprisingly difficult to implement *safely*. Safety Pass provides graph traversals, reusable rewriting patterns, pass pipelines, and visualization that all abides by a reference-counted definition of memory safety. Safety Pass can be used to prototype synthesis optimizations, inspect unfamiliar netlists, teach compiler transformations, debug wire connectivity, or build specialized Verilog tooling.

Safety Pass's provided `nl_opt` tool keeps the entire optimization loop in one composable workflow: parse → transform → verify → emit.

## Some Misc. Capabilities

- Parse Verilog into a programmable netlist.
- Chain analyses and transformations in a reconfigurable order.
- Apply greedy rewrites to a fixed point.
- Emit Verilog or DOT graphs and verify structure after each pass.

## Prerequisites

The guided tutorial starts with a broken ripple-carry adder, finds the buggy connection, experiments with a transform on all 4 full adders, and verifies the result across all 256 possible four-bit input pairs. However, you do need some basic external tools:

- **Git** - download the repository
- **Rust and Cargo** - compile and run safety-pass
- **Graphviz** - turn a DOT graph into an image
- **Verilator** - simulate & test the Verilog circuit

Installation commands:

**macOS (Homebrew):**

```bash
brew install graphviz
brew install verilator
```

**Ubuntu/Debian:**

```bash
sudo apt update
sudo apt install graphviz verilator
```

Verify everything is available on either system:

```bash
git --version
cargo --version
dot -V
verilator --version
```

If things are not working, checkout [common setup errors](#common-setup-errors).

## 1. Download the tutorial, create a work branch

Clone the repository:

```bash
git clone https://github.com/matth2k/safety-pass.git
```

Enter the repository:

```bash
cd safety-pass
```

The entire repo is required because the tutorial files depend on the [`safety-net` dependency](../safety-pass/Cargo.toml#L37-L42), the [`safety-pass` library](../safety-pass/), and the [`nl_opt` command-line entry point](../nl_opt/src/main.rs) elsewhere in the project.

Before making any changes, create a separate branch:

```bash
git switch -c tutorial-pass-work
```

Then enter the tutorial directory:

```bash
cd tutorial
```

## 2. Visualize the broken circuit

Generate an image of the netlist:

```bash
make rca.png
```

`make` reads the tutorial's [`Makefile`](./Makefile) and runs [the recipe associated with the `rca.png` target](./Makefile#L14-L16). It also tracks dependencies, so it only rebuilds `rca.png` when the file is missing or [`rca.v`](./rca.v) has changed.

For this target, `make` runs:

`cargo run --release --quiet -- rca.v -p dot-graph | dot -Tpng > rca.png`

This runs `safety-pass` on [`rca.v`](./rca.v) with the [`dot-graph` pass](../safety-pass/src/passes.rs#L87-L112), pipes the resulting DOT graph into Graphviz (`dot -Tpng`), and writes the rendered image to `rca.png`.

Open the generated image:

**macOS:**

```bash
open rca.png
```

**Ubuntu with a desktop environment:**

```bash
xdg-open rca.png
```

To reiterate, we have:

1. Parsed `rca.v`
2. Converted the Verilog into a safety-net netlist
3. Ran the existing `dot-graph` pass
4. Used Graphviz to create `rca.png`

In [the RCA implementation](./rca.v#L12-L44), the first three full adders form a carry chain:

> `fa_0 --carry[0]--> fa_1 --carry[1]--> fa_2`

However, [`fa_3` is not connected to that chain](./rca.v#L38-L44). There is no wire from `carry[2]` entering the `CI` port of `fa_3`.

The complete chain should instead be:

> `fa_0 --carry[0]--> fa_1 --carry[1]--> fa_2 --carry[2]--> fa_3`

## 3. Test the broken circuit

Run the provided Verilator test before fixing anything. [The `test` target](./Makefile#L7-L8) runs the executable built by [the Verilator build target](./Makefile#L10-L12) from [`rca_main.cpp`](./rca_main.cpp):

```bash
make test
```

The first failing case, reported by [the testbench's result check](./rca_main.cpp#L14-L20), should be:

> `ERROR: 1 + 7 != 0`

Why does `1 + 7` expose the bug?

> `1 + 7 = 8 = 1000₂`

Producing the `1` in the highest output position requires a carry to propagate from `fa_2` into `fa_3`. Because that connection is missing, the final full adder never receives the carry.

## 4. Repair the broken circuit

Open [`rca.v` at the `fa_3` full adder](./rca.v#L38-L44). Its [carry-input connection](./rca.v#L41) is erroneously commented out:

`// .CI(carry[2]),`

Remove the two slashes:

`.CI(carry[2]),`

This connects the carry output from `fa_2` to the carry input of `fa_3`.

Regenerate the image with [the same `rca.png` Makefile target](./Makefile#L14-L16):

```bash
make rca.png
```

Rebuild the image, and open it again. The graph should now show `carry[2]` connecting `fa_2` to the `CI` port of `fa_3`

## 5. Test the repaired circuit

Run a provided Verilator test with [the `test` Makefile target](./Makefile#L7-L8):

```bash
make test
```

A four-bit input can represent the numbers 0 through 15; [the test's nested loops](./rca_main.cpp#L8-L23) try every pair of inputs, from `0 + 0` through `15 + 15`, giving:

> 16 × 16 = 256 test cases

Every line should say `OK` and the final line should be:

> `OK: 15 + 15 = 30`

At this point, we know the original circuit works correctly. This gives us a baseline. If the circuit stops working after our transformation, the transformation introduced the problem.

## 6. Make your own `nl_opt` pass

Before implementing the transformation, it helps to understand what a **compiler pass** is.

A compiler generally represents its input using an internal data structure called an **intermediate representation**, or IR. Here, the IR is a [`safety-net` dependency](https://crates.io/crates/safety-net) **netlist**: a graph containing hardware cells and the wires connecting them.

A **compiler pass** performs one operation over that representation. A pass might:

- inspect cells and connections
- collect information about the circuit
- modify the circuit
- produce another representation such as a graph or Verilog

The [`safety-pass` pass implementations](../safety-pass/src/passes.rs) contain these analyses and transformations, and [`nl_opt`](../nl_opt/src/main.rs#L83-L162) allows passes to be run from the command line.

For this tutorial, you will implement a pass called [`MyPass`](./pass_template.patch#L9-L30).

Rather than manually writing all the boilerplate required to add a new pass, the tutorial provides [`pass_template.patch`](./pass_template.patch):

`pass_template.patch`

This patch contains the **starter code needed to add a new compiler pass with `nl_opt`**. The [`todo!`](./pass_template.patch#L24-L26) intentionally leaves the actual transformation unfinished for you to implement.

Apply it with [the `patch` Makefile target](./Makefile#L18-L20):

```bash
make patch
```

This applies [`pass_template.patch`](./pass_template.patch), which adds a new pass named [`MyPass`](./pass_template.patch#L9-L30) to [`safety-pass/src/passes.rs`](../safety-pass/src/passes.rs).

Only run `make patch` once. Running it again will fail because the changes have already been applied.

The important unfinished portion is:

> ```
> for cell in netlist.matches(|p| p.get_type() == CellType::FA) {
>     todo!("Do something with this full adder cell! Swap A and B?")
> }
> ```

This snippet of code finds every cell in the netlist whose type is [`CellType::FA`](../safety-pass/src/cells.rs#L68) then perform some operation on it (which you will fill in).

The patch also [registers `MyPass`](./pass_template.patch#L35-L42) with the command-line program. This makes `-p my-pass` available through the pass registry and allows it to be selected using:

```bash
nl_opt -p my-pass
```

## 7. Implement the transformation

Your goal is to modify every full adder so that its `A` and `B` inputs are swapped.

Before:

> ```
> a[i] drives A input
> b[i] drives B input
> ```

After:

> ```
> b[i] drives A input
> a[i] drives B input
> ```

This should not change the behavior of the circuit because [the full-adder logic](./cells.v#L2-L13) treats `A` and `B` symmetrically (i.e. addition is commutative).

Here are some useful methods to help you accomplish this rewrite:

[`find_input(...)`](https://matth2k.github.io/safety-pass/safety_net/trait.Instantiable.html#method.find_input)

Find an input port belonging to a cell.

[`get_driver()`](https://matth2k.github.io/safety-pass/safety_net/struct.InputPort.html#method.get_driver)

Find the net currently driving that input.

[`connect(...)`](https://matth2k.github.io/safety-pass/safety_net/struct.InputPort.html#method.connect)

Connect an input to a different driver.

A useful approach is:

1. Find the `A` input
2. Find the `B` input
3. Save both original drivers
4. Connect `A` to the original `B` driver
5. Connect `B` to the original `A` driver
6. Return a string message to the user on how many full adders were modified

You will need to make **temporary variables** to save the original drivers such that they can be swapped.

Check and run the implementation:

```bash
cargo check
cargo run --release --quiet -- rca.v -p my-pass
```

Depending on the return message you wrote, you should see something like:

> `Swapped A and B inputs on 4 full adders`

A reference solution is provided separately in [`pass_solution.patch`](./pass_solution.patch).

If you want to apply the reference solution immediately after `make patch`, [the `solution` Makefile target](./Makefile#L22-L24) applies [`pass_solution.patch`](./pass_solution.patch):

```bash
make solution
```

## 8. Observing the structural change

Run `MyPass` followed by the [`dot-graph` pass](../safety-pass/src/passes.rs#L87-L112) in the same pipeline (the [`nl_opt` pipeline handling](../nl_opt/src/main.rs#L125-L137) runs the passes in the order of the command arguments):

```bash
cargo run --release --quiet -- rca.v \
    -p my-pass \
    -p dot-graph |
    dot -Tpng > rca_swapped.png
```

`MyPass` first modifies the in-memory netlist and `dot-graph` then visualizes the modified version.

Open the result:

**macOS:**

```bash
open rca_swapped.png
```

**Ubuntu with a desktop environment:**

```bash
xdg-open rca_swapped.png
```

In `rca_swapped.png` verify:

- Each `b[i]` wire now connects to port `A`
- Each `a[i]` wire now connects to port `B`
- The carry chain remains unchanged

## 9. Verify that behavior is preserved

Emit the transformed netlist as Verilog with the [`print-verilog` pass](../safety-pass/src/passes.rs#L62-L85):

```bash
cargo run --release --quiet -- rca.v \
    -p my-pass \
    -p print-verilog \
    > rca_swapped.v
```

Compile the transformed circuit in a separate Verilator build directory, reusing [`rca_main.cpp`](./rca_main.cpp) as the testbench:

```bash
verilator --cc --exe --build \
    -Wno-PINMISSING \
    --top-module rca \
    --Mdir obj_dir_swapped \
    rca_main.cpp cells.v rca_swapped.v
```

[`cells.v`](./cells.v) is included in order to provide the [the separate `FA` module definition](./cells.v#L2-L13).

Lastly, run the simulation on the transformed circuit:

```bash
./obj_dir_swapped/Vrca
```

[All 256 cases](./rca_main.cpp#L8-L23) should pass.

## Summary of the workflow

Congratulations! Here's what you accomplished:

1. Compile Verilog into a safety-net netlist
2. Traverse cells of a specific type
3. Inspect and modify their connections
4. Visualize the transformed structure
5. Emit the transformed netlist as Verilog
6. Verify that the transformation preserves behavior

## Common setup errors

If you see:

> `dot: command not found`

install Graphviz. The Rust `Broken pipe` message is only a consequence of the missing `dot` command.

If you see:

> `make: verilator: No such file or directory`

install Verilator and run the command again.

If a program is installed but your terminal still cannot find it, check your `PATH` environment variable. `PATH` is the list of directories your shell searches when you type a command.

```bash
echo $PATH
which dot
which verilator
```

If `which` prints nothing, the executable is not currently visible through your `PATH`.

**macOS (Homebrew):**

You can find Homebrew's installation directory with:

```bash
brew --prefix
```

Make sure its `bin` directory is included in your `PATH`.

**Ubuntu/Debian:**

Programs installed with `apt` are normally placed in standard locations such as `/usr/bin`, which should already be in your `PATH`. You can check with:

```bash
ls -l /usr/bin/dot
ls -l /usr/bin/verilator
```

## Interested?

Try it: install nl_opt with cargo `cargo install nl_opt`

Learn it: Safety-pass provides the compiler infrastructure while safety-net is the actual IR API: https://matth2k.github.io/safety-net/safety_net/index.html

Extend it: Try to provide an EDIF or BLIF frontend to safety-net: https://github.com/matth2k/nl-compiler
