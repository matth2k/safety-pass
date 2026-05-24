![](https://github.com/matth2k/safety-pass/actions/workflows/rust.yml/badge.svg)
[![Docs](https://img.shields.io/badge/docs-github--pages-blue)](https://matth2k.github.io/safety-pass/)
[![crates.io](https://img.shields.io/badge/crates.io-github--pages-blue)](https://crates.io/crates/safety-pass)

# Safety Pass: Compiler Pass and Pattern Rewriting for Circuits

## Description

A Rust library for orchestrating compiler pass pipelines on safety-net netlists.

You can read the docs [here](https://matth2k.github.io/safety-pass/).

## Getting Started

### Pattern-Based Rewriting

This crate allows for greedy, pattern-based rewriting of a netlist. Here is an example pattern that folds `A && A => A` and `A || A => A` (idempotence):

```rust
impl Pattern for Idempotent {
    type I = Cell;

    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        _create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, Error> {
        if !matches!(
            cell_type.get_type(),
            CellType::AND | CellType::AND2 | CellType::OR | CellType::OR2
        ) {
            return Ok(false);
        }

        let a = cell.get_input(0).get_driver();
        let b = cell.get_input(1).get_driver();

        if a.is_none() || b.is_none() {
            return Ok(false);
        }

        let a = a.unwrap();
        let b = b.unwrap();

        if a != b {
            return Ok(false);
        }

        let c = cell.get_output(0);
        debug!("Pattern applied to cell {}!", c.as_net());
        replace(c, a)?;

        Ok(true)
    }
}
```

### Pass Pipelines

You can also compose all your transformations (passes) into a pipeline and run it on multiple netlists:

```rust
    let mut pipeline = Pipeline::default();

    // Make a greedy pattern folder
    let mut folder = Folder::new(100);
    folder.insert(safety_pass::patterns::Idempotent);

    // Add it to the pipeline
    pipeline.insert(folder);

    for pass in args.passes {
        pipeline.insert_dyn(pass.get_pass());
    }

    let output = pipeline
        .execute(&netlist, false)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    println!("{output}");
```
