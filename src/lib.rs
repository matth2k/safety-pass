#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
/*!

Compiler pass infrastructure for safety-net

*/

use log::info;
use safety_net::{
    DrivenNet, Identifier, Instantiable, Logic, Net, NetRef, Netlist, Parameter, format_id,
};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    rc::Rc,
    str::FromStr,
};
use thiserror::Error;

/// A logic cell type
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    AND,
    NAND,
    OR,
    NOR,
    XOR,
    XNOR,
    NOT,
    INV,
    AND2,
    NAND2,
    OR2,
    NOR2,
    XOR2,
    XNOR2,
    AND3,
    NAND3,
    OR3,
    NOR3,
    AND4,
    NAND4,
    OR4,
    NOR4,
    MUX,
    MUX2,
    MUXF7,
    MUXF8,
    MUXF9,
    AOI21,
    OAI21,
    AOI211,
    AOI22,
    OAI211,
    OAI22,
    OAI221,
    AOI221,
    OAI222,
    AOI222,
    LUT1,
    LUT2,
    LUT3,
    LUT4,
    LUT5,
    LUT6,
    VCC,
    GND,
    FDRE,
    FDSE,
    FDPE,
    FDCE,
    MAJ3,
}

impl CellType {
    /// Return the number of inputs
    pub fn get_num_inputs(&self) -> usize {
        match self {
            Self::AND2 | Self::AND => 2,
            Self::NAND2 | Self::NAND => 2,
            Self::OR2 | Self::OR => 2,
            Self::NOR2 | Self::NOR => 2,
            Self::XOR2 | Self::XOR => 2,
            Self::XNOR2 | Self::XNOR => 2,
            Self::NOT | Self::INV | Self::LUT1 => 1,
            Self::MUX | Self::MUX2 | Self::MUXF7 | Self::MUXF8 | Self::MUXF9 => 3,
            Self::AND3 | Self::NAND3 | Self::OR3 | Self::NOR3 => 3,
            Self::AND4 | Self::NAND4 | Self::OR4 | Self::NOR4 => 4,
            Self::AOI21 | Self::OAI21 => 3,
            Self::AOI211 | Self::AOI22 | Self::OAI211 | Self::OAI22 => 4,
            Self::AOI221 | Self::OAI221 => 5,
            Self::AOI222 | Self::OAI222 => 6,
            Self::LUT2 => 2,
            Self::LUT3 => 3,
            Self::LUT4 => 4,
            Self::LUT5 => 5,
            Self::LUT6 => 6,
            Self::VCC | Self::GND => 0,
            Self::FDRE | Self::FDSE | Self::FDPE | Self::FDCE => 4,
            Self::MAJ3 => 3,
        }
    }

    /// Get the list of input ports for this cell type
    pub fn get_input_ports(&self) -> Vec<Identifier> {
        match self {
            Self::AND
            | Self::NAND
            | Self::OR
            | Self::NOR
            | Self::XOR
            | Self::XNOR
            | Self::XOR2
            | Self::XNOR2 => {
                vec!["A".into(), "B".into()]
            }
            Self::INV | Self::NOT => vec!["A".into()],
            Self::AND2 | Self::NAND2 | Self::OR2 | Self::NOR2 => {
                vec!["A1".into(), "A2".into()]
            }
            Self::AND3 | Self::NAND3 | Self::OR3 | Self::NOR3 | Self::MAJ3 => {
                vec!["A1".into(), "A2".into(), "A3".into()]
            }
            Self::AND4 | Self::NAND4 | Self::OR4 | Self::NOR4 => {
                vec!["A1".into(), "A2".into(), "A3".into(), "A4".into()]
            }
            Self::MUX => {
                vec!["S".into(), "A".into(), "B".into()]
            }
            Self::MUX2 => {
                vec!["S".into(), "B".into(), "A".into()]
            }
            Self::MUXF7 | Self::MUXF8 | Self::MUXF9 => {
                vec!["S".into(), "I1".into(), "I0".into()]
            }
            Self::AOI21 | Self::OAI21 => vec!["A".into(), "B1".into(), "B2".into()],
            Self::AOI22 | Self::OAI22 => vec!["A1".into(), "A2".into(), "B1".into(), "B2".into()],
            Self::AOI211 | Self::OAI211 => vec!["A".into(), "B".into(), "C1".into(), "C2".into()],
            Self::AOI221 | Self::OAI221 => vec![
                "A".into(),
                "B1".into(),
                "B2".into(),
                "C1".into(),
                "C2".into(),
            ],
            Self::AOI222 | Self::OAI222 => vec![
                "A1".into(),
                "A2".into(),
                "B1".into(),
                "B2".into(),
                "C1".into(),
                "C2".into(),
            ],
            Self::LUT1 => vec!["I0".into()],
            Self::LUT2 => vec!["I1".into(), "I0".into()],
            Self::LUT3 => vec!["I2".into(), "I1".into(), "I0".into()],
            Self::LUT4 => vec!["I3".into(), "I2".into(), "I1".into(), "I0".into()],
            Self::LUT5 => vec![
                "I4".into(),
                "I3".into(),
                "I2".into(),
                "I1".into(),
                "I0".into(),
            ],
            Self::LUT6 => vec![
                "I5".into(),
                "I4".into(),
                "I3".into(),
                "I2".into(),
                "I1".into(),
                "I0".into(),
            ],
            Self::VCC | Self::GND => vec![],
            Self::FDRE => vec!["D".into(), "C".into(), "CE".into(), "R".into()],
            Self::FDSE => vec!["D".into(), "C".into(), "CE".into(), "S".into()],
            Self::FDPE => vec!["D".into(), "C".into(), "CE".into(), "PRE".into()],
            Self::FDCE => vec!["D".into(), "C".into(), "CE".into(), "CLR".into()],
        }
    }

    /// Get the name of the output ports for this cell type
    pub fn get_output_ports(&self) -> Vec<Identifier> {
        match self {
            Self::AND
            | Self::NAND
            | Self::OR
            | Self::NOR
            | Self::XOR
            | Self::XNOR
            | Self::NOT
            | Self::MUX => vec!["Y".into()],
            Self::LUT1
            | Self::LUT2
            | Self::LUT3
            | Self::LUT4
            | Self::LUT5
            | Self::LUT6
            | Self::MUXF7
            | Self::MUXF8
            | Self::MUXF9 => vec!["O".into()],
            Self::VCC => vec!["P".into()],
            Self::GND => vec!["G".into()],
            Self::FDRE | Self::FDSE | Self::FDPE | Self::FDCE => vec!["Q".into()],
            Self::MUX2 | Self::XOR2 => vec!["Z".into()],
            _ => vec!["ZN".into()],
        }
    }

    /// Returns true if the cell is a k-LUT
    pub fn is_lut(&self) -> bool {
        matches!(
            self,
            Self::LUT1 | Self::LUT2 | Self::LUT3 | Self::LUT4 | Self::LUT5 | Self::LUT6
        )
    }

    /// Returns true if the cell is a constant logical value
    pub fn is_const(&self) -> bool {
        matches!(self, Self::VCC | Self::GND)
    }

    /// Returns true if the cell is not a LUT, reg, or constant
    pub fn is_gate(&self) -> bool {
        !self.is_lut() && !self.is_reg() && !self.is_const()
    }

    /// Returns true if the cell is a register (FDRE, FDSE, FDPE, FDCE)
    pub fn is_reg(&self) -> bool {
        matches!(self, Self::FDRE | Self::FDSE | Self::FDPE | Self::FDCE)
    }

    /// Get the area of a minimum sized instance of the cell type
    pub fn get_min_area(&self) -> Option<f32> {
        match self {
            Self::AND2 => Some(1.064),
            Self::AND3 => Some(1.33),
            Self::AND4 => Some(1.596),
            Self::AOI21 => Some(1.064),
            Self::AOI22 => Some(1.33),
            Self::AOI211 => Some(1.33),
            Self::AOI221 => Some(1.596),
            Self::AOI222 => Some(2.128),
            Self::INV => Some(0.532),
            Self::MUX2 => Some(1.862),
            Self::NAND2 => Some(0.798),
            Self::NAND3 => Some(1.064),
            Self::NAND4 => Some(1.33),
            Self::NOR2 => Some(0.798),
            Self::NOR3 => Some(1.064),
            Self::NOR4 => Some(1.33),
            Self::OAI21 => Some(1.064),
            Self::OAI22 => Some(1.33),
            Self::OAI211 => Some(1.33),
            Self::OAI221 => Some(1.596),
            Self::OAI222 => Some(2.128),
            Self::OR2 => Some(1.064),
            Self::OR3 => Some(1.33),
            Self::OR4 => Some(1.596),
            Self::XNOR2 => Some(1.596),
            Self::XOR2 => Some(1.596),
            Self::MAJ3 => Some(1.064),
            _ => None,
        }
    }
}

impl FromStr for CellType {
    type Err = safety_net::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pre = match s.split_once("_X") {
            Some((p, _)) => p,
            None => s,
        };

        match pre {
            "INV" => Ok(Self::INV),
            "MUX" => Ok(Self::MUX),
            "AND2" => Ok(Self::AND2),
            "NAND2" => Ok(Self::NAND2),
            "OR2" => Ok(Self::OR2),
            "NOR2" => Ok(Self::NOR2),
            "XOR2" => Ok(Self::XOR2),
            "XNOR2" => Ok(Self::XNOR2),
            "AND3" => Ok(Self::AND3),
            "NAND3" => Ok(Self::NAND3),
            "OR3" => Ok(Self::OR3),
            "NOR3" => Ok(Self::NOR3),
            "AND4" => Ok(Self::AND4),
            "NAND4" => Ok(Self::NAND4),
            "OR4" => Ok(Self::OR4),
            "NOR4" => Ok(Self::NOR4),
            "AOI21" => Ok(Self::AOI21),
            "OAI21" => Ok(Self::OAI21),
            "AOI211" => Ok(Self::AOI211),
            "AOI22" => Ok(Self::AOI22),
            "OAI211" => Ok(Self::OAI211),
            "OAI22" => Ok(Self::OAI22),
            "AOI221" => Ok(Self::AOI221),
            "OAI221" => Ok(Self::OAI221),
            "AOI222" => Ok(Self::AOI222),
            "OAI222" => Ok(Self::OAI222),
            "MUX2" => Ok(Self::MUX2),
            "AND" => Ok(Self::AND),
            "NAND" => Ok(Self::NAND),
            "OR" => Ok(Self::OR),
            "NOR" => Ok(Self::NOR),
            "XOR" => Ok(Self::XOR),
            "XNOR" => Ok(Self::XNOR),
            "NOT" => Ok(Self::NOT),
            "MUXF7" => Ok(Self::MUXF7),
            "MUXF8" => Ok(Self::MUXF8),
            "MUXF9" => Ok(Self::MUXF9),
            "LUT1" => Ok(Self::LUT1),
            "LUT2" => Ok(Self::LUT2),
            "LUT3" => Ok(Self::LUT3),
            "LUT4" => Ok(Self::LUT4),
            "LUT5" => Ok(Self::LUT5),
            "LUT6" => Ok(Self::LUT6),
            "VCC" => Ok(Self::VCC),
            "GND" => Ok(Self::GND),
            "FDRE" => Ok(Self::FDRE),
            "FDSE" => Ok(Self::FDSE),
            "FDPE" => Ok(Self::FDPE),
            "FDCE" => Ok(Self::FDCE),
            "MAJ3" => Ok(Self::MAJ3),
            _ => Err(safety_net::Error::ParseError(format!(
                "Unknown cell type: {s}"
            ))),
        }
    }
}

impl fmt::Display for CellType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// An instantiable cell in some [CellType]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    name: Identifier,
    ptype: CellType,
    inputs: Vec<Net>,
    outputs: Vec<Net>,
    params: HashMap<Identifier, Parameter>,
}

impl Cell {
    /// Create a new primitive cell
    pub fn new(ptype: CellType, size: Option<usize>) -> Self {
        Self {
            name: if let Some(s) = size {
                format_id!("{}_X{}", ptype, s)
            } else {
                format_id!("{}", ptype)
            },
            ptype,
            inputs: ptype
                .get_input_ports()
                .into_iter()
                .map(Net::new_logic)
                .collect(),
            outputs: ptype
                .get_output_ports()
                .into_iter()
                .map(Net::new_logic)
                .collect(),
            params: HashMap::new(),
        }
    }

    /// Get the cell type
    pub fn get_type(&self) -> CellType {
        self.ptype
    }

    /// Remap the ith input port to a new net name
    pub fn remap_input(mut self, ind: usize, name: Identifier) -> Self {
        let net = &mut self.inputs[ind];
        net.set_identifier(name);
        self
    }

    /// Remap the ith output port to a new net name
    pub fn remap_output(mut self, ind: usize, name: Identifier) -> Self {
        let net = &mut self.outputs[ind];
        net.set_identifier(name);
        self
    }
}

impl Instantiable for Cell {
    fn get_name(&self) -> &Identifier {
        &self.name
    }

    fn get_input_ports(&self) -> impl IntoIterator<Item = &Net> {
        self.inputs.iter()
    }

    fn get_output_ports(&self) -> impl IntoIterator<Item = &Net> {
        self.outputs.iter()
    }

    fn has_parameter(&self, id: &Identifier) -> bool {
        self.params.contains_key(id)
    }

    fn get_parameter(&self, id: &Identifier) -> Option<Parameter> {
        self.params.get(id).cloned()
    }

    fn set_parameter(&mut self, id: &Identifier, val: Parameter) -> Option<Parameter> {
        self.params.insert(id.clone(), val)
    }

    fn parameters(&self) -> impl Iterator<Item = (Identifier, Parameter)> {
        self.params.clone().into_iter()
    }

    fn from_constant(val: Logic) -> Option<Self> {
        match val {
            Logic::False => Some(Cell::new(CellType::GND, None)),
            Logic::True => Some(Cell::new(CellType::VCC, None)),
            _ => None,
        }
    }

    fn get_constant(&self) -> Option<Logic> {
        match self.ptype {
            CellType::GND => Some(Logic::False),
            CellType::VCC => Some(Logic::True),
            _ => None,
        }
    }

    fn is_seq(&self) -> bool {
        self.ptype.is_reg()
    }
}

impl nl_compiler::FromId for Cell {
    fn from_id(s: &Identifier) -> Result<Self, safety_net::Error> {
        CellType::from_str(&s.to_string()).map(|ctype| Cell::new(ctype, None))
    }
}

/// Errors for running passes
#[derive(Error, Debug)]
pub enum Error {
    /// An netlist error in running the pass.
    #[error("Pass error: {0}")]
    PassError(#[from] safety_net::Error),
    /// An I/O error in writing the pass output.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// A pass on a netlist.
pub trait Pass: fmt::Display {
    /// The type of Instantiable in the netlist
    type I: Instantiable;

    /// Run the pass on the given netlist and return any info as a string.
    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error>;

    /// Run the pass with verification before.
    fn run_verified(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
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

    /// Run the pipeline on a netlist. If `verify` is true, verify the netlist after each pass.
    pub fn run(&self, netlist: &Rc<Netlist<I>>, verify: bool) -> Result<String, Error> {
        let mut res = String::new();
        let n = self.passes.len();
        for (i, pass) in self.passes.iter().enumerate() {
            info!("Running pass {i} ({pass})...");
            match pass.run(netlist) {
                Ok(output) => {
                    res = output;
                    if i != n - 1 {
                        if verify {
                            netlist.verify()?;
                        }
                        info!("{pass}: {}", res);
                    }
                }
                Err(e) => return Err(e),
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
pub trait Pattern {
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
    ) -> Result<bool, Error>;
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

    /// Apply the patterns in one iteration.
    fn fold(&self, netlist: &Rc<Netlist<I>>) -> Result<usize, Error> {
        let mut cleaned: HashSet<NetRef<I>> = HashSet::new();
        let mut i = 0;
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
                        if pattern.apply(&cell, &cell_type, &create, &mut replace)? {
                            change = true;
                            break 'iter;
                        }
                    }
                }
            }

            if !change {
                break;
            }

            for (a, b) in replacements {
                let a = netlist.replace_net_uses(a, &b)?;
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

impl<C: Instantiable> Pass for Folder<C> {
    type I = C;

    fn run(&self, netlist: &Rc<Netlist<Self::I>>) -> Result<String, Error> {
        let iters = self.fold(netlist)?;
        Ok(format!(
            "Folded {} pattterns over {} iterations",
            self.patterns.len(),
            iters
        ))
    }
}

pub mod passes;
pub mod patterns;
