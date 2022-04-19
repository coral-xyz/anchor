#!/bin/bash

# Write a keypair for program deploy
mkdir -p target/deploy
cp keypairs/idl_commands_one-keypair.json target/deploy

echo "Starting local validator for test"

solana-test-validator --reset -q --mint tgyXxAhCkpgtKCEi4W6xWJSzqwVGs3uk2RodbZP2J49 &

sleep 10

echo "Building and deploying programs"

anchor build && anchor deploy

echo "Running tests"

anchor test --skip-deploy --skip-local-validator

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
