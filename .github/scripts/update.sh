#!/usr/bin/env bash

solana_ver=$(cat ./.github/releases/solana-latest.txt)
solana_ver_cut=$(echo $solana_ver | cut -d'v' -f 2)
source ./.github/scripts/patch_crates.sh

update_solana_dependencies . $solana_ver_cut