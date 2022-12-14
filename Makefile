.PHONY: module-bindings
module-bindings:
	$(info Generaing FFI bindings...)
	bindgen ./module/monisens_api.h -o ./src/module/bindings-gen.rs

.PHONY: generate
generate: module-bindings
