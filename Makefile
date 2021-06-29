.PHONY: build-cli
build-anchor-cli:
	cargo build -p anchor-cli --release
	cp target/release/anchor cli/npm-package/anchor
