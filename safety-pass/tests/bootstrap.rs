use safety_net::{
    Gate, Identifier, Instantiable, Net, Netlist, Parameter, assert_verilog_eq, format_id,
};
use safety_pass::{ModInst, ModOrCell};
use std::collections::HashMap;
use std::rc::Rc;

type Inst = ModOrCell<Gate>;

fn and() -> Gate {
    Gate::new_logical("AND2".into(), vec!["A".into(), "B".into()], "Y".into())
}

fn inv() -> Gate {
    Gate::new_logical("INV".into(), vec!["A".into()], "Y".into())
}

fn simple_nl() -> Rc<Netlist<Gate>> {
    let nl = Netlist::new("simple".into());

    let a = nl.insert_input(Net::new_logic("a".into()));
    let b = nl.insert_input(Net::new_logic("b".into()));
    let y = nl.insert_gate(and(), "and_inst".into(), &[a, b]).unwrap();

    let y_inv = nl
        .insert_gate(inv(), "inv_inst".into(), &[y.into()])
        .unwrap();
    y_inv.expose_with_name("y".into());

    nl
}

fn simple_and() -> Rc<Netlist<ModOrCell<Gate>>> {
    let nl = Netlist::new("simple".into());

    let a = nl.insert_input(Net::new_logic("a".into()));
    let b = nl.insert_input(Net::new_logic("b".into()));
    let y = nl
        .insert_gate(ModOrCell::Cell(and()), "and_inst".into(), &[a, b])
        .unwrap();

    let y_inv = nl
        .insert_gate(ModOrCell::Cell(inv()), "inv_inst".into(), &[y.into()])
        .unwrap();
    y_inv.expose_with_name("y".into());

    nl
}

fn nested_and() -> Rc<Netlist<ModOrCell<Gate>>> {
    let nl = Netlist::new("nested".into());

    let a = nl.insert_input(Net::new_logic("a".into()));
    let b = nl.insert_input(Net::new_logic("b".into()));
    let c = nl.insert_input(Net::new_logic("c".into()));
    let d = nl.insert_input(Net::new_logic("d".into()));

    let inner = simple_and();
    let inst = ModOrCell::ModInst(ModInst::new(&inner));

    let a_b = nl
        .insert_gate(inst.clone(), "inst1".into(), &[a, b])
        .unwrap();
    let c_d = nl
        .insert_gate(inst.clone(), "inst2".into(), &[c, d])
        .unwrap();
    let y = nl
        .insert_gate(inst.clone(), "inst3".into(), &[a_b.into(), c_d.into()])
        .unwrap();

    y.expose_with_name("y".into());

    nl
}

fn passthru_nl<I: Instantiable>(id: Identifier) -> Rc<Netlist<I>> {
    let nl = Netlist::new(id);

    let x = nl.insert_input(Net::new_logic("x".into()));

    x.expose_with_name("y".into());

    nl
}

#[test]
fn test_nesting() {
    let outer: Rc<Netlist<Inst>> = passthru_nl("outer".into());
    let inner: Rc<Netlist<Inst>> = passthru_nl("inner".into());

    let inst = ModOrCell::ModInst(ModInst::new(&inner));

    assert!(!inst.is_seq());

    let nr = outer.insert_gate_disconnected(inst, "inst".into());

    let oin = outer.first().unwrap().get_output(0);
    let oin = outer.replace_net_uses(oin, &nr.get_output(0)).unwrap();
    nr.get_input(0).connect(oin);

    let verilog = inner.to_string() + "\n" + &outer.to_string();

    assert_verilog_eq!(
        verilog,
        "module inner (
           x,
           y
         );
           input wire x;
           output wire y;
         
         
           assign y = x;
         
         endmodule
         
         module outer (
           x,
           y
         );
           input wire x;
           output wire y;
           wire inst_y;
         
           inner inst (
             .x(x),
             .y(inst_y)
           );
         
           assign y = inst_y;
         
         endmodule"
            .to_string()
    );
}

#[test]
#[should_panic(expected = "Cannot set parameter")]
fn test_modinst_set_param() {
    let inner: Rc<Netlist<Inst>> = passthru_nl("inner".into());

    let mut inst = ModOrCell::ModInst(ModInst::new(&inner));

    assert!(!inst.has_parameter(&"ex".into()));
    assert!(inst.get_parameter(&"ex".into()).is_none());

    inst.set_parameter(&"ex".into(), Parameter::from_bool(true));
}

#[test]
fn test_modinst_get_const() {
    let inst = ModOrCell::<Gate>::from_constant(false.into()).unwrap();

    assert!(matches!(inst, ModOrCell::Cell(_)));

    assert_eq!(inst.get_constant(), Some(false.into()));
    assert!(!inst.is_seq());
}

#[test]
fn test_clone_into_inst_into() {
    let outer: Rc<Netlist<Inst>> = passthru_nl("outer".into());
    let inner: Rc<Netlist<Gate>> = passthru_nl("inner".into());

    let input = inner.first().unwrap();
    let _clone = outer.clone_into(&input, Some("myclone".into()), &mut HashMap::new());
    assert!(outer.verify().is_ok());
    assert_verilog_eq!(
        outer.to_string(),
        "module outer (
           myclone_x,
           x,
           y
         );
           input wire myclone_x;
           input wire x;
           output wire y;


           assign y = x;

         endmodule"
            .to_string()
    );
}

#[test]
fn test_inline() {
    let nl = simple_nl();
    let inputs = nl.inputs().map(|n| Some(n)).collect::<Vec<_>>();
    let inst = ModInst::new(&nl);

    for i in 0..2 {
        let res = inst.inline_into(&nl, Some(format_id!("inlined_{i}")), &inputs);
        assert!(res.is_ok());
    }

    assert_verilog_eq!(
        nl.to_string(),
        "module simple (
           a,
           b,
           y
         );
           input wire a;
           input wire b;
           output wire y;
           wire and_inst_Y;
           wire inlined_0_and_inst_Y;
           wire inlined_0_inv_inst_Y;
           wire inlined_1_and_inst_Y;
           wire inlined_1_inv_inst_Y;
           wire inv_inst_Y;
           AND2 and_inst (
             .A(a),
             .B(b),
             .Y(and_inst_Y)
           );
           INV inv_inst (
             .A(and_inst_Y),
             .Y(inv_inst_Y)
           );
           AND2 inlined_0_and_inst (
             .A(a),
             .B(b),
             .Y(inlined_0_and_inst_Y)
           );
           INV inlined_0_inv_inst (
             .A(inlined_0_and_inst_Y),
             .Y(inlined_0_inv_inst_Y)
           );
           AND2 inlined_1_and_inst (
             .A(a),
             .B(b),
             .Y(inlined_1_and_inst_Y)
           );
           INV inlined_1_inv_inst (
             .A(inlined_1_and_inst_Y),
             .Y(inlined_1_inv_inst_Y)
           );
           assign y = inv_inst_Y;
         endmodule"
            .to_string()
    );
}

#[test]
fn test_inline_recursive() {
    let nl = nested_and();

    assert!(nl.verify().is_ok());

    let inlined = safety_pass::inline_recursive(&nl);
    assert!(inlined.is_ok());
    let inlined = inlined.unwrap();

    assert_verilog_eq!(
        inlined.to_string(),
        "module nested (
           a,
           b,
           c,
           d,
           y
         );
           input wire a;
           input wire b;
           input wire c;
           input wire d;
           output wire y;
           wire inst1_and_inst_Y;
           wire inst1_inv_inst_Y;
           wire inst2_and_inst_Y;
           wire inst2_inv_inst_Y;
           wire inst3_and_inst_Y;
           wire inst3_inv_inst_Y;
           AND2 inst1_and_inst (
             .A(a),
             .B(b),
             .Y(inst1_and_inst_Y)
           );
           INV inst1_inv_inst (
             .A(inst1_and_inst_Y),
             .Y(inst1_inv_inst_Y)
           );
           AND2 inst2_and_inst (
             .A(c),
             .B(d),
             .Y(inst2_and_inst_Y)
           );
           INV inst2_inv_inst (
             .A(inst2_and_inst_Y),
             .Y(inst2_inv_inst_Y)
           );
           AND2 inst3_and_inst (
             .A(inst1_inv_inst_Y),
             .B(inst2_inv_inst_Y),
             .Y(inst3_and_inst_Y)
           );
           INV inst3_inv_inst (
             .A(inst3_and_inst_Y),
             .Y(inst3_inv_inst_Y)
           );
           assign y = inst3_inv_inst_Y;
         endmodule"
            .to_string()
    );
}
