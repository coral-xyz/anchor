import { PublicKey } from "@solana/web3.js";

interface ErrorCode {
    code: string,
    number: number
}

interface FileLine {
    file: string,
    line: number
}

type Origin = string | FileLine;
type ComparedValues = [string, string] | [PublicKey, PublicKey];

export class AnchorError extends Error {
    constructor(readonly errorCode: ErrorCode, readonly errorMessage: string, readonly errorLogs: string[], readonly programStack: string[], readonly origin?: Origin, readonly comparedValues?: ComparedValues) {
        super(errorLogs.join("\n"));
    }

    public static parse(logs: string[]) {
        const programStack = getProgramStackFromLogs(logs);

        const anchorErrorLogIndex = logs.findIndex((log) => log.startsWith("Program log: AnchorError"));
        if (anchorErrorLogIndex === -1) {
            return null;
        };
        const anchorErrorLog = logs[anchorErrorLogIndex];
        let comparedValues: ComparedValues | undefined;
        if (anchorErrorLogIndex + 1 < logs.length) {
            if (logs[anchorErrorLogIndex + 1] === "Program log: Left:") {
                const pubkeyRegex = /^Program log: (.*)$/;
                const leftPubkey = pubkeyRegex.exec(logs[anchorErrorLogIndex + 2])![1];
                const rightPubkey = pubkeyRegex.exec(logs[anchorErrorLogIndex + 4])![1];
                comparedValues = [new PublicKey(leftPubkey), new PublicKey(rightPubkey)];
            } else if (logs[anchorErrorLogIndex + 1].startsWith("Program log: Left:")) {
                const valueRegex = /^Program log: Left: (.*)$/;
                const leftValue = valueRegex.exec(logs[anchorErrorLogIndex + 1])![1];
                const rightValue = valueRegex.exec(logs[anchorErrorLogIndex + 2])![1];
                comparedValues = [leftValue, rightValue];
            }
        }
        const regexNoInfo = /^Program log: AnchorError occurred\. Error Code: (.*)\. Error Number: (\d*)\. Error Message: (.*)\./;
        const noInfoAnchorErrorLog = regexNoInfo.exec(anchorErrorLog);
        const regexFileLine = /^Program log: AnchorError thrown in (.*):(\d*)\. Error Code: (.*)\. Error Number: (\d*)\. Error Message: (.*)\./;
        const fileLineAnchorErrorLog = regexFileLine.exec(anchorErrorLog);
        const regexAccountName = /^Program log: AnchorError caused by account: (.*)\. Error Code: (.*)\. Error Number: (\d*)\. Error Message: (.*)\./;
        const accountNameAnchorErrorLog = regexAccountName.exec(anchorErrorLog);
        if (noInfoAnchorErrorLog) {
            const [errorCodeString, errorNumber, errorMessage] = noInfoAnchorErrorLog.slice(1,4);
            const errorCode = { code: errorCodeString, number: parseInt(errorNumber) };
            return new AnchorError(errorCode, errorMessage, [anchorErrorLog], programStack, undefined, comparedValues);
        } else if (fileLineAnchorErrorLog) {
            const [file, line, errorCodeString, errorNumber, errorMessage] = fileLineAnchorErrorLog.slice(1,6);
            const errorCode = { code: errorCodeString, number: parseInt(errorNumber) };
            const fileLine = { file, line: parseInt(line) };
            return new AnchorError(errorCode, errorMessage, [anchorErrorLog], programStack, fileLine, comparedValues);
        } else if (accountNameAnchorErrorLog) {
            const [accountName, errorCodeString, errorNumber, errorMessage] = accountNameAnchorErrorLog.slice(1,5);
            const origin = accountName;
            const errorCode = { code: errorCodeString, number: parseInt(errorNumber)};
            return new AnchorError(errorCode, errorMessage, [anchorErrorLog], programStack, origin, comparedValues);
        } else {
            return null;
        }
    }

    // TODO: should return a `Pubkey`
    get program(): string {
        return this.programStack[this.programStack.length - 1];
    }
}

// TODO: dont export this?
// TODO: should return Pubkeys
export function getProgramStackFromLogs(logs: string[]) {
    const programKeyRegex = /^Program (\w*) invoke/;
    const successRegex = /^Program \w* success/;

    const programStack: string[] = [];
    for (let i = 0; i < logs.length; i++) {
        if (successRegex.exec(logs[i])) {
            programStack.pop();
            continue;
        }

        const programKey = programKeyRegex.exec(logs[i])?.[1];
        if (!programKey) {
            continue;
        }
        programStack.push(programKey);
    }
    return programStack;
}
