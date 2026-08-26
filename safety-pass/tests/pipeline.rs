use safety_net::{Net, Netlist, assert_verilog_eq};
use safety_pass::{Cell, CellType, Folder, Pipeline, patterns::Idempotent};
use std::rc::Rc;

fn and_gate() -> Cell {
    Cell::new(CellType::AND2, None)
}

fn ex_netlist() -> Rc<Netlist<Cell>> {
    let nl = Netlist::new("top".into());
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

#[test]
fn test_folder_debug() {
    use safety_pass::patterns::{DoubleNegation, Idempotent};
    let mut folder = Folder::new(101);
    folder.insert(DoubleNegation);
    folder.insert(Idempotent);

    let debug_str = format!("{folder:#?}");
    assert_verilog_eq!(
        debug_str,
        "Folder {
            patterns: [
                DoubleNegation,
                Idempotent,
            ],
            max_iters: 101,
        }"
    );
}

#[test]
fn test_proc_multiple() {
    let nl = ex_netlist();
    let mut vec = Vec::new();
    for _ in 0..4 {
        vec.push(nl.deep_clone());
    }
    vec.push(nl);
    let mut pipeline = Pipeline::default();
    {
        let mut folder = Folder::<Cell>::new(101);
        folder.insert(Idempotent);

        pipeline.insert(folder);
    }

    let res = pipeline.execute_many(&vec, false);
    assert!(res.is_ok());
    let res = res.unwrap();
    assert_eq!(res.lines().count(), 5);
}
