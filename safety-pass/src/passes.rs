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
        let mut folder = crate::Folder::new(100000);
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

/// Insert a pair of inverters at every point in the netlist.
#[derive(Debug)]
pub struct InsertInv;

impl fmt::Display for InsertInv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InsertInv")
    }
}

impl Pass for InsertInv {
    type I = Cell;
    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        use crate::CellType;
        let mut everything = Vec::new();

        for node in netlist.objects() {
            for output in node.outputs() {
                everything.push(output);
            }
        }

        // n increases with every run of InsertInv, ensuring the net names are unique.
        let n = everything.len();

        // We use i to differentiate between nets that have the same base identifer.
        for (i, net) in everything.into_iter().enumerate() {
            // Combine the net's base name (n) and i to to create unique instance names
            // across both repeated runs of this pass and nets with identical base names.
            let inst_name = net.as_net().get_identifier().clone()
                + "_inv".into()
                + n.to_string().into()
                + i.to_string().into();

            let inv_type = match net.get_instance_type() {
                Some(t) => t.new_like(CellType::INV),
                _ => Cell::new(CellType::INV, None),
            };

            let net_inv = netlist.insert_gate_disconnected(inv_type.clone(), inst_name.clone());

            // Repeat the pattern for the second inverter
            let inst_name = inst_name + "inv".into() + n.to_string().into() + i.to_string().into();
            let net_inv_inv =
                netlist.insert_gate(inv_type.clone(), inst_name, &[net_inv.clone().into()])?;

            // Replace the uses of the original net
            let replacement = net_inv_inv.get_output(0);
            let disconnected = netlist.replace_net_uses(net, &replacement)?;

            // Now take our disconnected net and drive the inverter pair
            net_inv.get_input(0).connect(disconnected);
        }

        Ok(format!("Inserted {} pairs of inverters", n))
    }
}

/// A pass that remaps cells according to some arbitrary cell mapping function.
pub struct RemapCells<I: Instantiable> {
    map: Box<dyn Fn(&I) -> Option<I>>,
}

impl<I: Instantiable> fmt::Display for RemapCells<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RemapCells")
    }
}

impl<I: Instantiable> fmt::Debug for RemapCells<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RemapCells")
    }
}

impl<I: Instantiable> Default for RemapCells<I> {
    fn default() -> Self {
        Self {
            map: Box::new(|_| None),
        }
    }
}

impl<I: Instantiable> Pass for RemapCells<I> {
    type I = I;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        let mut remapped = 0;
        for node in netlist.objects() {
            if let Some(mut inst_type) = node.get_instance_type_mut() {
                let Some(remap) = (self.map)(&inst_type) else {
                    continue;
                };
                *inst_type = remap;
                remapped += 1;
            }
        }
        Ok(format!("Remapped {} cells", remapped))
    }
}

impl<I: Instantiable> RemapCells<I> {
    /// Create a new pass for remapping cells with a boxed function.
    pub fn new_boxed(map: Box<dyn Fn(&I) -> Option<I>>) -> Self {
        Self { map }
    }

    /// Create a new pass for remapping cells.
    pub fn new<F: Fn(&I) -> Option<I> + 'static>(map: F) -> Self {
        Self { map: Box::new(map) }
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
    /// Insert a pair of inverters at every point in the netlist.
    InsertInv,
    /// A dummy pass that emits the Verilog of the netlist.
    PrintVerilog<Cell>,
    /// A pass that renames wires and instances sequentially __0__, __1__, ...
    RenameNets<Cell>,
);
