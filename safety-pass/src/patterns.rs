/*!

  Simple cell patterns.

*/

use crate::{Cell, CellType, Create, Pattern, Replace};
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
/// This allows collapsing arbitrary balanced or unbalanced trees of AND/OR gates into a single wider gate.
#[derive(Debug)]
pub struct MonotoneFold;

impl fmt::Display for MonotoneFold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AND2(AND2(a,b), c) => AND3(a,b,c)")
    }
}

impl MonotoneFold {
    /// Helper to [apply] that checks homogeneity of fanin gates
    fn can_combine(ct: CellType, children: &[CellType], extra: usize) -> Option<CellType> {
        if ct.is_and() && ct.is_or() {
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

        let num_inputs = root_type.get_num_inputs();

        let mut child_drivers: Vec<(usize, NetRef<Self::I>, CellType)> = Vec::new();
        let mut non_child_drivers: Vec<DrivenNet<Self::I>> = Vec::new();

        for i in 0..num_inputs {
            let driver = cell.get_input(i).get_driver();
            if driver.is_none() {
                return Ok(false);
            }

            let driver = driver.unwrap();
            let driver_ref = driver.clone().unwrap();

            let child_ctype = driver_ref.get_instance_type().map(|t| t.get_type());

            match child_ctype {
                Some(ct)
                    if (ct.is_and() && root_type.is_and()) || (ct.is_or() && root_type.is_or()) =>
                {
                    child_drivers.push((i, driver_ref, ct));
                }
                _ => {
                    non_child_drivers.push(driver);
                }
            }
        }

        if child_drivers.is_empty() {
            return Ok(false);
        }

        let child_types: Vec<CellType> = child_drivers.iter().map(|(_, _, ct)| *ct).collect();

        let new_type = match Self::can_combine(root_type, &child_types, non_child_drivers.len()) {
            Some(t) => t,
            None => return Ok(false),
        };

        let new_inst_name = cell.get_instance_name().unwrap() + "_folded".into();
        let new_gate = create(Cell::new(new_type, None), new_inst_name);

        let mut port_idx = 0;
        for (_, child_ref, _) in &child_drivers {
            let child_num_inputs = child_ref
                .get_instance_type()
                .unwrap()
                .get_type()
                .get_num_inputs();
            for j in 0..child_num_inputs {
                let grandchild_driver = child_ref.get_input(j).get_driver();
                if grandchild_driver.is_none() {
                    return Ok(false);
                }
                new_gate
                    .get_input(port_idx)
                    .connect(grandchild_driver.unwrap());
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

type ConstInputs = Option<(Option<usize>, Option<usize>, Option<DrivenNet<Cell>>)>;

/// Scans the inputs of a 2-input gate and returns which slots hold a
/// constant-false driver, a constant-true driver, and the non-constant driver.
/// Returns `None` if any input is undriven (pattern cannot fire).
/// Helper to [AndAbsorb], [OrAbsorb], [AndIdentity], and [OrIdentity]
fn search_constant_inputs(cell: &NetRef<Cell>, ct: &CellType) -> ConstInputs {
    let num_inputs = ct.get_num_inputs();
    let mut const_false: Option<usize> = None;
    let mut const_true: Option<usize> = None;
    let mut other: Option<DrivenNet<Cell>> = None;

    for i in 0..num_inputs {
        let driver = cell.get_input(i).get_driver()?;
        let driver_ref = driver.clone().unwrap();
        match driver_ref
            .get_instance_type()
            .and_then(|t| t.get_constant())
        {
            Some(safety_net::Logic::False) => {
                const_false = Some(i);
            }
            Some(safety_net::Logic::True) => {
                const_true = Some(i);
            }
            _ => {
                other = Some(driver);
            }
        }
    }

    Some((const_false, const_true, other))
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
        let Some((const_false, _, _)) = search_constant_inputs(cell, &ct) else {
            return Ok(false);
        };
        let Some(idx) = const_false else {
            return Ok(false);
        };
        let gnd_driver = cell.get_input(idx).get_driver().unwrap();
        let output = cell.get_output(0);
        debug!("AndAbsorb: AND absorb 0 on cell {}!", output.as_net());
        replace(output, gnd_driver)?;
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
        let Some((_, const_true, other)) = search_constant_inputs(cell, &ct) else {
            return Ok(false);
        };
        if const_true.is_none() {
            return Ok(false);
        }
        let Some(other_driver) = other else {
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
        let Some((_, const_true, _)) = search_constant_inputs(cell, &ct) else {
            return Ok(false);
        };
        let Some(idx) = const_true else {
            return Ok(false);
        };
        let vcc_driver = cell.get_input(idx).get_driver().unwrap();
        let output = cell.get_output(0);
        debug!("OrAbsorb: OR absorb 1 on cell {}!", output.as_net());
        replace(output, vcc_driver)?;
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
        let Some((const_false, _, other)) = search_constant_inputs(cell, &ct) else {
            return Ok(false);
        };
        if const_false.is_none() {
            return Ok(false);
        }
        let Some(other_driver) = other else {
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
        // Outer gate must be an inverter
        if !matches!(cell_type.get_type(), CellType::NOT | CellType::INV) {
            return Ok(false);
        }

        let driver = cell.get_input(0).get_driver();
        if driver.is_none() {
            return Ok(false);
        }
        let driver = driver.unwrap();
        let driver_ref = driver.clone().unwrap();

        // Inner gate must also be an inverter
        let inner_type = driver_ref.get_instance_type().map(|t| t.get_type());
        if !matches!(inner_type, Some(CellType::NOT) | Some(CellType::INV)) {
            return Ok(false);
        }

        // Get the input to the inner inverter
        let inner_input = driver_ref.get_input(0).get_driver();
        if inner_input.is_none() {
            return Ok(false);
        }

        let output = cell.get_output(0);
        debug!("DoubleNegation applied to cell {}!", output.as_net());
        replace(output, inner_input.unwrap())?;

        Ok(true)
    }
}

type DrivenConstInputs = Option<(
    Option<DrivenNet<Cell>>,
    Option<DrivenNet<Cell>>,
    Option<DrivenNet<Cell>>,
)>;

/// Scans the inputs of a 2-input gate. Returns `None` if any input is undriven.
/// Slots hold the full driver so absorb patterns can forward the constant cell's
/// output directly; the non-constant input is stored in `other`.
fn search_driven_inputs(cell: &NetRef<Cell>, ct: &CellType) -> DrivenConstInputs {
    let num_inputs = ct.get_num_inputs();
    let mut const_false: Option<DrivenNet<Cell>> = None;
    let mut const_true: Option<DrivenNet<Cell>> = None;
    let mut other: Option<DrivenNet<Cell>> = None;

    for i in 0..num_inputs {
        let driver = cell.get_input(i).get_driver()?;
        let driver_ref = driver.clone().unwrap();
        match driver_ref
            .get_instance_type()
            .and_then(|t| t.get_constant())
        {
            Some(safety_net::Logic::False) => {
                const_false = Some(driver);
            }
            Some(safety_net::Logic::True) => {
                const_true = Some(driver);
            }
            _ => {
                other = Some(driver);
            }
        }
    }

    Some((const_false, const_true, other))
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
        let Some((const_false, _, _)) = search_driven_inputs(cell, &ct) else {
            return Ok(false);
        };
        if const_false.is_none() {
            return Ok(false);
        }
        let inst_name = cell.get_instance_name().unwrap();
        let vcc = create(Cell::new(CellType::VCC, None), inst_name + "_const1".into());
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
        let Some((_, const_true, other)) = search_driven_inputs(cell, &ct) else {
            return Ok(false);
        };
        if const_true.is_none() {
            return Ok(false);
        }
        let Some(other_driver) = other else {
            return Ok(false);
        };
        let inst_name = cell.get_instance_name().unwrap();
        let inv = create(Cell::new(CellType::INV, None), inst_name + "_inv".into());
        inv.get_input(0).connect(other_driver);
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
        let Some((_, const_true, _)) = search_driven_inputs(cell, &ct) else {
            return Ok(false);
        };
        if const_true.is_none() {
            return Ok(false);
        }
        let inst_name = cell.get_instance_name().unwrap();
        let gnd = create(Cell::new(CellType::GND, None), inst_name + "_const0".into());
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
        let Some((const_false, _, other)) = search_driven_inputs(cell, &ct) else {
            return Ok(false);
        };
        if const_false.is_none() {
            return Ok(false);
        }
        let Some(other_driver) = other else {
            return Ok(false);
        };
        let inst_name = cell.get_instance_name().unwrap();
        let inv = create(Cell::new(CellType::INV, None), inst_name + "_inv".into());
        inv.get_input(0).connect(other_driver);
        let output = cell.get_output(0);
        debug!("NorIdentity: NOR(A, 0) = NOT(A) on {}!", output.as_net());
        replace(output, inv.get_output(0))?;
        Ok(true)
    }
}
