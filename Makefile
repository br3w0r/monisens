.PHONY: module-bindings
module-bindings:
	$(info Generaing FFI bindings...)
	bindgen ./module/monisens_def.h\
		--default-enum-style rust \
		--allowlist-file './module/monisens_def.h' \
		--raw-line '#![allow(warnings)]' \
		-o ./src/module/bindings_gen.rs

.PHONY: generate
generate: module-bindings
