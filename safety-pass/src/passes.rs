/*!

  Simple netlist passes.

*/

use crate::{Cell, Pass};
use safety_net::{
    Error, FanOutTable, Identifier, Instantiable, Netlist, format_id, rewriter::NetMapper,
};
use std::collections::HashMap;
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
        use safety_net::emitter::VerilogEmitter;
        let emitter = VerilogEmitter::new_default(netlist);
        Ok(emitter.to_string())
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
        Ok(netlist.dot_string())
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

/// Prints stats on all the cell types in the netlist.
pub struct CellStats<I: Instantiable>(pub std::marker::PhantomData<I>);

impl<I: Instantiable> fmt::Display for CellStats<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CellStats")
    }
}

impl<I: Instantiable> fmt::Debug for CellStats<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CellStats")
    }
}

impl<I: Instantiable> Pass for CellStats<I> {
    type I = I;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        let mut map: HashMap<Identifier, usize> = HashMap::new();
        for node in netlist.objects() {
            if let Some(inst_type) = node.get_instance_type() {
                *map.entry(inst_type.get_name().clone()).or_insert(0) += 1;
            }
        }
        let mut pairs: Vec<(Identifier, usize)> = map.into_iter().collect();
        pairs.sort_by_key(|a| a.1);

        let mut res = String::new();
        let mut total = 0;
        for (cell, count) in pairs.into_iter().rev() {
            res += &format!("\t{}:\t{}\n", cell, count);
            total += count;
        }
        res += &format!("\n\tTotal:\t{}\n", total);
        Ok(format!("Cell Usage:\n{res}"))
    }
}

/// List all nets in the netlist
pub struct ListNets<I: Instantiable>(pub std::marker::PhantomData<I>);

impl<I: Instantiable> fmt::Display for ListNets<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ListNets")
    }
}

impl<I: Instantiable> fmt::Debug for ListNets<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ListNets")
    }
}

impl<I: Instantiable> Pass for ListNets<I> {
    type I = I;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        let mut buf = "Name\tFanin\tFanout\tDrivers\tUsers\n".to_string();
        let table = netlist.get_analysis::<FanOutTable<_>>()?;
        for node in netlist.objects() {
            let fi = node
                .inputs()
                .filter_map(|i| i.get_driver())
                .collect::<Vec<_>>();
            for output in node.outputs() {
                let fo = table.get_users(&output).collect::<Vec<_>>();
                buf += &format!(
                    "{}\t{}\t{}\t",
                    output.as_net().get_identifier(),
                    fi.len(),
                    fo.len()
                );
                for (i, driver) in fi.iter().enumerate() {
                    buf += &format!("{}", driver.as_net().get_identifier());
                    if i != fi.len() - 1 {
                        buf += ", ";
                    }
                }
                buf += "\t";
                let l = fo.len();
                for (i, user) in fo.into_iter().enumerate() {
                    let port_name = user.get_port().take_identifier();
                    buf += &format!(
                        "{}:{}",
                        user.unwrap().get_instance_name().unwrap(),
                        port_name
                    );
                    if i != l - 1 {
                        buf += ", ";
                    }
                }
                buf += "\n";
            }
        }
        Ok(buf)
    }
}

/// Strip all cell attributes except 'dont_touch` and `keep`
pub struct StripAttributes<I: Instantiable>(pub std::marker::PhantomData<I>);

impl<I: Instantiable> fmt::Display for StripAttributes<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StripAttributes")
    }
}

impl<I: Instantiable> fmt::Debug for StripAttributes<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StripAttributes")
    }
}

impl<I: Instantiable> Pass for StripAttributes<I> {
    type I = I;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        const ATTRS: [&str; 2] = ["dont_touch", "keep"];

        let mut i = 0;
        for obj in netlist.objects() {
            let mut modified = false;
            for a in obj.attributes() {
                let k = a.key();
                if !ATTRS.contains(&k.as_str()) {
                    obj.clear_attribute(k);
                    modified = true;
                }
            }
            if modified {
                i += 1;
            }
        }
        Ok(format!("Stripped attributes from {i} cells"))
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

        let n = everything.len();

        let mut mapper = NetMapper::new(netlist)?;

        // We use i to differentiate between nets that have the same base identifer.
        for (i, net) in everything.into_iter().enumerate() {
            // Combine the net's base name (n) and i to to create unique instance names
            // across both repeated runs of this pass and nets with identical base names.
            let inst_name = net.as_net().get_identifier().clone() + format_id!("_{i}_{n}");

            let inv_type = match net.get_instance_type() {
                Some(t) => t.new_like(CellType::INV),
                _ => Cell::new(CellType::INV, None),
            };

            let net_inv = netlist.insert_gate_disconnected(inv_type.clone(), inst_name.clone());

            // Repeat the pattern for the second inverter
            let inst_name = inst_name + "inv".into();
            let net_inv_inv =
                netlist.insert_gate(inv_type.clone(), inst_name, &[net_inv.clone().into()])?;

            // Replace the uses of the original net
            let replacement = net_inv_inv.get_output(0);
            let disconnected = mapper.replace(net, replacement);

            // Now take our disconnected net and drive the inverter pair
            net_inv.get_input(0).connect(disconnected);
        }

        mapper.apply()?;

        Ok(format!("Inserted {} pairs of inverters", n))
    }
}

/// Insert a scan chain along the FDREs
#[derive(Debug)]
pub struct InsertScanChain;

impl fmt::Display for InsertScanChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InsertScanChain")
    }
}

impl Pass for InsertScanChain {
    type I = Cell;
    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        use crate::CellType;

        fn mux() -> Cell {
            Cell::new(CellType::MUX, None)
        }

        let mut n = 0;

        let scan_en = netlist.insert_input("scan_en".into());
        let mut iter = netlist.matches(|i| i.get_type() == CellType::FDRE);
        let Some(prev) = iter.next() else {
            return Ok("No FDREs found, no scan chain inserted".into());
        };

        let mut prev = prev.get_output(0);

        for reg in iter {
            let input = reg.find_input(&"D".into()).unwrap();
            let Some(driver) = input.get_driver() else {
                continue;
            };

            let rmux = netlist.insert_gate(
                mux(),
                reg.get_instance_name().unwrap() + format_id!("scan_mux_{n}"),
                &[scan_en.clone(), prev, driver],
            )?;

            input.connect(rmux.into());
            prev = reg.into();

            n += 1;
        }

        prev.expose_with_name("scan_out".into());

        Ok(format!("Inserted a scan chain of length {}", n))
    }
}

/// Explicity inverts the clock of a cell that has a `IS_CLK_INVERTED` param
#[derive(Debug)]
pub struct ExtractInvClock;

impl fmt::Display for ExtractInvClock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExtractInvClock")
    }
}

impl Pass for ExtractInvClock {
    type I = Cell;
    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        use crate::CellType;
        use safety_net::Parameter;

        for cell in netlist.matches(|p| p.has_parameter(&"IS_CLK_INVERTED".into())) {
            let Some(port) = cell.find_input(&"C".into()) else {
                continue;
            };

            let Some(driver) = port.get_driver() else {
                continue;
            };

            let inverter = netlist.insert_gate(
                Cell::new(CellType::INV, None),
                cell.get_instance_name().unwrap()
                    + port.get_port().take_identifier()
                    + "inv".into(),
                &[driver],
            )?;

            port.connect(inverter.into());
            cell.get_instance_type_mut()
                .unwrap()
                .set_parameter(&"IS_CLK_INVERTED".into(), Parameter::from_bool(false));
        }

        Ok("Explicitly inverted all inverted pins".to_string())
    }
}

/// Returns `Some(I)` if the cell should be replaced with something else
type Remap<I> = dyn Fn(&I) -> Option<I> + 'static;

/// A pass that remaps cells according to some arbitrary cell mapping function.
pub struct RemapCells<I: Instantiable> {
    map: Box<Remap<I>>,
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
    pub fn new_boxed(map: Box<Remap<I>>) -> Self {
        Self { map }
    }

    /// Create a new pass for remapping cells.
    pub fn new<F: Fn(&I) -> Option<I> + 'static>(map: F) -> Self {
        Self { map: Box::new(map) }
    }
}

register_passes!(BasicPasses<Cell>;
    /// Prints stats on all the cell types in the netlist.
    CellStats<Cell>,
    /// A pass that cleans the netlist.
    Clean<Cell>,
    /// A pass that prints the dot graph of the netlist.
    #[cfg(feature = "graph")]
    DotGraph<Cell>,
    /// Explicity inverts the clock of a cell that has a `IS_CLK_INVERTED` param
    ExtractInvClock,
    /// A pass that runs all built-in patterns to a fixed point.
    FoldAllPatterns,
    /// Insert a pair of inverters at every point in the netlist.
    InsertInv,
    /// Insert a scan chain along the FDREs
    InsertScanChain,
    /// List all nets in the netlist
    ListNets<Cell>,
    /// A dummy pass that emits the Verilog of the netlist.
    PrintVerilog<Cell>,
    /// Renames nets/instances sequentially __0__, __1__, ...
    RenameNets<Cell>,
    /// Strip all cell attributes except 'dont_touch` and `keep`
    StripAttributes<Cell>,
);
