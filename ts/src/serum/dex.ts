export type SerumDex = {
  version: "0.5.4";
  name: "serum_dex";
  instructions: [
    {
      name: "initializeMarket";
      accounts: [
        {
          name: "market";
          isMut: true;
          isSigner: false;
        },
        {
          name: "requestQueue";
          isMut: true;
          isSigner: false;
        },
        {
          name: "eventQueue";
          isMut: true;
          isSigner: false;
        },
        {
          name: "bids";
          isMut: true;
          isSigner: false;
        },
        {
          name: "asks";
          isMut: true;
          isSigner: false;
        },
        {
          name: "coinVault";
          isMut: true;
          isSigner: false;
        },
        {
          name: "pcVault";
          isMut: true;
          isSigner: false;
        },
        {
          name: "coinMint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "pcMint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "rent";
          isMut: false;
          isSigner: false;
        },
        {
          name: "marketAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "pruneAuthority";
          isMut: false;
          isSigner: false;
        },
        {
          name: "crankAuthority";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "coinLotSize";
          type: "u64";
        },
        {
          name: "pcLotSize";
          type: "u64";
        },
        {
          name: "feeRateBps";
          type: "u16";
        },
        {
          name: "vaultSignerNonce";
          type: "u64";
        },
        {
          name: "pcDustThreshold";
          type: "u64";
        }
      ];
    }
  ];
};
