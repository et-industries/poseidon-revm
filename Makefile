.PHONY: contracts
contracts:
	forge build --revert-strings debug -C examples/ --extra-output-files abi --out src/out

.PHONY: build
build:
	cargo build

.PHONY: examples
examples: contracts build
	cargo run -p revm-test --example poseidon_test
