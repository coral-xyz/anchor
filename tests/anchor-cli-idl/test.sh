#!/bin/bash

# Write a keypair for program deploy
mkdir -p target/deploy
cp keypairs/idl_commands_one-keypair.json target/deploy

echo "Starting local validator for test"

solana-test-validator --reset -q --mint 65YSPtxtGVG21kq91FC7uLsrsVV2tNzACM2uzYwpLUcF &

sleep 10

echo "Building and deploying programs"

anchor build && anchor deploy -p idl-commands-one

echo "Running tests"

anchor test --skip-deploy --skip-local-validator

# trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
