#!/bin/bash

set -e

steps=(step0 step1 step2 step3)
step0=(anchor-syn)
step1=(
  anchor-attribute-access-control
  anchor-attribute-account
  anchor-attribute-constant
  anchor-attribute-error
  anchor-attribute-event
  anchor-attribute-interface
  anchor-attribute-program
  anchor-attribute-state
  anchor-derive-accounts
)
step2=(anchor-lang)
step3=(
  anchor-spl
  anchor-client
  anchor-cli
)

for stepName in "${steps[@]}"; do
  declare -n step="$stepName"
  pids=()
  for prog in "${step[@]}"; do
    cargo publish -p "$prog" &
    pids+=($!)
  done
  for pid in "${pids[@]}"; do
    wait "$pid"
  done
done
