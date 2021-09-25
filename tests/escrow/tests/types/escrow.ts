export type EscrowIDL = {
  version: "0.0.0";
  name: "escrow";
  instructions: [
    {
      name: "initializeEscrow";
      accounts: [
        { name: "initializer"; isMut: false; isSigner: true },
        {
          name: "initializerDepositTokenAccount";
          isMut: true;
          isSigner: false;
        },
        {
          name: "initializerReceiveTokenAccount";
          isMut: false;
          isSigner: false;
        },
        { name: "escrowAccount"; isMut: true; isSigner: false },
        { name: "tokenProgram"; isMut: false; isSigner: false },
        { name: "rent"; isMut: false; isSigner: false }
      ];
      args: [
        { name: "initializerAmount"; type: "u64" },
        { name: "takerAmount"; type: "u64" }
      ];
    },
    {
      name: "cancelEscrow";
      accounts: [
        { name: "initializer"; isMut: false; isSigner: false },
        { name: "pdaDepositTokenAccount"; isMut: true; isSigner: false },
        { name: "pdaAccount"; isMut: false; isSigner: false },
        { name: "escrowAccount"; isMut: true; isSigner: false },
        { name: "tokenProgram"; isMut: false; isSigner: false }
      ];
      args: [];
    },
    {
      name: "exchange";
      accounts: [
        { name: "taker"; isMut: false; isSigner: true },
        { name: "takerDepositTokenAccount"; isMut: true; isSigner: false },
        { name: "takerReceiveTokenAccount"; isMut: true; isSigner: false },
        { name: "pdaDepositTokenAccount"; isMut: true; isSigner: false },
        {
          name: "initializerReceiveTokenAccount";
          isMut: true;
          isSigner: false;
        },
        { name: "initializerMainAccount"; isMut: true; isSigner: false },
        { name: "escrowAccount"; isMut: true; isSigner: false },
        { name: "pdaAccount"; isMut: false; isSigner: false },
        { name: "tokenProgram"; isMut: false; isSigner: false }
      ];
      args: [];
    }
  ];
  accounts: [
    {
      name: "escrowAccount";
      type: {
        kind: "struct";
        fields: [
          { name: "initializerKey"; type: "publicKey" },
          { name: "initializerDepositTokenAccount"; type: "publicKey" },
          { name: "initializerReceiveTokenAccount"; type: "publicKey" },
          { name: "initializerAmount"; type: "u64" },
          { name: "takerAmount"; type: "u64" }
        ];
      };
    }
  ];
  metadata: { address: "orwyukmpT9ZKxq78aQCkM75xJDgV1VXgAktFgX6PZob" };
};
