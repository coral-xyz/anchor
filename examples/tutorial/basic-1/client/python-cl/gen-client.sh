#!/bin/bash
set -e

cd "$(dirname "$(readlink -f "$0")")"

poetry install --no-root
poetry shell

IDL_PATH="$(dirname "$(dirname "$(dirname "$(readlink -f "$0")")")")/target/idl/basic_1.json"
OUT_PATH="$(dirname "$(readlink -f "$0")")/gen"
PROGRAM_ID_PATH="$(dirname "$(readlink -f "$0")")/program_id.txt"

if [ ! -e $IDL_PATH ]; then
    echo "===error==="
    echo "$IDL_PATH does not exist."
    exit 1
fi

if [ ! -d $OUT_PATH ]; then
    echo "===error==="
    echo "$OUT_PATH does not exist."
    exit 1
fi

if [ ! -e $PROGRAM_ID_PATH ]; then
    echo "===error==="
    echo "$PROGRAM_ID_PATH does not exist."
    exit 1
fi

PROGRAM_ID=$(< $PROGRAM_ID_PATH)
anchorpy client-gen --program-id $PROGRAM_ID $IDL_PATH $OUT_PATH