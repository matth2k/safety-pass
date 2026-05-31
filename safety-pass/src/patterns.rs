/*!

  Simple cell patterns.

*/

use crate::{Cell, CellType, Create, Pattern, Replace};
use log::debug;
use safety_net::{Error, NetRef, DrivenNet, Instantiable};
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

/**
 * Helper to [can_combine] to check all AND CellTypes
 */
fn is_and(ct: CellType) -> bool {
    matches!(ct, CellType::AND | CellType::AND2 | CellType::AND3 | CellType::AND4)
}

/**
 * Helper to [can_combine] to check all OR CellTypes
 */
fn is_or(ct: CellType) -> bool {
    matches!(ct, CellType::OR | CellType::OR2 | CellType::OR3 | CellType::OR4)
}

/**
 * Checks if monotone pattern can combine; helper to [MonotoneFold]
 */
fn can_combine(ctype: CellType, children: &[CellType], extra: usize) -> Option<CellType> {
    if !is_and(ctype) && !is_or(ctype) {
        return None;
    }
    let andg = is_and(ctype);
    let mut fanin = extra;
    for child in children {
        if andg != is_and(*child) {
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

/// Fold monotone gates AND2(AND2(a,b), AND2(c,d)) = AND4(a,b,c,d)
#[derive(Debug)]
pub struct MonotoneFold;

impl fmt::Display for MonotoneFold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AND2(AND2(a,b), c) => AND3(a,b,c)")
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
        if !is_and(root_type) && !is_or(root_type) {
            return Ok(false);
        }

        let num_inputs = root_type.get_num_inputs();

        // Collect drivers and their cell types for each input of the root gate
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
                Some(ct) if (is_and(ct) && is_and(root_type))
                         || (is_or(ct) && is_or(root_type)) =>
                {
                    child_drivers.push((i, driver_ref, ct));
                }
                _ => {
                    non_child_drivers.push(driver);
                }
            }
        }

        // Need 1+ foldable child to do anything
        if child_drivers.is_empty() {
            return Ok(false);
        }

        // Collect all grandchild inputs
        let child_types: Vec<CellType> = child_drivers.iter().map(|(_, _, ct)| *ct).collect();

        // Check the combined fanin
        let new_type = match can_combine(root_type, &child_types, non_child_drivers.len()) {
            Some(t) => t,
            None => return Ok(false),
        };

        let new_inst_name = cell.get_instance_name().unwrap() + "_folded".into();
        let new_gate = create(Cell::new(new_type, None), new_inst_name);

        // Connect grandchildren inputs first, then non-child inputs
        let mut port_idx = 0;
        for (_, child_ref, _) in &child_drivers {
            let child_num_inputs = child_ref.get_instance_type().unwrap().get_type().get_num_inputs();
            for j in 0..child_num_inputs {
                let grandchild_driver = child_ref.get_input(j).get_driver();
                if grandchild_driver.is_none() {
                    return Ok(false);
                }
                new_gate.get_input(port_idx).connect(grandchild_driver.unwrap());
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

/// A AND 0 = 0, A OR 1 = 1 (absorbing element)
/// A AND 1 = A, A OR 0 = A (identity element)
#[derive(Debug)]
pub struct ConstantFold;

impl fmt::Display for ConstantFold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A AND 0 = 0, A AND 1 = A, A OR 1 = 1, A OR 0 = A")
    }
}

impl Pattern for ConstantFold {
    type I = Cell;

    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        _create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, Error> {
        let ct = cell_type.get_type();
        let is_and = matches!(ct, CellType::AND | CellType::AND2);
        let is_or = matches!(ct, CellType::OR | CellType::OR2);

        if !is_and && !is_or {
            return Ok(false);
        }

        let num_inputs = ct.get_num_inputs();
        let mut const_false: Option<usize> = None;
        let mut const_true: Option<usize> = None;
        let mut other: Option<DrivenNet<Self::I>> = None;

        for i in 0..num_inputs {
            let driver = cell.get_input(i).get_driver();
            if driver.is_none() {
                return Ok(false);
            }
            let driver = driver.unwrap();
            let driver_ref = driver.clone().unwrap();
            match driver_ref
                .get_instance_type()
                .and_then(|t| t.get_constant())
            {
                Some(safety_net::Logic::False) => { const_false = Some(i); }
                Some(safety_net::Logic::True)  => { const_true = Some(i); }
                _ => { other = Some(driver); }
            }
        }

        let output = cell.get_output(0);

        if is_and {
            if let Some(_) = const_false {
                // AND with 0 = 0 passthrough replace output with GND driver
                let gnd_driver = cell.get_input(const_false.unwrap()).get_driver().unwrap();
                debug!("ConstantFold: AND absorb 0 on cell {}!", output.as_net());
                replace(output, gnd_driver)?;
                return Ok(true);
            }
            if const_true.is_some() {
                if let Some(other_driver) = other {
                    // AND with 1 = other input
                    debug!("ConstantFold: AND identity 1 on cell {}!", output.as_net());
                    replace(output, other_driver)?;
                    return Ok(true);
                }
            }
        }

        if is_or {
            if let Some(_) = const_true {
                // OR with 1 = 1 passthrough replace output with VCC driver
                let vcc_driver = cell.get_input(const_true.unwrap()).get_driver().unwrap();
                debug!("ConstantFold: OR absorb 1 on cell {}!", output.as_net());
                replace(output, vcc_driver)?;
                return Ok(true);
            }
            if const_false.is_some() {
                if let Some(other_driver) = other {
                    // OR with 0 = other input
                    debug!("ConstantFold: OR identity 0 on cell {}!", output.as_net());
                    replace(output, other_driver)?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
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

/// NAND(A, 0) = 1, NAND(A, 1) = NOT(A)
/// NOR(A, 1)  = 0, NOR(A, 0)  = NOT(A)
#[derive(Debug)]
pub struct ConstantNandNor;

impl fmt::Display for ConstantNandNor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NAND(A,0)=1, NAND(A,1)=NOT(A), NOR(A,1)=0, NOR(A,0)=NOT(A)")
    }
}

impl Pattern for ConstantNandNor {
    type I = Cell;

    fn apply(
        &self,
        cell: &NetRef<Self::I>,
        cell_type: &Self::I,
        create: &Create<Self::I>,
        replace: &mut Replace<Self::I>,
    ) -> Result<bool, Error> {
        let ct = cell_type.get_type();
        let is_nand = matches!(ct, CellType::NAND | CellType::NAND2);
        let is_nor  = matches!(ct, CellType::NOR  | CellType::NOR2);

        if !is_nand && !is_nor {
            return Ok(false);
        }

        let num_inputs = ct.get_num_inputs();
        let mut const_false: Option<DrivenNet<Self::I>> = None;
        let mut const_true:  Option<DrivenNet<Self::I>> = None;
        let mut other:       Option<DrivenNet<Self::I>> = None;

        for i in 0..num_inputs {
            let driver = cell.get_input(i).get_driver();
            if driver.is_none() {
                return Ok(false);
            }
            let driver = driver.unwrap();
            let driver_ref = driver.clone().unwrap();
            match driver_ref
                .get_instance_type()
                .and_then(|t| t.get_constant())
            {
                Some(safety_net::Logic::False) => { const_false = Some(driver); }
                Some(safety_net::Logic::True)  => { const_true  = Some(driver); }
                _ => { other = Some(driver); }
            }
        }

        let output = cell.get_output(0);
        let inst_name = cell.get_instance_name().unwrap();

        if is_nand {
            if let Some(gnd) = const_false {
                // NAND(A, 0) = 1 — replace with VCC
                let vcc = create(
                    Cell::new(CellType::VCC, None),
                    inst_name + "_const1".into(),
                );
                debug!("ConstantNandNor: NAND absorb 0 on {}!", output.as_net());
                replace(output, vcc.get_output(0))?;
                let _ = gnd;
                return Ok(true);
            }
            if const_true.is_some() {
                if let Some(other_driver) = other {
                    // NAND(A, 1) = NOT(A)
                    let inv = create(
                        Cell::new(CellType::INV, None),
                        inst_name + "_inv".into(),
                    );
                    inv.get_input(0).connect(other_driver);
                    debug!("ConstantNandNor: NAND(A,1)=NOT(A) on {}!", output.as_net());
                    replace(output, inv.get_output(0))?;
                    return Ok(true);
                }
            }
        }

        if is_nor {
            if let Some(vcc) = const_true {
                // NOR(A, 1) = 0 — replace with GND
                let gnd = create(
                    Cell::new(CellType::GND, None),
                    inst_name + "_const0".into(),
                );
                debug!("ConstantNandNor: NOR absorb 1 on {}!", output.as_net());
                replace(output, gnd.get_output(0))?;
                let _ = vcc;
                return Ok(true);
            }
            if const_false.is_some() {
                if let Some(other_driver) = other {
                    // NOR(A, 0) = NOT(A)
                    let inv = create(
                        Cell::new(CellType::INV, None),
                        inst_name + "_inv".into(),
                    );
                    inv.get_input(0).connect(other_driver);
                    debug!("ConstantNandNor: NOR(A,0)=NOT(A) on {}!", output.as_net());
                    replace(output, inv.get_output(0))?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}