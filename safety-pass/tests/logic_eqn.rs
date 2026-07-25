use safety_pass::CellType;

#[test]
fn inv_verilog_output() {
    // INV(a) = !a
    let eqn = CellType::INV.get_logic_eqn();
    assert_eq!(eqn.to_string(), "wire n1;\nassign n1 = ~A;\n");
}

#[test]
fn and2_verilog_output() {
    // AND2(A1, A2) = A1 & A2
    let eqn = CellType::AND2.get_logic_eqn();
    assert_eq!(eqn.to_string(), "wire n2;\nassign n2 = A1 & A2;\n");
}

#[test]
fn nand2_verilog_output() {
    // NAND2(A1, A2) = !(A1 & A2)
    let eqn = CellType::NAND2.get_logic_eqn();
    assert_eq!(
        eqn.to_string(),
        "wire n2;\nwire n3;\nassign n2 = A1 & A2;\nassign n3 = ~n2;\n"
    );
}

#[test]
fn aoi21_input_ports() {
    // AOI21 ports: A, B1, B2
    let eqn = CellType::AOI21.get_logic_eqn();
    assert_eq!(eqn.input_names(), vec!["A", "B1", "B2"]);
}

#[test]
fn maj3_output_expected() {
    // we just check if maj3 builds
    let eqn = CellType::MAJ3.get_logic_eqn();
    assert!(eqn.output().is_some());
}
