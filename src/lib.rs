#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
/*!

Compiler pass infrastructure for safety-net

*/

use log::info;
use safety_net::{DrivenNet, Identifier, Instantiable, NetRef, Netlist};
use std::{collections::HashSet, fmt, rc::Rc};
use thiserror::Error;

mod cells;

/// Errors for running passes
#[derive(Error, Debug)]
pub enum Error<'a, I: Instantiable> {
    /// A netlist error in running a pass.
    #[error("Pass error: {0}, {1}")]
    PassError(&'a dyn Pass<I = I>, safety_net::Error),
    /// A netlist error in applying a pattern.
    #[error("Pattern error: {0}, {1}")]
    PatternError(&'a dyn Pattern<I = I>, safety_net::Error),
    /// Any other netlist error.
    #[error("Netlist error: {0}")]
    Other(#[from] safety_net::Error),
}

impl<'a, I: Instantiable> Error<'a, I> {
    /// Returns the underlying error.
    pub fn unwrap(self) -> safety_net::Error {
        match self {
            Self::PassError(_, e) | Self::PatternError(_, e) | Self::Other(e) => e,
        }
    }
}

/// A pass on a netlist.
pub trait Pass: fmt::Debug + fmt::Display {
    /// The type of Instantiable in the netlist
    type I: Instantiable;

    /// Run the pass on the given netlist and return any info as a string.
    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, safety_net::Error>;

    /// Run the pass with verification before.
    fn run_verified(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, safety_net::Error> {
        netlist.verify()?;
        self.run(netlist)
    }
}

/// A sequential pipeline of [Pass]es
pub struct Pipeline<I: Instantiable> {
    passes: Vec<Box<dyn Pass<I = I>>>,
}

impl<I: Instantiable> Default for Pipeline<I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Instantiable> Pipeline<I> {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self { passes: vec![] }
    }

    /// Add a pass to the end of the pipeline
    pub fn insert<P: Pass<I = I> + 'static>(&mut self, pass: P) {
        self.passes.push(Box::new(pass));
    }

    /// Add a boxed pass to the end of the pipeline
    pub fn insert_dyn(&mut self, pass: Box<dyn Pass<I = I>>) {
        self.passes.push(pass);
    }

    /// Execute the pipeline on a netlist. If `verify` is true, verify the netlist after each pass.
    /// Returns the output of the last pass.
    pub fn execute(&self, netlist: &Rc<Netlist<I>>, verify: bool) -> Result<String, Error<'_, I>> {
        let mut res = String::new();
        let n = self.passes.len();
        for (i, pass) in self.passes.iter().enumerate() {
            info!("Running pass {i} ({pass})...");
            match pass.run(netlist) {
                Ok(output) => {
                    res = output;
                    if i != n - 1 {
                        if verify && let Err(e) = netlist.verify() {
                            return Err(Error::PassError(pass.as_ref(), e));
                        }
                        info!("{pass}: {}", res);
                    }
                }
                Err(e) => return Err(Error::PassError(pass.as_ref(), e)),
            }
        }
        netlist.verify()?;
        Ok(res)
    }
}

/// A function that inserts new cells into a netlist.
pub type Create<'a, I> = dyn Fn(I, Identifier) -> NetRef<I> + 'a;
/// A function that replaces the first arg with the second in a netlist.
pub type Replace<'a, I> =
    dyn FnMut(DrivenNet<I>, DrivenNet<I>) -> Result<(), safety_net::Error> + 'a;

/// A peephole pattern applied cell-wise in the netlist
pub trait Pattern: fmt::Debug + fmt::Display {
    /// The type of Instantiable in the netlist
    type I: Instantiable;

    /// Returns true if the pattern was matched and applied to the cell.
    /// Only return true if the netlist was modified.
    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, safety_net::Error>;
}

/// Greedily folds [Pattern]s
pub struct Folder<I: Instantiable> {
    patterns: Vec<Box<dyn Pattern<I = I>>>,
    max_iters: usize,
}

impl<I: Instantiable> Folder<I> {
    /// Create a new empty pipeline
    pub fn new(max_iters: usize) -> Self {
        Self {
            patterns: vec![],
            max_iters,
        }
    }

    /// Set the maximum number of iterations for folding
    pub fn with_max_iters(self, max_iters: usize) -> Self {
        Self { max_iters, ..self }
    }

    /// Add a pattern to the group
    pub fn insert<P: Pattern<I = I> + 'static>(&mut self, pattern: P) {
        self.patterns.push(Box::new(pattern));
    }

    /// Apply the patterns. Returns the number of iterations to find a fixed point.
    pub fn fold(&self, netlist: &Rc<Netlist<I>>) -> Result<usize, Error<'_, I>> {
        let mut cleaned: HashSet<NetRef<I>> = HashSet::new();
        let mut i = 0;
        let mut last_pat = None;
        let create = |t, i| netlist.insert_gate_disconnected(t, i);

        while i < self.max_iters {
            let mut replacements: Vec<(DrivenNet<I>, DrivenNet<I>)> = Vec::new();
            let mut replace = |a: DrivenNet<I>, b: DrivenNet<I>| {
                replacements.push((a, b));
                Ok(())
            };

            let mut change = false;
            'iter: for cell in netlist.objects() {
                if cleaned.contains(&cell) {
                    continue;
                }

                let ctype = cell.get_instance_type().map(|r| r.clone());
                if let Some(cell_type) = ctype {
                    for pattern in &self.patterns {
                        last_pat = Some(pattern.as_ref());
                        match pattern.apply(&cell, &cell_type, &create, &mut replace) {
                            Ok(true) => {
                                change = true;
                                break 'iter;
                            }
                            Err(e) => return Err(Error::PatternError(pattern.as_ref(), e)),
                            _ => (),
                        }
                    }
                }
            }

            if !change {
                break;
            }

            for (a, b) in replacements {
                let a = match (netlist.replace_net_uses(a, &b), last_pat) {
                    (Ok(a), _) => a,
                    (Err(e), Some(p)) => return Err(Error::PatternError(p, e)),
                    (Err(e), None) => return Err(e.into()),
                };
                cleaned.insert(a.unwrap());
            }

            i += 1;
        }

        drop(cleaned);
        netlist.clean()?;

        Ok(i)
    }
}

impl<I: Instantiable> fmt::Display for Folder<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PatternFolder({})", self.patterns.len())
    }
}

impl<I: Instantiable> fmt::Debug for Folder<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Folder")
            .field("patterns", &self.patterns)
            .field("max_iters", &self.max_iters)
            .finish()
    }
}

impl<C: Instantiable> Pass for Folder<C> {
    type I = C;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, safety_net::Error> {
        let iters = self.fold(netlist).map_err(|e| e.unwrap())?;
        Ok(format!(
            "Folded {} patterns over {} iterations",
            self.patterns.len(),
            iters
        ))
    }
}

pub mod passes;
pub mod patterns;
pub use cells::*;
