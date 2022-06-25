#!/usr/bin/env bash

cd programs/idl
anchor idl parse --file src/lib.rs > ../../tests/testdata/idl_parse_exp.json
anchor idl build > ../../tests/testdata/idl_gen_exp.json