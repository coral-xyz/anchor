import { spawn } from "child_process";

describe("testing anchor bench", () => {
    it("output the stack size of each instruction to bench_stack_size.json", async () => {
        // running the modified anchor version
        const resultSpawn = spawn("cargo", ["r", "bench"]);
    });
});