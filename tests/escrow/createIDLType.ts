const fs = require('fs')
import camelcase from "camelcase";

fs.rmdirSync("tests/types", { recursive: true });
fs.mkdir("tests/types", { recursive: true }, (err) => {
    if (err) {
        throw err;
    }
});

let escrowIDLJSON = JSON.parse(fs.readFileSync('./target/idl/escrow.json'));
for (let account of escrowIDLJSON.accounts) {
    account.name = camelcase(account.name);
}

const fileContents = `export type EscrowIDL = ${JSON.stringify(escrowIDLJSON)};`;
fs.writeFileSync("tests/types/escrow.ts", fileContents);
