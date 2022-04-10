import BN from "bn.js";
import bs58 from "bs58";
import * as mockttp from "mockttp";
import * as fs from "fs/promises";
import * as path from "path";
import { Connection, Keypair, PublicKey, HttpHeaders } from "@solana/web3.js";

import { utils } from "../src";

const { BpfLoaderUpgradeable, BpfLoaderUpgradeableProgram } = utils.bpfLoader;

const airdrop = (() => {
  const mockServer: mockttp.Mockttp | undefined =
    process.env.TEST_LIVE === undefined ? mockttp.getLocal() : undefined;

  let uniqueCounter = 0;
  const uniqueSignature = () => {
    return bs58.encode(new BN(++uniqueCounter).toArray(undefined, 64));
  };

  type RpcRequest = {
    method: string;
    params?: Array<any>;
  };

  type RpcResponse = {
    context: {
      slot: number;
    };
    value: any;
  };

  const mockRpcSocket: Array<[RpcRequest, RpcResponse]> = [];

  const mockRpcMessage = ({
    method,
    params,
    result,
  }: {
    method: string;
    params: Array<any>;
    result: any;
  }) => {
    mockRpcSocket.push([
      { method, params },
      {
        context: { slot: 11 },
        value: result,
      },
    ]);
  };

  const mockRpcResponse = async ({
    method,
    params,
    value,
    error,
    withContext,
    withHeaders,
  }: {
    method: string;
    params: Array<any>;
    value?: any;
    error?: any;
    withContext?: boolean;
    withHeaders?: HttpHeaders;
  }) => {
    if (!mockServer) return;

    let result = value;
    if (withContext) {
      result = {
        context: {
          slot: 11,
        },
        value,
      };
    }

    await mockServer
      .post("/")
      .withJsonBodyIncluding({
        jsonrpc: "2.0",
        method,
        params,
      })
      .withHeaders(withHeaders || {})
      .thenReply(
        200,
        JSON.stringify({
          jsonrpc: "2.0",
          id: "",
          error,
          result,
        })
      );
  };

  return async ({
    connection,
    address,
    amount,
  }: {
    connection: Connection;
    address: PublicKey;
    amount: number;
  }) => {
    await mockRpcResponse({
      method: "requestAirdrop",
      params: [address.toBase58(), amount],
      value: uniqueSignature(),
    });

    const signature = await connection.requestAirdrop(address, amount);

    await mockRpcMessage({
      method: "signatureSubscribe",
      params: [signature, { commitment: "confirmed" }],
      result: { err: null },
    });

    await connection.confirmTransaction(signature, "confirmed");
    return signature;
  };
})();

describe("BPF Loader Upgradeable", () => {
  if (!process.env.TEST_LIVE) {
    it("No tests for non TEST_LIVE", () => {});
  } else {
    const connection = new Connection("http://127.0.0.1:8899/", "confirmed");
    let programData: Buffer;

    beforeAll(async function () {
      programData = await fs.readFile(
        path.join(__dirname, "fixtures/noop-program/solana_bpf_rust_noop.so")
      );
    });

    // create + load + authority + close
    it("Buffer lifecycle", async function () {
      const payerAccount = Keypair.generate();
      const bufferAccount = Keypair.generate();
      const authorityAccount = Keypair.generate();
      const authorityAccount2 = Keypair.generate();

      const { feeCalculator } = await connection.getRecentBlockhash();
      const fees =
        feeCalculator.lamportsPerSignature *
        // createAccount
        (2 +
          // loadBuffer
          Math.ceil(
            programData.length / BpfLoaderUpgradeable.WRITE_CHUNK_SIZE
          ) *
            2 +
          // setBufferAuthority
          2 +
          // closeBuffer
          2 * 2);
      const payerBalance = await connection.getMinimumBalanceForRentExemption(
        0
      );
      const bufferAccountSize = BpfLoaderUpgradeable.getBufferAccountSize(
        programData.length
      );
      const bufferAccountBalance =
        await connection.getMinimumBalanceForRentExemption(bufferAccountSize);

      await airdrop({
        connection,
        address: payerAccount.publicKey,
        amount: payerBalance + bufferAccountBalance + fees,
      });

      await BpfLoaderUpgradeable.createBuffer(
        connection,
        payerAccount,
        bufferAccount,
        authorityAccount.publicKey,
        bufferAccountBalance,
        programData.length
      );

      await BpfLoaderUpgradeable.loadBuffer(
        connection,
        payerAccount,
        bufferAccount.publicKey,
        authorityAccount,
        programData
      );

      await BpfLoaderUpgradeable.setBufferAuthority(
        connection,
        payerAccount,
        bufferAccount.publicKey,
        authorityAccount,
        authorityAccount2.publicKey
      );

      expect(
        BpfLoaderUpgradeable.closeBuffer(
          connection,
          payerAccount,
          bufferAccount.publicKey,
          authorityAccount,
          payerAccount.publicKey
        )
      ).rejects.toThrow();

      await BpfLoaderUpgradeable.closeBuffer(
        connection,
        payerAccount,
        bufferAccount.publicKey,
        authorityAccount2,
        payerAccount.publicKey
      );

      expect(
        await connection.getAccountInfo(bufferAccount.publicKey)
      ).toBeNull();
    }, 120_000);

    // create buffer + write buffer + deploy + upgrade + set authority + close
    it("Program lifecycle", async function () {
      const payerAccount = Keypair.generate();
      const bufferAccount = Keypair.generate();
      const bufferAuthorityAccount = Keypair.generate();
      const programAccount = Keypair.generate();
      const programAuthorityAccount = Keypair.generate();

      const { feeCalculator } = await connection.getRecentBlockhash();
      const fees =
        feeCalculator.lamportsPerSignature *
        // createAccount
        (2 +
          // loadBuffer
          Math.ceil(
            programData.length / BpfLoaderUpgradeable.WRITE_CHUNK_SIZE
          ) *
            2 +
          // deployProgram
          3 +
          // setProgramAuthority
          2 +
          // closeProgram
          2 * 2);
      const payerBalance = await connection.getMinimumBalanceForRentExemption(
        0
      );
      const bufferAccountSize = BpfLoaderUpgradeable.getBufferAccountSize(
        programData.length
      );
      const bufferAccountBalance =
        await connection.getMinimumBalanceForRentExemption(bufferAccountSize);
      const programAccountSize = BpfLoaderUpgradeable.getBufferAccountSize(
        BpfLoaderUpgradeable.BUFFER_PROGRAM_SIZE
      );
      const programAccountBalance =
        await connection.getMinimumBalanceForRentExemption(programAccountSize);
      const programDataAccountSize =
        BpfLoaderUpgradeable.BUFFER_PROGRAM_DATA_HEADER_SIZE +
        programData.length * 2;
      const programDataAccountBalance =
        await connection.getMinimumBalanceForRentExemption(
          programDataAccountSize
        );

      await airdrop({
        connection,
        address: payerAccount.publicKey,
        amount:
          payerBalance +
          bufferAccountBalance * 2 +
          programAccountSize +
          programDataAccountBalance +
          fees,
      });

      await BpfLoaderUpgradeable.createBuffer(
        connection,
        payerAccount,
        bufferAccount,
        bufferAuthorityAccount.publicKey,
        bufferAccountBalance,
        programData.length
      );

      await BpfLoaderUpgradeable.loadBuffer(
        connection,
        payerAccount,
        bufferAccount.publicKey,
        bufferAuthorityAccount,
        programData
      );

      await BpfLoaderUpgradeable.deployProgram(
        connection,
        payerAccount,
        bufferAccount.publicKey,
        bufferAuthorityAccount,
        programAccount,
        programAccountBalance,
        programData.length * 2
      );

      expect(
        await connection.getAccountInfo(bufferAccount.publicKey)
      ).toBeNull();

      // Upgrade
      await BpfLoaderUpgradeable.createBuffer(
        connection,
        payerAccount,
        bufferAccount,
        bufferAuthorityAccount.publicKey,
        bufferAccountBalance,
        programData.length
      );

      await BpfLoaderUpgradeable.loadBuffer(
        connection,
        payerAccount,
        bufferAccount.publicKey,
        bufferAuthorityAccount,
        programData
      );

      await BpfLoaderUpgradeable.upgradeProgram(
        connection,
        payerAccount,
        programAccount.publicKey,
        bufferAuthorityAccount,
        bufferAccount.publicKey,
        payerAccount.publicKey
      );

      expect(
        await connection.getAccountInfo(bufferAccount.publicKey)
      ).toBeNull();

      // failed close + set authority + close
      expect(
        BpfLoaderUpgradeable.closeProgram(
          connection,
          payerAccount,
          programAccount.publicKey,
          programAuthorityAccount,
          payerAccount.publicKey
        )
      ).rejects.toThrow();

      await BpfLoaderUpgradeable.setProgramAuthority(
        connection,
        payerAccount,
        programAccount.publicKey,
        bufferAuthorityAccount,
        programAuthorityAccount.publicKey
      );

      await BpfLoaderUpgradeable.closeProgram(
        connection,
        payerAccount,
        programAccount.publicKey,
        programAuthorityAccount,
        payerAccount.publicKey
      );

      const programDataAccount =
        await BpfLoaderUpgradeableProgram.getProgramDataAddress(
          programAccount.publicKey
        );
      expect(await connection.getAccountInfo(programDataAccount)).toBeNull();
    }, 120_000);
  }
});
