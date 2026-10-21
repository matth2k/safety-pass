use safety_net::assert_verilog_eq;
use safety_pass::{Cell, CellType, VerilogEqn, logic_rule};

#[test]
fn check_and() {
    let result = logic_rule!(@expr CellType::AND2; 0 && 1);
    assert_eq!(result, "A1 & A2");
}

#[test]
fn check_negation() {
    let result = logic_rule!(@expr CellType::INV; !0);
    assert_eq!(result, "~A");
}

#[test]
fn check_and_negation() {
    let result1 = logic_rule!(@expr CellType::AND2;  (!0)  &&   1); // Dummy equations
    let result2 = logic_rule!(@expr CellType::AND2;   0  &&  (!1));
    let result3 = logic_rule!(@expr CellType::AND2;  (!0)  &&  (!1));

    assert_eq!(result1, "~A1 & A2");
    assert_eq!(result2, "A1 & ~A2");
    assert_eq!(result3, "~A1 & ~A2");
}

#[test]
fn check_or() {
    let result = logic_rule!(@expr CellType::OR2; 0 || 1);

    assert_eq!(result, "A1 | A2");
}

#[test]
fn check_or_negation() {
    let result1 = logic_rule!(@expr CellType::OR2; (!0)  ||   1);
    let result2 = logic_rule!(@expr CellType::OR2;  0  ||  (!1));
    let result3 = logic_rule!(@expr CellType::OR2; (!0)  ||  (!1));

    assert_eq!(result1, "~A1 | A2");
    assert_eq!(result2, "A1 | ~A2");
    assert_eq!(result3, "~A1 | ~A2");
}

#[test]
fn check_and_chain() {
    let result = logic_rule!(@expr CellType::AND3; 0 && 1 && 2);
    assert_eq!(result, "A1 & A2 & A3");
}

#[test]
fn check_nested_parens() {
    let result = logic_rule!(@expr CellType::AND2; (0 && (!1)) || ((!0) && 1));
    assert_eq!(result, "A1 & ~A2 | ~A1 & A2");
}

#[test]
fn check_and2_module() {
    let cell = Cell::new(CellType::AND2, None);
    let module = cell.get_verilog_module();
    let src = "module AND2 (
                         input A1,
                         input A2,
                         output ZN
                     );
                       assign ZN = A1 & A2;
                     endmodule";
    assert_verilog_eq!(module, src);
}

#[test]
fn check_lut2_module() {
    let cell = Cell::new(CellType::LUT2, None);
    let module = cell.get_verilog_module();
    let src = "module LUT2 #(
                         parameter INIT = 4'h0
                     ) (
                         input I1,
                         input I0,
                         output O
                     );
                       assign O = INIT[{I1, I0}];
                     endmodule";
    assert_verilog_eq!(module, src);
}
