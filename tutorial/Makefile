
.PHONY: clean test patch

clean:
	rm -rf obj_dir rca.png

test: obj_dir/Vrca
	./obj_dir/Vrca

obj_dir/Vrca: rca.v cells.v rca_main.cpp
	verilator --cc --exe --build -Wno-PINMISSING --top-module rca rca_main.cpp rca.v

rca.png: rca.v
	nl_opt rca.v -p dot-graph | dot -Tpng > rca.png

patch: decompose_fa.patch
	git apply decompose_fa.patch
