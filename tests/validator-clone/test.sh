#!/bin/bash

echo "Starting local validator for test"

solana-test-validator \
  --reset \
  --rpc-port 8900 \
  --faucet-port 9901 \
  --account 8DKwAVrCEVStDYNPCsmxHtUj8LH9oXNtkVRrBfpNKvhp accounts/8DKwAVrCEVStDYNPCsmxHtUj8LH9oXNtkVRrBfpNKvhp.json \
  --account AqH29mZfQFgRpfwaPoTMWSKJ5kqauoc1FwVBRksZyQrt accounts/AqH29mZfQFgRpfwaPoTMWSKJ5kqauoc1FwVBRksZyQrt.json \
  --account MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr accounts/MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr.json \
  --account metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s accounts/metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s.json \
  --account mv3ekLzLbnVPNxjSKvqBpU3ZeZXPQdEC3bp5MDEBG68 accounts/mv3ekLzLbnVPNxjSKvqBpU3ZeZXPQdEC3bp5MDEBG68.json \
  --account So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo accounts/So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo.json \
  --account PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT accounts/PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT.json \
  --account DMCvGv1fS5rMcAvEDPDDBawPqbDRSzJh2Bo6qXCmgJkR accounts/DMCvGv1fS5rMcAvEDPDDBawPqbDRSzJh2Bo6qXCmgJkR.json -q &

sleep 10

echo "Validator started, running tests"

anchor test
