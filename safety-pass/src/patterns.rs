/*!

  Simple cell patterns.

*/

use crate::{Cell, CellType, Create, Pattern, Primitive, Replace};
use log::debug;
use safety_net::{DrivenNet, Error, Instantiable, NetRef};
use std::fmt;

/// A * A = A
#[derive(Debug)]
pub struct Idempotent;

impl fmt::Display for Idempotent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A * A = A")
    }
}

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

/// Folds monotone AND/OR gate trees (nested homogeneous gates) into a single equivalent (wider) gate.
/// The fold applies to all AND or all OR gates until final total fan-in = 4 (max-sized gate, i.e. AND4)
#[derive(Debug)]
pub struct MonotoneFold;

impl fmt::Display for MonotoneFold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AND2(AND2(a,b), c) => AND3(a,b,c)")
    }
}

impl MonotoneFold {
    /// Helper to [Self::apply] that checks homogeneity of fanin gates
    fn can_combine(ct: CellType, children: &[CellType], extra: usize) -> Option<CellType> {
        if !ct.is_and() && !ct.is_or() {
            return None;
        }
        let andg = ct.is_and();
        let mut fanin = extra;
        for child in children {
            if andg != child.is_and() {
                return None;
            }
            fanin += child.get_num_inputs();
        }
        match (andg, fanin) {
            (true, 2) => Some(CellType::AND2),
            (false, 2) => Some(CellType::OR2),
            (true, 3) => Some(CellType::AND3),
            (false, 3) => Some(CellType::OR3),
            (true, 4) => Some(CellType::AND4),
            (false, 4) => Some(CellType::OR4),
            _ => None,
        }
    }
}

/// Return type for [`search_inputs`]: `(const_false_port, const_true_port, other_driver)`.
/// `None` at the outer level means an input was undriven; inner `None` means no constant of
/// that polarity was found.
type ConstInputs = Option<(
    Option<DrivenNet<Cell>>,
    Option<DrivenNet<Cell>>,
    Option<DrivenNet<Cell>>,
)>;

/// Scans the inputs of a gate and classifies each as a constant-false port, constant-true port,
/// or non-constant driver. Returns `None` if any input is undriven.
/// Helper to [`AndAbsorb`], [`AndIdentity`], [`OrAbsorb`], [`OrIdentity`],
/// [`NandAbsorb`], [`NandIdentity`], [`NorAbsorb`], and [`NorIdentity`].
fn search_inputs(cell: &NetRef<Cell>) -> ConstInputs {
    use safety_net::Logic;
    let mut const_false: Option<DrivenNet<Cell>> = None;
    let mut const_true: Option<DrivenNet<Cell>> = None;
    let mut other: Option<DrivenNet<Cell>> = None;
    for port in cell.inputs() {
        let driver = port.get_driver()?;
        match driver.get_instance_type().and_then(|t| t.get_constant()) {
            Some(Logic::False) => {
                const_false = Some(driver);
            }
            Some(Logic::True) => {
                const_true = Some(driver);
            }
            _ => {
                other = Some(driver);
            }
        }
    }
    Some((const_false, const_true, other))
}

impl Pattern for MonotoneFold {
    type I = Cell;

    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, Error> {
        let root_type = cell_type.get_type();

        if !root_type.is_and() && !root_type.is_or() {
            return Ok(false);
        }

        let mut child_drivers: Vec<NetRef<Self::I>> = Vec::new();
        let mut child_types: Vec<CellType> = Vec::new();
        let mut non_child_drivers: Vec<DrivenNet<Self::I>> = Vec::new();

        for input in cell.inputs() {
            let Some(driver) = input.get_driver() else {
                return Ok(false);
            };
            match driver.get_ptype() {
                Some(ct)
                    if (ct.is_and() && root_type.is_and()) || (ct.is_or() && root_type.is_or()) =>
                {
                    child_drivers.push(driver.clone().unwrap());
                    child_types.push(ct);
                }
                _ => {
                    non_child_drivers.push(driver);
                }
            }
        }

        if child_drivers.is_empty() {
            return Ok(false);
        }

        let Some(new_type) = Self::can_combine(root_type, &child_types, non_child_drivers.len())
        else {
            return Ok(false);
        };

        let new_inst_name = cell.get_instance_name().unwrap() + "_folded".into();
        let new_gate = create(cell_type.new_like(new_type), new_inst_name);

        let mut port_idx = 0;
        for child_ref in &child_drivers {
            for input in child_ref.inputs() {
                if let Some(grandchild) = input.get_driver() {
                    new_gate.get_input(port_idx).connect(grandchild);
                }
                port_idx += 1;
            }
        }

        for driver in non_child_drivers {
            new_gate.get_input(port_idx).connect(driver);
            port_idx += 1;
        }

        let old_output = cell.get_output(0);
        let new_output = new_gate.get_output(0);

        debug!("MonotoneFold applied to cell {}!", old_output.as_net());
        replace(old_output, new_output)?;

        Ok(true)
    }
}

/// A AND 0 = 0 (absorbing element)
#[derive(Debug)]
pub struct AndAbsorb;
impl fmt::Display for AndAbsorb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A AND 0 = 0")
    }
}
impl Pattern for AndAbsorb {
    type I = Cell;
    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        _create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, Error> {
        let ct = cell_type.get_type();
        if !matches!(ct, CellType::AND | CellType::AND2) {
            return Ok(false);
        }
        let Some((Some(gnd_port), _, _)) = search_inputs(cell) else {
            return Ok(false);
        };
        let output = cell.get_output(0);
        debug!("AndAbsorb: AND absorb 0 on cell {}!", output.as_net());
        replace(output, gnd_port)?;
        Ok(true)
    }
}

/// A AND 1 = A (identity element)
#[derive(Debug)]
pub struct AndIdentity;
impl fmt::Display for AndIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A AND 1 = A")
    }
}
impl Pattern for AndIdentity {
    type I = Cell;
    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        _create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, Error> {
        let ct = cell_type.get_type();
        if !matches!(ct, CellType::AND | CellType::AND2) {
            return Ok(false);
        }
        let Some((_, Some(_), Some(other_driver))) = search_inputs(cell) else {
            return Ok(false);
        };
        let output = cell.get_output(0);
        debug!("AndIdentity: AND identity 1 on cell {}!", output.as_net());
        replace(output, other_driver)?;
        Ok(true)
    }
}

/// A OR 1 = 1 (absorbing element)
#[derive(Debug)]
pub struct OrAbsorb;
impl fmt::Display for OrAbsorb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A OR 1 = 1")
    }
}
impl Pattern for OrAbsorb {
    type I = Cell;
    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        _create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, Error> {
        let ct = cell_type.get_type();
        if !matches!(ct, CellType::OR | CellType::OR2) {
            return Ok(false);
        }
        let Some((_, Some(vcc_port), _)) = search_inputs(cell) else {
            return Ok(false);
        };
        let output = cell.get_output(0);
        debug!("OrAbsorb: OR absorb 1 on cell {}!", output.as_net());
        replace(output, vcc_port)?;
        Ok(true)
    }
}

/// A OR 0 = A (identity element)
#[derive(Debug)]
pub struct OrIdentity;
impl fmt::Display for OrIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A OR 0 = A")
    }
}
impl Pattern for OrIdentity {
    type I = Cell;
    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        _create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, Error> {
        let ct = cell_type.get_type();
        if !matches!(ct, CellType::OR | CellType::OR2) {
            return Ok(false);
        }
        let Some((Some(_), _, Some(other_driver))) = search_inputs(cell) else {
            return Ok(false);
        };
        let output = cell.get_output(0);
        debug!("OrIdentity: OR identity 0 on cell {}!", output.as_net());
        replace(output, other_driver)?;
        Ok(true)
    }
}

/// NOT(NOT(A)) = A, INV(INV(A)) = A
#[derive(Debug)]
pub struct DoubleNegation;

impl fmt::Display for DoubleNegation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NOT(NOT(A)) = A")
    }
}

impl Pattern for DoubleNegation {
    type I = Cell;

    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        _create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, Error> {
        let cell_type = cell_type.get_type();
        if !matches!(cell_type, CellType::NOT | CellType::INV) {
            return Ok(false);
        }

        let Some(driver) = cell.get_input(0).get_driver() else {
            return Ok(false);
        };

        let Some(cell_type) = driver.get_ptype() else {
            return Ok(false);
        };

        if !matches!(cell_type, CellType::NOT | CellType::INV) {
            return Ok(false);
        }

        let Some(inner_input) = driver.unwrap().get_input(0).get_driver() else {
            return Ok(false);
        };
        let output = cell.get_output(0);
        debug!("DoubleNegation applied to cell {}!", output.as_net());
        replace(output, inner_input)?;

        Ok(true)
    }
}

/// NAND(A, 0) = 1 (absorbing element)
#[derive(Debug)]
pub struct NandAbsorb;
impl fmt::Display for NandAbsorb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NAND(A, 0) = 1")
    }
}
impl Pattern for NandAbsorb {
    type I = Cell;
    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, Error> {
        let ct = cell_type.get_type();
        if !matches!(ct, CellType::NAND | CellType::NAND2) {
            return Ok(false);
        }
        let Some((Some(_), _, _)) = search_inputs(cell) else {
            return Ok(false);
        };
        let inst_name = cell.get_instance_name().unwrap();
        let vcc = create(
            cell_type.new_like(CellType::VCC),
            inst_name + "_const1".into(),
        );
        let output = cell.get_output(0);
        debug!("NandAbsorb: NAND absorb 0 on {}!", output.as_net());
        replace(output, vcc.get_output(0))?;
        Ok(true)
    }
}

/// NAND(A, 1) = NOT(A) (identity element)
#[derive(Debug)]
pub struct NandIdentity;
impl fmt::Display for NandIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NAND(A, 1) = NOT(A)")
    }
}
impl Pattern for NandIdentity {
    type I = Cell;
    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, Error> {
        let ct = cell_type.get_type();
        if !matches!(ct, CellType::NAND | CellType::NAND2) {
            return Ok(false);
        }
        let Some((_, Some(_), Some(other))) = search_inputs(cell) else {
            return Ok(false);
        };
        let inst_name = cell.get_instance_name().unwrap();
        let inv = create(cell_type.new_like(CellType::INV), inst_name + "_inv".into());
        inv.get_input(0).connect(other);
        let output = cell.get_output(0);
        debug!("NandIdentity: NAND(A, 1) = NOT(A) on {}!", output.as_net());
        replace(output, inv.get_output(0))?;
        Ok(true)
    }
}

/// NOR(A, 1) = 0 (absorbing element)
#[derive(Debug)]
pub struct NorAbsorb;
impl fmt::Display for NorAbsorb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NOR(A, 1) = 0")
    }
}
impl Pattern for NorAbsorb {
    type I = Cell;
    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, Error> {
        let ct = cell_type.get_type();
        if !matches!(ct, CellType::NOR | CellType::NOR2) {
            return Ok(false);
        }
        let Some((_, Some(_), _)) = search_inputs(cell) else {
            return Ok(false);
        };
        let inst_name = cell.get_instance_name().unwrap();
        let gnd = create(
            cell_type.new_like(CellType::GND),
            inst_name + "_const0".into(),
        );
        let output = cell.get_output(0);
        debug!("NorAbsorb: NOR absorb 1 on {}!", output.as_net());
        replace(output, gnd.get_output(0))?;
        Ok(true)
    }
}

/// NOR(A, 0) = NOT(A) (identity element)
#[derive(Debug)]
pub struct NorIdentity;
impl fmt::Display for NorIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NOR(A, 0) = NOT(A)")
    }
}
impl Pattern for NorIdentity {
    type I = Cell;
    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, Error> {
        let ct = cell_type.get_type();
        if !matches!(ct, CellType::NOR | CellType::NOR2) {
            return Ok(false);
        }
        let Some((Some(_), _, Some(other))) = search_inputs(cell) else {
            return Ok(false);
        };
        let inst_name = cell.get_instance_name().unwrap();
        let inv = create(cell_type.new_like(CellType::INV), inst_name + "_inv".into());
        inv.get_input(0).connect(other);
        let output = cell.get_output(0);
        debug!("NorIdentity: NOR(A, 0) = NOT(A) on {}!", output.as_net());
        replace(output, inv.get_output(0))?;
        Ok(true)
    }
}
