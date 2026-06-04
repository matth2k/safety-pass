/*!

  Simple netlist passes.

*/

use crate::{Cell, Pass};
use safety_net::{Error, Instantiable, Netlist};
use std::fmt;
use std::rc::Rc;

/// Register passes in a wrapper enum for CLI arg parsing.
/// *Passes which would like to be parameterized should define a tuple struct with a public PhantomData field.*
///
/// # See example:
///
/// - [`PrintVerilog`]
///
/// # Example
/// ```
/// use safety_pass::register_passes;
/// use safety_pass::{Pass, Cell};
/// use safety_pass::passes::PrintVerilog;
/// // This defines a enum called `BasicPasses` with unit variants.
/// // They operate on netlists containing `Cell` cells.
/// register_passes!(BasicPasses<Cell>;
///   /// A dummy pass that emits the Verilog of the netlist.
///   PrintVerilog<Cell>);
/// ```
#[macro_export]
macro_rules! register_passes {
    ($e:ident < $i:ty > ; $($(#[$meta:meta])* $pass:ident $(<$pass_ty:ty>)?),+ $(,)?) => {
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
                    $(Self::$pass => Box::new($pass $(::<$pass_ty>(std::marker::PhantomData::<$pass_ty>::default()))?),)+
                }
            }
        }
    };
}

/// A dummy pass that emits the Verilog of the netlist.
pub struct PrintVerilog<I: Instantiable>(pub std::marker::PhantomData<I>);

impl<I: Instantiable> fmt::Display for PrintVerilog<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrintVerilog")
    }
}

impl<I: Instantiable> fmt::Debug for PrintVerilog<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrintVerilog")
    }
}

impl<I: Instantiable> Pass for PrintVerilog<I> {
    type I = I;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        Ok(netlist.to_string())
    }
}

/// Print the dot graph of the netlist
#[cfg(feature = "graph")]
pub struct DotGraph<I: Instantiable>(pub std::marker::PhantomData<I>);

#[cfg(feature = "graph")]
impl<I: Instantiable> fmt::Display for DotGraph<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DotGraph")
    }
}

#[cfg(feature = "graph")]
impl<I: Instantiable> fmt::Debug for DotGraph<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DotGraph")
    }
}

#[cfg(feature = "graph")]
impl<I: Instantiable> Pass for DotGraph<I> {
    type I = I;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        netlist.dot_string()
    }
}

/// Clean the netlist
pub struct Clean<I: Instantiable>(pub std::marker::PhantomData<I>);

impl<I: Instantiable> fmt::Display for Clean<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Clean")
    }
}

impl<I: Instantiable> fmt::Debug for Clean<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Clean")
    }
}

impl<I: Instantiable> Pass for Clean<I> {
    type I = I;

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
pub struct RenameNets<I: Instantiable>(pub std::marker::PhantomData<I>);

impl<I: Instantiable> fmt::Display for RenameNets<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RenameNets")
    }
}

impl<I: Instantiable> fmt::Debug for RenameNets<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RenameNets")
    }
}

impl<I: Instantiable> Pass for RenameNets<I> {
    type I = I;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        use safety_net::format_id;
        netlist.rename_nets(|_, i| format_id!("__{i}__"))?;
        Ok(format!("Renamed {} cells", netlist.len()))
    }
}

/// A pass that runs all patterns to a covergence.
/// Checks patterns in insertion order
/// AndIdentity, OrIdentity, AndAbsorb, OrAbsorb, NandIdentity, NorIdentity, NandAbsorb, NorAbsorb,
/// DoubleNegation, Idempotent, MonotoneFold
#[derive(Debug)]
pub struct FoldAllPatterns;

impl fmt::Display for FoldAllPatterns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FoldAllPatterns")
    }
}

impl Pass for FoldAllPatterns {
    type I = Cell;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        use crate::patterns::{
            AndAbsorb, AndIdentity, DoubleNegation, Idempotent, MonotoneFold, NandAbsorb,
            NandIdentity, NorAbsorb, NorIdentity, OrAbsorb, OrIdentity,
        };
        let mut folder = crate::Folder::new(1000);
        folder.insert(AndIdentity);
        folder.insert(OrIdentity);
        folder.insert(NandIdentity);
        folder.insert(NorIdentity);
        folder.insert(AndAbsorb);
        folder.insert(OrAbsorb);
        folder.insert(NandAbsorb);
        folder.insert(NorAbsorb);
        folder.insert(DoubleNegation);
        folder.insert(Idempotent);
        folder.insert(MonotoneFold);
        folder.run(netlist)
    }
}

register_passes!(BasicPasses<Cell>;
    /// A pass that cleans the netlist.
    Clean<Cell>,
    /// A pass that prints the dot graph of the netlist.
    #[cfg(feature = "graph")]
    DotGraph<Cell>,
    /// A pass that runs all built-in patterns to a fixed point.
    FoldAllPatterns,
    /// A dummy pass that emits the Verilog of the netlist.
    PrintVerilog<Cell>,
    /// A pass that renames wires and instances sequentially __0__, __1__, ...
    RenameNets<Cell>,
);
