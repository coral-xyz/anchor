note: this only works on ubuntu

## Install dependencies
1. Install node, npm, and yarn
2. Install rust/ cargo 
3. Install Solana

## Getting anchor setup
In the folder right outside of this one clone: `git@github.com:Lev-Stambler/anchor.git`
Then, within that folder do the following

1. make
2. `(cd ts && yarn && yarn build)`
3. Add this to your bash rc: (`vim ~/.bashrc`)
```
custom-anchor() {
	node <PATH TO ANCHOR FORK>/cli/npm-package/anchor.js $*
}
```
4. Source bashrc
5. Try running custom-anchor

## Getting the repo up and running
1. Change into this directory
2. Run `solana key-gen`
3. In `malloc-core/Anchor.toml` replace wallet with the location of your wallet
4. In `malloc-spl/Anchor.toml` replace wallet with the location of your wallet
5. run `custom-anchor build`
6. run `cd ts-packages/malloc-sdk` and run `yarn && yarn build`
7. Then go to `malloc-spl` and run `custom-anchor build`
8. Then run `yarn && yarn install`
9. To airdrop some SOL, do `solana airdrop 1 -u d`