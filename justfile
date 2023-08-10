setup-for-tests:
	solana-install init 1.16.0 && \
	git submodule update --init --recursive --depth 1 && \
	cd ts/packages/borsh && yarn --frozen-lockfile && yarn build && yarn link --force && cd ../../../ && \
	cd ts/packages/anchor && yarn --frozen-lockfile && yarn build:node && yarn link && cd ../../../ && \
	cd ts/packages/spl-associated-token-account && yarn --frozen-lockfile && yarn build:node && yarn link && cd ../../../ && \
	cd ts/packages/spl-token && yarn --frozen-lockfile && yarn build:node && yarn link && cd ../../../ && \
	cd examples/tutorial && yarn link @coral-xyz/anchor @coral-xyz/borsh && yarn --frozen-lockfile && cd ../../ && \
	cd tests && yarn link @coral-xyz/anchor @coral-xyz/borsh @coral-xyz/spl-associated-token-account @coral-xyz/spl-token && yarn --frozen-lockfile && cd .. && \
	cargo install --path cli anchor-cli --locked --force --debug
	
