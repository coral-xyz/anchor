#!/usr/bin/env bash
set -euo pipefail

# This is the compiler-level no-std/no-alloc check requested for Anchor V2.
# The SBF dependency graph check is necessary but cannot detect an alloc type
# hidden behind an accidentally broad cfg. Compiling the runtime crates for
# upstream BPF with only core catches that API/source boundary directly.

toolchain="${V2_NO_ALLOC_TOOLCHAIN:-nightly}"
target="${V2_NO_ALLOC_TARGET:-bpfel-unknown-none}"

for package in anchor-lang anchor-spl; do
    echo "checking no-alloc BPF build: $package ($target)"
    cargo "+$toolchain" check \
        -p "$package" \
        --target "$target" \
        -Zbuild-std=core \
        --all-features
done
