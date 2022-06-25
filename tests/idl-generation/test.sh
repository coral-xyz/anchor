#!/usr/bin/env bash
set -x
set -e

TMPDIR=$(mktemp -d)

cd programs/idl
anchor idl parse --file src/lib.rs > $TMPDIR/idl_parse_act.json
anchor idl build > $TMPDIR/idl_gen_act.json

cd ../..
echo "----------------------------------------------------"
echo "idl parse before > after"
echo "----------------------------------------------------"
echo ""
diff -y --color tests/testdata/idl_parse_exp.json $TMPDIR/idl_parse_act.json
PARSE_RETCODE=$?

echo ""
echo ""
echo "----------------------------------------------------"
echo "idl build before > after"
echo "----------------------------------------------------"
echo ""
diff -y --color tests/testdata/idl_gen_exp.json $TMPDIR/idl_gen_act.json
GEN_RETCODE=$?

# returns 0 when ok, 1 or 2 when outputs differ
exit $((PARSE_RETCODE+GEN_RETCODE))
