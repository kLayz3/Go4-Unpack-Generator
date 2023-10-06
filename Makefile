RUSTC = cargo
RUST_PROC_MACRO_LIB := proc_macro_lib/src/lib.rs
COMMON_STRUCT = go4_unpack_struct.common
STRUCTURE_FILE = structures.hh

all:
	touch -c  $(RUST_PROC_MACRO_LIB)
	rm -f *.struct
	@$(RUSTC) build
	@$(RUSTC) run
	@rm -f $(STRUCTURE_FILE) && touch $(STRUCTURE_FILE)
	@echo '#include "$(COMMON_STRUCT)"\n' > $(STRUCTURE_FILE)
	@ls -t | grep -E '\.struct$$' | tac | xargs cat >> $(STRUCTURE_FILE)
	@rm -f *.struct

.PHONY: clean
clean:
	$(RUSTC) clean

