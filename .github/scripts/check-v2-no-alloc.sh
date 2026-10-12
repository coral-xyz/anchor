#!/usr/bin/env bash
set -euo pipefail

# The proc-macro/IDL graph is deliberately host-side.  Exclude it so this
# audit describes only dependencies that can be linked into an SBF program.
# This is a graph check, not a replacement for the SBF build or runtime tests.

target="${SBF_TARGET:-sbpf-solana-solana}"
toolchain="$(rustup toolchain list 2>/dev/null | awk '$1 ~ /-sbpf-solana-/ { print $1; exit }')"
if [[ -n "$toolchain" ]]; then
    cargo_tree=(rustup run "$toolchain" cargo tree)
else
    cargo_tree=(cargo tree)
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

status=0
while IFS= read -r manifest; do
    # The equivalence suite intentionally keeps V1's alloc-heavy graph for
    # byte-for-byte comparison. This gate is only for V2 runtime crates.
    if grep -Eq 'package = "anchor-(lang|spl)-v1"' "$manifest"; then
        continue
    fi
    if ! grep -Eq 'anchor-lang|anchor-spl' "$manifest"; then
        continue
    fi

    name="$(basename "$(dirname "$manifest")")"
    tree="$tmp_dir/$name.tree"
    echo "checking no-alloc SBF graph: $manifest"
    if ! "${cargo_tree[@]}" \
        --manifest-path "$manifest" \
        --target "$target" \
        --edges features,no-dev,no-proc-macro \
        --prune anchor-derive-accounts >"$tree"; then
        echo "failed to resolve SBF graph for $manifest" >&2
        status=1
        continue
    fi

    # `anchor-derive-accounts` is a host proc-macro package. Cargo's feature
    # tree includes its host-side IDL dependencies even for an SBF target, so
    # prune that generated-code boundary before auditing the runtime closure.
    # These are either direct allocation features or known allocation-bearing
    # facades that must never enter an on-chain V2 program.
    if grep -En \
        'feature "(std|alloc)"|solana-instruction( | v)|solana-system-interface|solana-sysvar|solana-pubkey|borsh( | v)|spl-token-(interface|2022-interface|group-interface|metadata-interface)|mpl-token-metadata' \
        "$tree"; then
        echo "forbidden allocation-bearing dependency in $manifest" >&2
        status=1
    fi
done < <(find tests-v2/programs -name Cargo.toml -type f | sort)

# Metadata is explicitly host-only and is the one V2 source module allowed to
# mention alloc-backed Metaplex types.  No other runtime source may declare it.
if grep -RIn --include='*.rs' 'extern crate alloc' lang-v2/src spl-v2/src \
    | grep -v '^spl-v2/src/metadata.rs:'; then
    echo "runtime source declares alloc" >&2
    status=1
fi

exit "$status"
