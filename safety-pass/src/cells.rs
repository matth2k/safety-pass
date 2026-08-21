/*!

  A basic library of logic cells

*/

use safety_net::{
    DrivenNet, Identifier, Instantiable, Logic, Net, NetRef, Netlist, Parameter, format_id,
};
use std::{collections::HashMap, fmt, rc::Rc, str::FromStr};

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
    BUF,
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
    HA,
    FA,
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
            Self::NOT | Self::INV | Self::LUT1 | Self::BUF => 1,
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
            Self::HA => 2,
            Self::FA => 3,
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
            Self::INV | Self::NOT | Self::BUF => vec!["A".into()],
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
            Self::HA => vec!["A".into(), "B".into()],
            Self::FA => vec!["A".into(), "B".into(), "CI".into()],
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
            Self::MUX2 | Self::XOR2 | Self::BUF => vec!["Z".into()],
            Self::FA | Self::HA => vec!["CO".into(), "S".into()],
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

    /// Returns true if cell is an AND gate
    pub fn is_and(&self) -> bool {
        matches!(self, Self::AND | Self::AND2 | Self::AND3 | Self::AND4)
    }

    /// Returns true if cell is an OR gate
    pub fn is_or(&self) -> bool {
        matches!(self, Self::OR | Self::OR2 | Self::OR3 | Self::OR4)
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
            Self::BUF => Some(0.798),
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
            Self::HA => Some(2.66),
            Self::FA => Some(4.256),
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
            "BUF" => Ok(Self::BUF),
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
            "HA" => Ok(Self::HA),
            "FA" => Ok(Self::FA),
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
    size: Option<usize>,
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
            size,
        }
    }

    /// Get the cell type
    pub fn get_type(&self) -> CellType {
        self.ptype
    }

    /// Return a new cell with the same size
    pub fn new_like(&self, ctype: CellType) -> Self {
        Self::new(ctype, self.size)
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

    /// Returns the area of the cell if the cell type has a known area and size
    pub fn get_area(&self) -> Option<f32> {
        if let Some(min) = self.get_type().get_min_area()
            && let Some(size) = self.size
        {
            Some(min * size as f32)
        } else {
            None
        }
    }
}

impl Instantiable for Cell {
    fn get_name(&self) -> &Identifier {
        &self.name
    }

    fn get_input_ports(&self) -> &[Net] {
        &self.inputs
    }

    fn get_output_ports(&self) -> &[Net] {
        &self.outputs
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

    fn clear_parameter(&mut self, id: &Identifier) -> Option<Parameter> {
        self.params.remove(id)
    }

    fn parameters(&self) -> Vec<(Identifier, Parameter)> {
        self.params.clone().into_iter().collect()
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

/// A uniquified netlist instantiated as a module
#[derive(Debug)]
pub struct ModInst<I: Instantiable> {
    name: Identifier,
    inputs: Vec<Net>,
    outputs: Vec<Net>,
    seq: bool,
    netlist: Rc<Netlist<I>>,
}

impl<I: Instantiable> ModInst<I> {
    /// Uniquify a netlist that can be instantiated.
    /// **This deep clones the netlist**
    pub fn new(netlist: &Netlist<I>) -> Self {
        let name = netlist.get_name().clone();
        let inputs = netlist.get_input_ports().collect();
        let outputs = netlist.get_output_ports();
        let seq = netlist
            .objects()
            .any(|obj| obj.get_instance_type().is_some_and(|i| i.is_seq()));
        Self {
            name,
            inputs,
            outputs,
            seq,
            netlist: netlist.deep_clone(),
        }
    }

    /// Unwraps the instantiable into the underlying netlist.
    pub fn unwrap(self) -> Rc<Netlist<I>> {
        self.netlist
    }

    /// Inlines `self`'s unique netlist into `netlist` using `drivers` as inputs
    /// On success, returns a vector of the outputs nets from the inlined netlist.
    pub fn inline_into<O: Instantiable + From<I>>(
        &self,
        netlist: &Rc<Netlist<O>>,
        prefix: Option<Identifier>,
        drivers: &[DrivenNet<O>],
    ) -> Result<Vec<DrivenNet<O>>, safety_net::Error> {
        if drivers.len() != self.inputs.len() {
            return Err(safety_net::Error::ArgumentMismatch(
                self.inputs.len(),
                drivers.len(),
            ));
        }

        let mut map = HashMap::new();
        for (k, v) in self.netlist.inputs().zip(drivers) {
            map.insert(k, v.clone());
        }

        let mut cells = Vec::new();
        for obj in self.netlist.objects() {
            if obj.is_an_input() {
                continue;
            }
            cells.push((netlist.clone_into(&obj, prefix.clone(), &mut map), obj));
        }

        for (v, k) in cells {
            for (inpk, inpv) in k.inputs().zip(v.inputs()) {
                if let Some(driver) = inpk.get_driver()
                    && let Some(remap) = map.get(&driver).cloned()
                {
                    inpv.connect(remap);
                }
            }
        }

        let mut outputs = Vec::new();
        for (output, _) in self.netlist.outputs() {
            outputs.push(map[&output].clone());
        }

        Ok(outputs)
    }
}

impl<I: Instantiable> Clone for ModInst<I> {
    fn clone(&self) -> Self {
        Self::new(&self.netlist)
    }
}

impl<I: Instantiable> Instantiable for ModInst<I> {
    fn get_name(&self) -> &Identifier {
        &self.name
    }

    fn get_input_ports(&self) -> &[Net] {
        &self.inputs
    }

    fn get_output_ports(&self) -> &[Net] {
        &self.outputs
    }

    fn has_parameter(&self, _id: &Identifier) -> bool {
        false
    }

    fn get_parameter(&self, _id: &Identifier) -> Option<Parameter> {
        None
    }

    fn set_parameter(&mut self, _id: &Identifier, _val: Parameter) -> Option<Parameter> {
        panic!("Cannot set parameter on a module instance");
    }

    fn clear_parameter(&mut self, _id: &Identifier) -> Option<Parameter> {
        None
    }

    fn parameters(&self) -> Vec<(Identifier, Parameter)> {
        Vec::new()
    }

    fn from_constant(_val: Logic) -> Option<Self> {
        None
    }

    fn get_constant(&self) -> Option<Logic> {
        None
    }

    fn is_seq(&self) -> bool {
        self.seq
    }

    fn verify(&self) -> Result<(), String> {
        if self.netlist.get_input_ports().count() != self.inputs.len() {
            return Err(format!(
                "Module instance {} has {} input ports, but netlist has {}",
                self.name,
                self.inputs.len(),
                self.netlist.inputs().count()
            ));
        }

        if self.netlist.get_output_ports().len() != self.outputs.len() {
            return Err(format!(
                "Module instance {} has {} output ports to drive, but netlist has {}",
                self.name,
                self.outputs.len(),
                self.netlist.outputs().len()
            ));
        }

        if self.name != *self.netlist.get_name() {
            return Err(format!(
                "Module instance {} has name {}, but netlist has name {}",
                self.name,
                self.name,
                self.netlist.get_name()
            ));
        }

        if self.seq
            != self
                .netlist
                .objects()
                .any(|obj| obj.get_instance_type().is_some_and(|i| i.is_seq()))
        {
            return Err(
                "Module/netlist mismatch on whether netlist is sequential or combinational"
                    .to_string(),
            );
        }

        for (a, b) in self.inputs.iter().zip(self.netlist.get_input_ports()) {
            if a.get_identifier() != b.get_identifier() {
                return Err(format!(
                    "Module instance {} input port has name {}, but netlist has name {}",
                    self.name,
                    a.get_identifier(),
                    b.get_identifier()
                ));
            }
        }

        for (a, b) in self.outputs.iter().zip(self.netlist.get_output_ports()) {
            if a.get_identifier() != b.get_identifier() {
                return Err(format!(
                    "Module instance {} output port has name {}, but netlist has name {}",
                    self.name,
                    a.get_identifier(),
                    b.get_identifier()
                ));
            }
        }

        self.netlist.verify().map_err(|e| e.to_string())
    }
}

/// An instance wrapper enum around primitive or other netlists
#[derive(Debug, Clone)]
pub enum ModOrCell<I: Instantiable> {
    /// An instantiation of a unique netlist
    ModInst(ModInst<ModOrCell<I>>),
    /// A primitive cell
    Cell(I),
}

impl<I: Instantiable> From<I> for ModOrCell<I>
where
    I: Instantiable,
{
    fn from(cell: I) -> Self {
        Self::Cell(cell)
    }
}

impl<I: Instantiable> From<&Netlist<ModOrCell<I>>> for ModOrCell<I> {
    fn from(netlist: &Netlist<ModOrCell<I>>) -> Self {
        Self::ModInst(ModInst::new(netlist))
    }
}

impl<I: Instantiable> Instantiable for ModOrCell<I> {
    fn get_name(&self) -> &Identifier {
        match self {
            Self::ModInst(m) => m.get_name(),
            Self::Cell(c) => c.get_name(),
        }
    }

    fn get_input_ports(&self) -> &[Net] {
        match self {
            Self::ModInst(m) => m.get_input_ports(),
            Self::Cell(c) => c.get_input_ports(),
        }
    }

    fn get_output_ports(&self) -> &[Net] {
        match self {
            Self::ModInst(m) => m.get_output_ports(),
            Self::Cell(c) => c.get_output_ports(),
        }
    }

    fn has_parameter(&self, id: &Identifier) -> bool {
        match self {
            Self::ModInst(m) => m.has_parameter(id),
            Self::Cell(c) => c.has_parameter(id),
        }
    }

    fn get_parameter(&self, id: &Identifier) -> Option<Parameter> {
        match self {
            Self::ModInst(m) => m.get_parameter(id),
            Self::Cell(c) => c.get_parameter(id),
        }
    }

    fn set_parameter(&mut self, id: &Identifier, val: Parameter) -> Option<Parameter> {
        match self {
            Self::ModInst(m) => m.set_parameter(id, val),
            Self::Cell(c) => c.set_parameter(id, val),
        }
    }

    fn clear_parameter(&mut self, id: &Identifier) -> Option<Parameter> {
        match self {
            Self::ModInst(m) => m.clear_parameter(id),
            Self::Cell(c) => c.clear_parameter(id),
        }
    }

    fn parameters(&self) -> Vec<(Identifier, Parameter)> {
        match self {
            Self::ModInst(m) => m.parameters(),
            Self::Cell(c) => c.parameters(),
        }
    }

    fn from_constant(val: Logic) -> Option<Self> {
        I::from_constant(val).map(Self::Cell)
    }

    fn get_constant(&self) -> Option<Logic> {
        match self {
            Self::ModInst(m) => m.get_constant(),
            Self::Cell(c) => c.get_constant(),
        }
    }

    fn is_seq(&self) -> bool {
        match self {
            Self::ModInst(m) => m.is_seq(),
            Self::Cell(c) => c.is_seq(),
        }
    }

    fn verify(&self) -> Result<(), String> {
        match self {
            Self::ModInst(m) => m.verify(),
            Self::Cell(c) => c.verify(),
        }
    }
}

/// Returns the underling primitive variant associated with this object
pub trait Primitive {
    /// Get the primitive cell type
    fn get_ptype(&self) -> Option<CellType>;
}

impl Primitive for NetRef<Cell> {
    fn get_ptype(&self) -> Option<CellType> {
        self.get_instance_type().map(|t| t.get_type())
    }
}

impl Primitive for DrivenNet<Cell> {
    fn get_ptype(&self) -> Option<CellType> {
        self.get_instance_type().map(|t| t.get_type())
    }
}

impl Primitive for NetRef<ModOrCell<Cell>> {
    fn get_ptype(&self) -> Option<CellType> {
        self.get_instance_type().and_then(|t| match &*t {
            ModOrCell::ModInst(_) => None,
            ModOrCell::Cell(c) => Some(c.get_type()),
        })
    }
}

impl Primitive for DrivenNet<ModOrCell<Cell>> {
    fn get_ptype(&self) -> Option<CellType> {
        self.get_instance_type().and_then(|t| match &*t {
            ModOrCell::ModInst(_) => None,
            ModOrCell::Cell(c) => Some(c.get_type()),
        })
    }
}

#[cfg(feature = "id")]
impl nl_compiler::FromId for Cell {
    fn from_id(s: &Identifier) -> Result<Self, safety_net::Error> {
        CellType::from_str(&s.to_string()).map(|ctype| Cell::new(ctype, None))
    }
}

#[cfg(feature = "id")]
impl nl_compiler::FromId for ModOrCell<Cell> {
    fn from_id(s: &Identifier) -> Result<Self, safety_net::Error> {
        CellType::from_str(&s.to_string()).map(|ctype| ModOrCell::Cell(Cell::new(ctype, None)))
    }
}
