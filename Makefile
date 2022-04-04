.PHONY: build-cli
build-cli:
	cargo build -p anchor-cli --release
	cp target/release/anchor cli/npm-package/anchor

.PHONY: build-tests-bpf-%
build-tests-bpf-%: export NAME=$(subst _,/,$($(strip @):build-tests-bpf-%=%))
build-tests-bpf-%:
	cd tests/${NAME} && cargo build-bpf

.PHONY: build-example-bpf-%
build-example-bpf-%: export NAME=$(subst _,/,$($(strip @):build-example-bpf-%=%))
build-example-bpf-%:
	cd examples/tutorial/${NAME} && cargo build-bpf

.PHONY: build-example-bpf-permissioned-markets
build-tests-bpf-permissioned-markets:
	cd tests/permissioned-markets/deps/serum-dex/dex && cargo build-bpf
	cd tests/permissioned-markets && cargo build-bpf

.PHONY: build-example-bpf-swap
build-tests-bpf-swap:
	cd tests/swap/deps/serum-dex/dex && cargo build-bpf
	cd tests/swap && cargo build-bpf

.PHONY: build-tests-bpf-all
build-tests-bpf-all: build-tests-bpf-cashiers-check
build-tests-bpf-all: build-tests-bpf-cfo
build-tests-bpf-all: build-tests-bpf-chat
build-tests-bpf-all: build-tests-bpf-composite
build-tests-bpf-all: build-tests-bpf-errors
build-tests-bpf-all: build-tests-bpf-escrow
build-tests-bpf-all: build-tests-bpf-events
build-tests-bpf-all: build-tests-bpf-ido-pool
build-tests-bpf-all: build-tests-bpf-interface
build-tests-bpf-all: build-tests-bpf-lockup
build-tests-bpf-all: build-tests-bpf-misc
build-tests-bpf-all: build-tests-bpf-multisig
build-tests-bpf-all: build-tests-bpf-permissioned-markets
build-tests-bpf-all: build-tests-bpf-pyth
build-tests-bpf-all: build-tests-bpf-spl_token-proxy
build-tests-bpf-all: build-tests-bpf-swap
build-tests-bpf-all: build-tests-bpf-sysvars
build-tests-bpf-all: build-tests-bpf-typescript
build-tests-bpf-all: build-tests-bpf-zero-copy

.PHONY: build-example-bpf-all
build-example-bpf-all: build-example-bpf-tutorial_basic-0
build-example-bpf-all: build-example-bpf-tutorial_basic-1
build-example-bpf-all: build-example-bpf-tutorial_basic-2
build-example-bpf-all: build-example-bpf-tutorial_basic-3
build-example-bpf-all: build-example-bpf-tutorial_basic-4

.PHONY: build-all
build-all: build-tests-bpf-all build-example-bpf-all

.PHONY: clean
clean:
	find . -type d -name .anchor -print0 | xargs -0 rm -rf
	find . -type d -name node_modules -print0 | xargs -0 rm -rf
	find . -type d -name target -print0 | xargs -0 rm -rf

.PHONY: publish
publish:
	cd lang/syn/ && cargo publish && cd ../../
	sleep 25
	cd lang/derive/accounts/ && cargo publish && cd ../../../
	sleep 25
	cd lang/attribute/access-control/ && cargo publish && cd ../../../
	sleep 25
	cd lang/attribute/account/ && cargo publish && cd ../../../
	sleep 25
	cd lang/attribute/constant/ && cargo publish && cd ../../../
	sleep 25
	cd lang/attribute/error/ && cargo publish && cd ../../../
	sleep 25
	cd lang/attribute/interface/ && cargo publish && cd ../../../
	sleep 25
	cd lang/attribute/program/ && cargo publish && cd ../../..
	sleep 25
	cd lang/attribute/state/ && cargo publish && cd ../../../
	sleep 25
	cd lang/attribute/event/ && cargo publish && cd ../../../
	sleep 25
	cd lang/ && cargo publish && cd../
	sleep 25
	cd spl/ && cargo publish && cd ../
	sleep 25
	cd client/ && cargo publish && cd ../
