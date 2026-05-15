/*!

  Simple cell patterns.

*/

use crate::{Cell, CellType, Create, Error, Pattern, Replace};
use log::debug;
use safety_net::NetRef;

/// A * A = A
pub struct Idempotent;

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
