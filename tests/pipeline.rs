use safety_net::{Net, Netlist};
use safety_pass::{Cell, CellType, Folder, Pipeline, patterns::Idempotent};
use std::rc::Rc;

fn and_gate() -> Cell {
    Cell::new(CellType::AND2, None)
}

fn ex_netlist() -> Rc<Netlist<Cell>> {
    let nl = Netlist::new("top".to_string());
    let a = nl.insert_input(Net::new_logic("a".into()));
    let b = nl.insert_input(Net::new_logic("b".into()));
    let g = nl
        .insert_gate(and_gate(), "inst_0".into(), &[a, b])
        .unwrap()
        .get_output(0);
    let h = nl
        .insert_gate(and_gate(), "inst_1".into(), &[g.clone(), g])
        .unwrap();

    h.expose_with_name("y".into());

    nl
}

#[test]
fn test_pipeline() {
    let nl = ex_netlist();

    let mut folder = Folder::<Cell>::new(101);
    folder.insert(Idempotent);

    let mut pipeline = Pipeline::default();
    pipeline.insert(folder);

    let before = nl.len();

    let res = pipeline.execute(&nl, false);
    assert!(res.is_ok());

    let after = nl.len();
    assert_eq!(after + 1, before);

    assert_eq!(res.unwrap(), "Folded 1 patterns over 1 iterations");
}
