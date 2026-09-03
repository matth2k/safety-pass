
.PHONY: clean test patch

clean:
	rm -rf obj_dir rca.png

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
