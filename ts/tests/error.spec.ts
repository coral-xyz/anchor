import { publicKey } from "@project-serum/borsh";
import { getProgramStackFromLogs, AnchorError } from "../src/program/logs";

describe("Error log parsing", () => {
    test("basic", () => {
        const logs = [
            "Program ERRM6YCMsccM22TEaPuu35KVU4iCY3GLCz4qMsKLYReE invoke [1]",
            "Program ERRM6YCMsccM22TEaPuu35KVU4iCY3GLCz4qMsKLYReE consumed 3797 of 200000 compute units",
            "Program ERRM6YCMsccM22TEaPuu35KVU4iCY3GLCz4qMsKLYReE failed: custom program error: 0x29"
        ];

        expect(getProgramStackFromLogs(logs).map((publicKey) => publicKey.toString())).toEqual(["ERRM6YCMsccM22TEaPuu35KVU4iCY3GLCz4qMsKLYReE"])
    })

    it("basic multiple ix", () => {
        const logs = [
            "Program 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin invoke [1]",
            "Program 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin consumed 4308 of 200000 compute units",
            "Program 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin success",
            "Program ERRM6YCMsccM22TEaPuu35KVU4iCY3GLCz4qMsKLYReE invoke [1]",
            "Program ERRM6YCMsccM22TEaPuu35KVU4iCY3GLCz4qMsKLYReE consumed 3797 of 200000 compute units",
            "Program ERRM6YCMsccM22TEaPuu35KVU4iCY3GLCz4qMsKLYReE failed: custom program error: 0x29"
        ];

        expect(getProgramStackFromLogs(logs).map((publicKey) => publicKey.toString())).toEqual(["ERRM6YCMsccM22TEaPuu35KVU4iCY3GLCz4qMsKLYReE"])
    })

    it("failed inner ix", () => {
        const logs = [
            'Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS invoke [1]',
            'Program log: Instruction: Create',
            'Program 11111111111111111111111111111111 invoke [2]',
            'Program log: panicked at programs/floats/src/lib.rs:17:9',
            'Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS consumed 12619 of 1400000 compute units',
            'Program failed to complete: BPF program panicked',
            'Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS failed: Program failed to complete'
        ];
        expect(getProgramStackFromLogs(logs).map((publicKey) => publicKey.toString())).toEqual(["Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS", "11111111111111111111111111111111"])
    })

    it("ignore successful inner ix", () => {
        const logs = [
            'Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS invoke [1]',
            'Program log: Instruction: Create',
            'Program 11111111111111111111111111111111 invoke [2]',
            'Program 11111111111111111111111111111111 success',
            'Program log: panicked at programs/floats/src/lib.rs:17:9',
            'Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS consumed 12619 of 1400000 compute units',
            'Program failed to complete: BPF program panicked',
            'Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS failed: Program failed to complete'
        ];
        expect(getProgramStackFromLogs(logs).map((publicKey) => publicKey.toString())).toEqual(["Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"])
    })

    it("ignore successful inner ix but don't ignore failing inner ix", () => {
        const logs = [
            'Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS invoke [1]',
            'Program log: Instruction: Create',
            'Program 11111111111111111111111111111111 invoke [2]',
            'Program 11111111111111111111111111111111 success',
            'Program ERRM6YCMsccM22TEaPuu35KVU4iCY3GLCz4qMsKLYReE invoke [2]',
            'Program log: panicked at programs/floats/src/lib.rs:17:9',
            'Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS consumed 12619 of 1400000 compute units',
            'Program failed to complete: BPF program panicked',
            'Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS failed: Program failed to complete'
        ];
        expect(getProgramStackFromLogs(logs).map((publicKey) => publicKey.toString())).toEqual(["Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS", "ERRM6YCMsccM22TEaPuu35KVU4iCY3GLCz4qMsKLYReE"])
    })

    it("ignore successful inner ix but don't ignore failing inner ix - big nested", () => {
        const logs = [
            'Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS invoke [1]',
            'Program log: Instruction: Create',
            'Program 11111111111111111111111111111111 invoke [2]',
            'Program 11111111111111111111111111111111 success',
            'Program 1119iqpxV28XnisGGQVMHsABdWZAx9PjtwegepRhGm5 invoke [2]',
            'Program 1119iqpxV28XnisGGQVMHsABdWZAx9PjtwegepRhGm5 consumed 4308 of 200000 compute units',
            'Program 1119iqpxV28XnisGGQVMHsABdWZAx9PjtwegepRhGm5 success',
            'Program 222fsxyjMZSSpT9gpucChbiFmjZC2GtaZmKsBkh66KMZ invoke [2]',
            'Program 333fE7qebyWBjcaCJcVmkzwrheA1Ka9bjGChuhVD9iQr invoke [3]',
            'Program 444D5MLf9UbeJBiuFw5WzVG3bMejweunZHPboWm2oTsh invoke [4]',
            'Program 444D5MLf9UbeJBiuFw5WzVG3bMejweunZHPboWm2oTsh consumed 14343 of 200000 compute units',
            'Program 444D5MLf9UbeJBiuFw5WzVG3bMejweunZHPboWm2oTsh success',
            'Program 555CBVR14jAYjK8jRE5kurBACiSNYXkffciRSG2R3krX invoke [4]',
            'Program 555CBVR14jAYjK8jRE5kurBACiSNYXkffciRSG2R3krX consumed 163337 of 200000 compute units',
            'Program 555CBVR14jAYjK8jRE5kurBACiSNYXkffciRSG2R3krX success',
            'Program 666UBGVHWNP7qNqUdnYz86owJ8oErztVvgeF5Dd5v8cR invoke [4]',
            'Program 666UBGVHWNP7qNqUdnYz86owJ8oErztVvgeF5Dd5v8cR success',
            'Program 333fE7qebyWBjcaCJcVmkzwrheA1Ka9bjGChuhVD9iQr success',
            'Program 222fsxyjMZSSpT9gpucChbiFmjZC2GtaZmKsBkh66KMZ success',
            'Program 777UGK3pU4ygVWwnn7MDnetec1nSVg4Xi53DFSHu9D6A invoke [2]',
            'Program 888E49S65VpyDmydi6juT7tsSwNyD3ZEVkV8te1rL3iH invoke [3]',
            'Program 999X95icuyGzfYoeP6SPMb8aMn6ahfCpAt9VPddSNPPi invoke [4]',
            'Program 999X95icuyGzfYoeP6SPMb8aMn6ahfCpAt9VPddSNPPi success',
            'Program ERRM6YCMsccM22TEaPuu35KVU4iCY3GLCz4qMsKLYReE invoke [4]',
            'Program log: panicked at programs/floats/src/lib.rs:17:9',
            'Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS consumed 12619 of 1400000 compute units',
            'Program failed to complete: BPF program panicked',
            'Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS failed: Program failed to complete'
        ];

        expect(getProgramStackFromLogs(logs).map((publicKey) => publicKey.toString()))
            .toEqual(["Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS", "777UGK3pU4ygVWwnn7MDnetec1nSVg4Xi53DFSHu9D6A", "888E49S65VpyDmydi6juT7tsSwNyD3ZEVkV8te1rL3iH", "ERRM6YCMsccM22TEaPuu35KVU4iCY3GLCz4qMsKLYReE"])
    })
})

describe("AnchorError", () => {
    it("FileLine AnchorError", () => {
        const logs = [
            "Program SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f invoke [1]",
            "Program log: Instruction: AggregatorSaveResult",
            "Program log: AnchorError thrown in programs/switchboard_v2/src/actions/aggregator_save_result_action.rs:235. Error Code: OracleMismatchError. Error Number: 6021. Error Message: An unexpected oracle account was provided for the transaction..",
            "Program SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f consumed 28928 of 200000 compute units",
            "Program SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f failed: custom program error: 0x1785"
        ];

        const anchorError = AnchorError.parse(logs)!;
        expect(anchorError.errorCode).toEqual({ code: "OracleMismatchError", number: 6021 });
        expect(anchorError.errorMessage).toEqual("An unexpected oracle account was provided for the transaction.");
        expect(anchorError.programStack.map((publicKey) => publicKey.toString())).toEqual(["SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f"]);
        expect(anchorError.origin).toEqual({ file: "programs/switchboard_v2/src/actions/aggregator_save_result_action.rs", line: 235 });
    })
})
