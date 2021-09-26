const fs = require('fs')
import camelcase from "camelcase";

fs.rmdirSync("tests/types", { recursive: true });
fs.mkdir("tests/types", { recursive: true }, (err) => {
    if (err) {
        throw err;
    }
});

const escrowIdlJson = JSON.parse(fs.readFileSync('./target/idl/escrow.json'));
for (let account of escrowIdlJson.accounts) {
    account.name = camelcase(account.name);
}

const fileContents = `export type EscrowIdl = ${JSON.stringify(escrowIdlJson)};`;
fs.writeFileSync("tests/types/escrow.ts", fileContents);
