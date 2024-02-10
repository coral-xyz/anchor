#!/bin/bash

################################################################################
#
# A script to run the example as an integration test. It starts up a localnet
# and executes the current directory's rust binary.
#
# Usage:
#
# ./run.sh
#
# Run this script from within the `example/` directory in which it is located.
# The anchor cli must be installed.
#
# cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked
#
################################################################################

set -euox pipefail

# Root directories for each project
composite_root="../../tests/composite"
basic_2_root="../../examples/tutorial/basic-2"
basic_4_root="../../examples/tutorial/basic-4"
events_root="../../tests/events"
optional_root="../../tests/optional"
relations_derivation_root="../../tests/relations-derivation"

# Program IDs for each project
composite_pid="EHthziFziNoac9LBGxEaVN47Y3uUiRoXvqAiR6oes4iU"
basic_2_pid="Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"
basic_4_pid="CwrqeMj2U8tFr1Rhkgwc84tpAsqbt9pTt2a4taoTADPr"
events_pid="2dhGsWUzy5YKUsjZdLHLmkNpUDAXkNa9MYWsPc4Ziqzy"
optional_pid="FNqz6pqLAwvMSds2FYjR4nKV3moVpPNtvkfGFrqLKrgG"
relations_derivation_pid="6mogAuKLW1uiXg8Br8YwPtCTPyxKSK4YHrCcyHu6zBUY"

build_programs() {
  for project_root in $composite_root $basic_2_root $basic_4_root $events_root $optional_root $relations_derivation_root; do
    pushd "$project_root"
    anchor build
    popd
  done
}

cleanup() {
  pkill -P $$ || true
  wait || true
  rm test-validator.log || true
}

start_clean_validator() {
  cleanup

  solana-test-validator -r \
    --bpf-program $composite_pid $composite_root/target/deploy/composite.so \
    --bpf-program $basic_2_pid $basic_2_root/target/deploy/basic_2.so \
    --bpf-program $basic_4_pid $basic_4_root/target/deploy/basic_4.so \
    --bpf-program $events_pid $events_root/target/deploy/events.so \
    --bpf-program $optional_pid $optional_root/target/deploy/optional.so \
    --bpf-program $relations_derivation_pid $relations_derivation_root/target/deploy/relations_derivation.so \
    > test-validator.log &
  sleep 5

  pushd $relations_derivation_root
  anchor idl init $relations_derivation_pid \
    --provider.cluster localnet \
    --filepath target/idl/relations_derivation.json
  popd
}

run_tests() {
  local extra_cargo_flags="$1" # Flags for cargo itself
  local extra_program_flags="$2" # Flags for the test program

  cargo run $extra_cargo_flags -- \
    --composite-pid $composite_pid \
    --basic-2-pid $basic_2_pid \
    --basic-4-pid $basic_4_pid \
    --events-pid $events_pid \
    --optional-pid $optional_pid \
    --relations-derivation-pid $relations_derivation_pid \
    $extra_program_flags
}

trap_add() {
  trap_add_cmd=$1; shift || fatal "${FUNCNAME} usage error"
  for trap_add_name in "$@";
  do
    trap -- "$(
      extract_trap_cmd() { printf '%s\n' "${3:-}"; }
      eval "extract_trap_cmd $(trap -p "${trap_add_name}")"
      printf '%s\n' "${trap_add_cmd}"
      )" "${trap_add_name}" \
        || fatal "unable to add to trap ${trap_add_name}"
  done
}

main() {
  build_programs

  ######
  # Run single threaded tests
  start_clean_validator
  run_tests "" ""

  ######
  # Run multithreaded tests
  start_clean_validator
  run_tests "" "--multithreaded"

  ######
  # Run async tests
  start_clean_validator
  run_tests "--features async" "--multithreaded"
}

declare -f -t trap_add
trap_add 'cleanup' EXIT
main
