---
title: Publishing Source
description: Anchor - Publishing Source
---

The Anchor Program Registry at [apr.dev](https://apr.dev)
hosts a catalog of verified programs on Solana both written with and without Anchor. It is recommended
that authors of smart contracts publish their source to promote best
practices for security and transparency.

---

{% callout title="Note" %}
The Anchor Program Registry is currently in alpha testing. For access to publishing
please ask on [Discord](http://discord.gg/ZCHmqvXgDw).
{% /callout %}

## Getting Started

The process for publishing is mostly identical to `crates.io`.

- Signup for an account [here](https://apr.dev).
- Navigate to your Profile on the top navbar.
- Click "Generate New Access Token".
- Run `anchor login <token>` at the command line.

And you're ready to interact with the registry.

## Configuring a Build

Whether your program is written in Anchor or not, all source being published must
have an `Anchor.toml` to define the build.

An example `Anchor.toml` config looks as follows,

```toml
anchor_version = "0.25.0"

[workspace]
members = ["programs/multisig"]

[provider]
cluster = "mainnet"
wallet = "~/.config/solana/id.json"

[programs.mainnet]
multisig = "A9HAbnCwoD6f2NkZobKFf6buJoN9gUVVvX5PoUnDHS6u"

[programs.localnet]
multisig = "A9HAbnCwoD6f2NkZobKFf6buJoN9gUVVvX5PoUnDHS6u"
```

Here there are four sections.

1. `anchor_version` (optional) - sets the anchor docker image to use. By default, the builder will use the latest version of Anchor.
2. `[workspace]` (optional) - sets the paths--relative to the `Anchor.toml`--
   to all programs in the local
   workspace, i.e., the path to the `Cargo.toml` manifest associated with each
   program that can be compiled by the `anchor` CLI. For programs using the
   standard Anchor workflow, this can be ommitted. For programs not written in Anchor
   but still want to publish, this should be added.
3. `[provider]` - configures the wallet and cluster settings. Here, `mainnet` is used because the registry only supports `mainnet` binary verification at the moment.
4. `[programs.mainnet]` - configures each program in the workpace, providing
   the `address` of the program to verify.

{% callout title="Note" %}
When defining program in `[programs.mainnet]`, make sure the name provided
matches the **lib** name for your program, which is defined
by your program's Cargo.toml.
{% /callout %}

### Examples

#### Anchor Program

An example of a toml file for an Anchor program can be found [here](https://www.apr.dev/program/22Y43yTVxuUkoRKdm9thyRhQ3SdgQS7c7kB6UNCiaczD/build/2).

#### Non Anchor Program

An example of a toml file for a non-anchor program can be found [here](https://www.apr.dev/program/9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin/build/1).

## Publishing

To publish to the Anchor Program Registry, change directories to the `Anchor.toml`
defined root and run

```shell
anchor publish <program-name>
```

where `<program-name>` is as defined in `[programs.mainnet]`, i.e., `multisig`
in the example above.
