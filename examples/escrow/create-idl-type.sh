#!/usr/bin/env bash

TYPES_DIR="./tests/types"
rm -rf $TYPES_DIR
mkdir -p $TYPES_DIR
OUT_PATH="$TYPES_DIR/escrow.ts"

typename="EscrowIDL"
echo "export type $typename =" >>$OUT_PATH
cat target/idl/escrow.json >>$OUT_PATH
echo ";" >>$OUT_PATH