/*!

  A basic library of logic cells

*/

use safety_net::{Identifier, Instantiable, Logic, Net, Parameter, format_id};
use std::{collections::HashMap, fmt, str::FromStr};
use crate::logic_eqn::LogicEqn;
use crate::VerilogLib;



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

fn or2(eqn: &mut LogicEqn, a: usize, b: usize) -> usize {
    let not_a = eqn.inv(a);
    let not_b = eqn.inv(b);
    let and_ab = eqn.and(not_a, not_b);
    return eqn.inv(and_ab);
}

fn and_n(eqn: &mut LogicEqn, ins: &Vec<usize>) -> usize {
    let mut result = ins[0];
    let mut i = 1;
    while i < ins.len() {
        result = eqn.and(result, ins[i]);
        i = i + 1;
    }
    return result;
}

fn or_n(eqn: &mut LogicEqn, ins: &Vec<usize>) -> usize {
    let mut result = ins[0];
    let mut i = 1;
    while i < ins.len() {
        result = or2(eqn, result, ins[i]);
        i = i + 1;
    }
    return result;

}

fn nand_n(eqn: &mut LogicEqn, ins: &Vec<usize>) -> usize {
    let and_result = and_n(eqn, ins);
    return eqn.inv(and_result);
}

fn nor_n(eqn: &mut LogicEqn, ins: &Vec<usize>) -> usize {
    let and_result = or_n(eqn, ins);
    return eqn.inv(and_result);
}

fn xor2(eqn: &mut LogicEqn, a: usize, b: usize) -> usize {
    let not_a = eqn.inv(a);
    let not_b = eqn.inv(b);
    let and1_result  = eqn.and(a, not_b);
    let and2_result  = eqn.and(not_a, b);
    return or2(eqn, and1_result, and2_result);
}

fn xnor2(eqn: &mut LogicEqn, a: usize, b: usize) -> usize {
    let xor_result = xor2(eqn, a, b);
    return eqn.inv(xor_result);

}

fn mux2(eqn: &mut LogicEqn, s: usize, a: usize, b: usize) -> usize {
    let not_s = eqn.inv(s);
    let or_1 = eqn.and(not_s, a);
    let and_1 = eqn.and(s, b);
    return or2(eqn, or_1, and_1);
}

impl CellType {
    /// Builds the combiniational logic of this cell as a LogicEqn
    pub fn get_logic_eqn(&self) -> LogicEqn {
        let mut eqn = LogicEqn::new();

        // make an Input node for every port this cell type has
        let ports = self.get_input_ports();
        let mut ins: Vec<usize> = Vec::new();
        let mut i = 0;
        while i < ports.len() {
            let name = ports[i].to_string();
            let idx = eqn.input(&name);
            ins.push(idx);
            i = i + 1;
        }

        match self {
            Self::AND | Self::AND2 | Self::AND3 | Self::AND4 => {
                and_n(&mut eqn, &ins);
            }
            Self::NAND | Self::NAND2 | Self::NAND3 | Self::NAND4 => {
                nand_n(&mut eqn, &ins);
            }
            Self::OR | Self::OR2 | Self::OR3 | Self::OR4 => {
                or_n(&mut eqn, &ins);
            }
            Self::NOR | Self::NOR2 | Self::NOR3 | Self::NOR4 => {
                nor_n(&mut eqn, &ins);
            }
            Self::XOR | Self::XOR2 => {
                xor2(&mut eqn, ins[0], ins[1]);
            }
            Self::XNOR | Self::XNOR2 => {
                xnor2(&mut eqn, ins[0], ins[1]);
            }
            Self::NOT | Self::INV => {
                eqn.inv(ins[0]);
            }
            Self::MAJ3 => {
                let ab_and = eqn.and(ins[0], ins[1]);
                let ac_and = eqn.and(ins[0], ins[2]);
                let bc_and = eqn.and(ins[1], ins[2]);
                let or_1 = or2(&mut eqn, ab_and, ac_and);
                or2(&mut eqn, or_1, bc_and);
            }
            Self::MUX => {
                mux2(&mut eqn, ins[0], ins[1], ins[2]);
            }
            Self::MUX2 => {
                mux2(&mut eqn, ins[0], ins[2], ins[1]);
            }
            Self::MUXF7 | Self::MUXF8 | Self::MUXF9 => {
                mux2(&mut eqn, ins[0], ins[2], ins[1]);
            }
            Self::AOI21 => {
                panic!("TODO- not entirely sure");
            }
            Self::OAI21 => {
                panic!("TODO");
            }
            Self::AOI22 => {
                panic!("TODO");
            }
            Self::OAI22 => {
                panic!("TODO"); 
            }
            Self::AOI211 => {
                panic!("TODO");
            }
            Self::OAI211 => {
                panic!("TODO");
            }
            Self::AOI221 => {
                panic!("TODO");
            }
            Self::OAI221 => {
                panic!("TODO");
            }
            Self::AOI222 => {
                panic!("TODO");
            }
            Self::OAI222 => {
                panic!("TODO");
            }
            Self::LUT1 | Self::LUT2 | Self::LUT3 | Self::LUT4 | Self::LUT5 | Self::LUT6 => {
                panic!("LUT function cannot be represented by a LogicEqn");
            }
            Self::VCC | Self::GND => {
                panic!("constants cannot be represented by a LogicEqn");
            }
            Self::FDRE | Self::FDSE | Self::FDPE | Self::FDCE => {
                panic!("flip-flops cannot be represented by a LogicEqn");
            }
        }

        return eqn;
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

#[cfg(feature = "id")]
impl nl_compiler::FromId for Cell {
    fn from_id(s: &Identifier) -> Result<Self, safety_net::Error> {
        CellType::from_str(&s.to_string()).map(|ctype| Cell::new(ctype, None))
    }
}

impl VerilogLib for Cell {
    fn verilog_library() -> String {
        // Explictly put all celltypes into a list so we can make verilog modules of them.
        let all_cell_types = vec![
            CellType::AND, CellType::NAND, CellType::OR, CellType::NOR,
            CellType::XOR, CellType::XNOR, CellType::NOT, CellType::INV,
            CellType::AND2, CellType::NAND2, CellType::OR2, CellType::NOR2,
            CellType::XOR2, CellType::XNOR2,
            CellType::AND3, CellType::NAND3, CellType::OR3, CellType::NOR3,
            CellType::AND4, CellType::NAND4, CellType::OR4, CellType::NOR4,
            CellType::MUX, CellType::MUX2, CellType::MUXF7, CellType::MUXF8, CellType::MUXF9, CellType::MAJ3,
            // CellType::AOI21, CellType::OAI21, CellType::AOI211, CellType::AOI22,
            // CellType::OAI211, CellType::OAI22, CellType::OAI221, CellType::AOI221,
            // CellType::OAI222, CellType::AOI222, 
        ];

        let mut output = String::new(); // this will be the finished verilog file
        let mut i = 0;

        while i < all_cell_types.len() { // Iterate through all the cells
            let cell_type = all_cell_types[i];
            let eqn = cell_type.get_logic_eqn(); // Get the logic equation of the current cell
            let output_ports = cell_type.get_output_ports(); // Get the name(s) of the output port(s)
            
            
            // module header name; module AND
            output.push_str("module ");
            output.push_str(&cell_type.to_string());
            output.push_str("(\n");

            // input ports names; input A, ..
            let input_names = eqn.input_names();
            let mut j = 0;
            while j < input_names.len() {
                output.push_str("    input ");
                output.push_str(&input_names[j]);
                output.push_str(",\n");
                j = j + 1;
            }

            // output port; output Y
            let out_port_name = output_ports[0].to_string();
            output.push_str("    output ");
            output.push_str(&out_port_name);
            output.push_str("\n);\n");

            // Now we have something like
            // module AND(
            //     input A,
            //     input B,
            //     output Y
            // );

            

            // Use the previously defined LogicEqn display
            output.push_str(&eqn.to_string()); 

            // wire the last node to the module output; assign Y = nX
            if let Some(last_index) = eqn.output() {
                output.push_str("assign ");
                output.push_str(&out_port_name);
                output.push_str(" = n");
                output.push_str(&last_index.to_string());
                output.push_str(";\n");
            }

            output.push_str("endmodule\n\n");

            i = i + 1;
        }

        return output;
    }
}
