
.PHONY: clean test patch solution

clean:
	rm -rf obj_dir rca.png obj_dir_swapped rca_swapped.png rca_swapped.v

test: obj_dir/Vrca
	./obj_dir/Vrca

# Rebuilds the testbench
obj_dir/Vrca: rca.v cells.v rca_main.cpp
	verilator --cc --exe --build -Wno-PINMISSING --top-module rca rca_main.cpp rca.v

# Builds a visualization
rca.png: rca.v
	cargo run --release --quiet -- rca.v -p dot-graph | dot -Tpng > rca.png

# Gives you the boilerplate code for a new pass
patch: pass_template.patch
	cd .. && git apply tutorial/pass_template.patch

# Applies the reference solution after the starter patch
solution: pass_solution.patch
	cd .. && git apply tutorial/pass_solution.patch