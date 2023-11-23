RUSTC = cargo
RUST_PROC_MACRO_LIB = proc_macro_lib/src/lib.rs
COMMON_STRUCT = go4_unpack_struct.hh
STRUCTURE_FILE = structures.hh

MAIN_SPEC = event.spec

all:
	touch -c  $(RUST_PROC_MACRO_LIB)
	gcc -E -x c++ $(MAIN_SPEC) -P -o __main_event.spec
	rm -f *.struct
	@$(RUSTC) build
	@$(RUSTC) run
	@echo '#include "$(COMMON_STRUCT)"\n' > $(STRUCTURE_FILE)
	@ls -t | grep -E '\.struct$$' | tac | xargs cat >> $(STRUCTURE_FILE)
	@rm -f *.struct
	@rm -f __main_event.spec

.PHONY: clean
clean:
	$(RUSTC) clean
	rm -f *.struct
	rm -f structures.hh

