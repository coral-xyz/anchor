# Building Release Packages

In order to make your Anchor program usable to another on-chain program, you have to release a [Rust Crate](https://crates.io/) and an [NPM Package](https://docs.npmjs.com/)

## Creating the Crate

### 1. Create a Crates IO account

You must register with [Crates](https://crates.io/) and create an access token.

### 2. Build and deploy the crate

1. Once you get an access token, login with: `cargo login <ACCESS_TOKEN>`
1. Edit the `Cargo.toml` with your relevant project information
1. To test publish the package, `cargo publish --dry-run`. Remove the `--dry-run` flag to publish.

### 3. Including your package in other Programs

Inside the Cargo.toml, add your package under the `[dependencies]`.

For example, all Anchor projects require Anchor as: `anchor-lang = "X.X.X"`

::: details
You will sometimes need to include your crate with additional features such as `cpi`.
You can include them using: `some-crate = { version = "X.X.X", features = ["cpi"] }`
:::

## Creating the NPM Package

### 1. Create an NPM account

You must register with [NPM](https://www.npmjs.com/) and a paid account is required for private packages.

### 2. Build and deploy the package

You can find the full instructions to build and deploy an npm package [here](https://docs.npmjs.com/creating-and-publishing-scoped-public-packages).

#### Include the IDL in your package

The most important part of creating your package is ensuring that your entrypoint, ie `index.js` includes your Program IDL.

During an `anchor build`, a program idl (a JSON specification for your Program) is generated and put into the `project_root/.target/idl/your_program.json`.
You have to copy it to another folder that's included in your git repository. For example: `cp ./target/idl/your_program.json ./app`

Your `index.js` file can be as simple as:
```javascript
const PROGRAM_IDL = require("./app/your_program.json");
const PROGRAM_ID = "GxJJd3q28eUd7kpPCbNXGeixqHmBYJ2owqUYqse3ZrGS";

module.exports = { PROGRAM_IDL, PROGRAM_ID }
```

### 3. Including your package in other Clients

Now you can add your program to a new client by installing your package through yarn or npm, ie `yarn add your_program`

An example import could look like:
```javascript
import { PROGRAM_IDL, PROGRAM_ID } from '@your-name/your-project';

...javascript
const yourProgram = new anchor.Program(PROGRAM_IDL, PROGRAM_ID, provider);
```

## Running another Anchor Program locally

Sometimes during development, it's easier if you can test your Program's CPIs locally rather than depend on devnet. This is especially useful when you're creating a Program intended to be used by others.The `SystemProgram`, `TokenProgram`, `Rent` are included automatically but for other Programs, you'll have to use a separate approach.

### 1. Using the Solana Test Validator Clone

Use `solana-test-validator` to run the local network. This will generate files and is commonly run internally in the `root/.anchor` directory.

You can use the `--clone` argument to copy a program locally. For example: `solana-test-validator --clone Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS --url https://api.devnet.solana.com`

### 2. Building from Source

If you're working with multiple repositorities locally, you can share the same solana test validator.

You can use `anchor build && anchor deploy` to deploy the CPI Anchor Program from a repository.

And in another repository run `anchor test --skip-local-validator` or add the `--skip-deploy` if no Program changes were made.