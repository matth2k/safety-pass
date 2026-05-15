/*!

  Simple netlist passes.

*/

use crate::{Cell, Error, Pass};
use safety_net::Netlist;
use std::fmt;
use std::rc::Rc;

/// Register passes in a wrapper enum for CLI arg parsing.
///
/// # Example
/// ```
/// use safety_pass::register_passes;
/// use safety_pass::{Pass, Cell};
/// use safety_pass::passes::PrintVerilog;
/// // This defines a enum called `BasicPasses` with unit variants.
/// // They operate on netlists containing `Cell` cells.
/// register_passes!(BasicPasses <Cell>;
///   /// A dummy pass that emits the Verilog of the netlist.
///   PrintVerilog);
/// ```
#[macro_export]
macro_rules! register_passes {
    ($e:ident < $i:ty > ; $($(#[$meta:meta])* $pass:ident),+ $(,)?) => {
        /// Enum containing all registered passes for argument parsing.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
        pub enum $e {
            $(
                $(#[$meta])*
                $pass
            ),+
        }

        impl std::fmt::Display for $e {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        impl $e {
            /// Returns a boxed instance of the pass corresponding to this variant.
            pub fn get_pass(&self) -> Box<dyn Pass<I = $i>> {
                match self {
                    $(Self::$pass => Box::new($pass),)+
                }
            }
        }
    };
}

/// A dummy pass that emits the Verilog of the netlist.
pub struct PrintVerilog;

impl fmt::Display for PrintVerilog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrintVerilog")
    }
}

impl Pass for PrintVerilog {
    type I = Cell;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        Ok(netlist.to_string())
    }
}

/// Print the dot graph of the netlist
#[cfg(feature = "graph")]
pub struct DotGraph;

#[cfg(feature = "graph")]
impl fmt::Display for DotGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DotGraph")
    }
}

#[cfg(feature = "graph")]
impl Pass for DotGraph {
    type I = Cell;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        Ok(netlist.dot_string()?)
    }
}

/// Clean the netlist
pub struct Clean;

impl fmt::Display for Clean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Clean")
    }
}

impl Pass for Clean {
    type I = Cell;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        let cleaned = netlist.clean()?;
        Ok(format!(
            "Cleaned {} objects. {} remain.",
            cleaned.len(),
            netlist.len()
        ))
    }
}

/// Rename wires and instances sequentially __0__, __1__, ...
pub struct RenameNets;

impl fmt::Display for RenameNets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RenameNets")
    }
}

impl Pass for RenameNets {
    type I = Cell;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        use safety_net::format_id;
        netlist.rename_nets(|_, i| format_id!("__{i}__"))?;
        Ok(format!("Renamed {} cells", netlist.len()))
    }
}

register_passes!(BasicPasses <Cell>;
    /// A dummy pass that emits the Verilog of the netlist.
    PrintVerilog,
    /// A pass that prints the dot graph of the netlist.
    #[cfg(feature = "graph")]
    DotGraph,
    /// A pass that cleans the netlist.
    Clean,
    /// A pass that renames wires and instances sequentially __0__, __1__, ...
    RenameNets,
);
