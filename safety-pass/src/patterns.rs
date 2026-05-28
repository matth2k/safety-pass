/*!

  Simple cell patterns.

*/

use crate::{Cell, CellType, Create, Pattern, Replace};
use log::debug;
use safety_net::{Error, NetRef, DrivenNet, FanOutTable};
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
        _fan_out: &FanOutTable<Self::I>,
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

/// Returns None if the cells if
/// (1) A gate is not monotone (AND or OR)
/// -or- (2) A gate is mismatched (need to be all AND or all OR)
/// -or- (3) The fan-in of all the children gates is too large
/// -or- (4) A gate feeds itself
fn can_combine(ctype: CellType, children: &[CellType]) -> Option<CellType> {
    if !is_and(ctype) && !is_or(ctype) {
        return None;
    }
    let andg = is_and(ctype);
    let mut fanin = 0;
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

/// Fold monotone gates: AND2(AND2(a,b), AND2(c,d)) => AND4(a,b,c,d)
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
        fan_out: &FanOutTable<Self::I>,
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

            // Only fold if this child has fan-out of 1 (nothing else uses it)
            let child_ctype = driver_ref.get_instance_type().map(|t| t.get_type());
            match child_ctype {
                Some(ct) if (is_and(ct) && is_and(root_type))
                         || (is_or(ct) && is_or(root_type)) =>
                {
                    let users = fan_out.get_node_users(&driver_ref).count();
                    if users == 1 {
                        child_drivers.push((i, driver_ref, ct));
                    } else {
                        non_child_drivers.push(driver);
                    }
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
        let total_fanin: usize = child_types.iter().map(|ct| ct.get_num_inputs()).sum::<usize>()
            + non_child_drivers.len();

        // Check the combined fanin fits a supported gate size
        let new_type = match (is_and(root_type), total_fanin) {
            (true, 2) => CellType::AND2,
            (true, 3) => CellType::AND3,
            (true, 4) => CellType::AND4,
            (false, 2) => CellType::OR2,
            (false, 3) => CellType::OR3,
            (false, 4) => CellType::OR4,
            _ => return Ok(false),
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