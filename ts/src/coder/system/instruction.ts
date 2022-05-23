import * as BufferLayout from "buffer-layout";
import camelCase from "camelcase";
import { InstructionCoder } from "../index.js";
import { Idl } from "../../idl.js";
import { PublicKey } from "@solana/web3.js";

export class SystemInstructionCoder implements InstructionCoder {
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  constructor(_: Idl) {}

  encode(ixName: string, ix: any): Buffer {
    switch (camelCase(ixName)) {
      case 'createAccount': {
        return encodeCreateAccount(ix);
      }
      case 'assign': {
        return encodeAssign(ix);
      }
      case 'transfer': {
        return encodeTransfer(ix);
      }
      case 'createAccountWithSeed': {
        return encodeCreateAccountWithSeed(ix);
      }
      case 'advanceNonceAccount': {
        return encodeAdvanceNonceAccount(ix);
      }
      case 'withdrawNonceAccount': {
        return encodeWithdrawNonceAccount(ix);
      }
      case 'authorizeNonceAccount': {
        return encodeAuthorizeNonceAccount(ix);
      }
      case 'allocate': {
        return encodeAllocate(ix);
      }
      case 'allocateWithSeed': {
        return encodeAllocateWithSeed(ix);
      }
      case 'assignWithSeed': {
        return encodeAssignWithSeed(ix);
      }
      case 'transferWithSeed': {
        return encodeTransferWithSeed(ix);
      }
      default: {
        throw new Error(`Invalid instruction: ${ixName}`);
      }
    }
  }

  encodeState(_ixName: string, _ix: any): Buffer {
    throw new Error('System does not have state');
  }
}

function encodeCreateAccount({ lamports, space, owner }: any): Buffer {
  return encodeData({
    createAccount: { lamports, space, owner },
  });
}

function encodeAssign({ owner }: any): Buffer {
  return encodeData({
    assign: { owner },
  });
}

function encodeTransfer({ lamports }: any): Buffer {
  return encodeData({
    transfer: { lamports },
  });
}

function encodeCreateAccountWithSeed({
  base,
  seed,
  lamports,
  space,
  owner,
}: any): Buffer {
  return encodeData({
    createAccountWithSeed: { base, seed, lamports, space, owner },
  });
}

function encodeAdvanceNonceAccount(_ix: any): Buffer {
  return encodeData({
    advanceNonceAccount: {},
  });
}

function encodeWithdrawNonceAccount({ arg }: any): Buffer {
  return encodeData({
    withdrawNonceAccount: { arg },
  });
}

function encodeAuthorizeNonceAccount({ arg }: any): Buffer {
  return encodeData({
    authorizeNonceAccount: { arg },
  });
}

function encodeAllocate({ space }: any): Buffer {
  return encodeData({
    allocate: { space },
  });
}

function encodeAllocateWithSeed({ base, seed, space, owner }: any): Buffer {
  return encodeData({
    allocateWithSeed: { base, seed, space, owner },
  });
}

function encodeAssignWithSeed({ base, seed, owner }: any): Buffer {
  return encodeData({
    assignWithSeed: { base, seed, owner },
  });
}

function encodeTransferWithSeed({
  lamports,
  fromSeed,
  fromOwner,
}: any): Buffer {
  return encodeData({
    transferWithSeed: { lamports, fromSeed, fromOwner },
  });
}


class WrappedLayout<T, U> extends BufferLayout.Layout<U> {
  layout: BufferLayout.Layout<T>;
  decoder: (data: T) => U;
  encoder: (src: U) => T;

  constructor(
    layout: BufferLayout.Layout<T>,
    decoder: (data: T) => U,
    encoder: (src: U) => T,
    property?: string
  ) {
    super(layout.span, property);
    this.layout = layout;
    this.decoder = decoder;
    this.encoder = encoder;
  }

  decode(b: Buffer, offset?: number): U {
    return this.decoder(this.layout.decode(b, offset));
  }

  encode(src: U, b: Buffer, offset?: number): number {
    return this.layout.encode(this.encoder(src), b, offset);
  }

  getSpan(b: Buffer, offset?: number): number {
    return this.layout.getSpan(b, offset);
  }
}

function publicKey(property?: string): BufferLayout.Layout<PublicKey> {
  return new WrappedLayout(
    BufferLayout.blob(32),
    (b: Buffer) => new PublicKey(b),
    (key: PublicKey) => key.toBuffer(),
    property
  );
}

const LAYOUT = BufferLayout.union(BufferLayout.u8('instruction'));
LAYOUT.addVariant(
  0,
  BufferLayout.struct([
    BufferLayout.nu64('lamports'),
    BufferLayout.nu64('space'),
    publicKey('owner'),
  ]),
  'createAccount'
);
LAYOUT.addVariant(
  1, 
  BufferLayout.struct([publicKey('owner')]), 
  'assign'
);
LAYOUT.addVariant(
  2,
  BufferLayout.struct([BufferLayout.nu64('lamports')]),
  'transfer'
);
LAYOUT.addVariant(
  3,
  BufferLayout.struct([
    publicKey('base'),
    BufferLayout.cstr('seed'), // I'm not sure if this would work
    BufferLayout.nu64('lamports'),
    BufferLayout.nu64('space'),
    publicKey('owner')
  ]),
  'createAccountWithSeed'
);
LAYOUT.addVariant(
  4,
  BufferLayout.struct([]),
  'advanceNonceAccount'
);
LAYOUT.addVariant(
  5, 
  BufferLayout.struct([BufferLayout.nu64('arg')]), 
  'withdrawNonceAccount'
);
LAYOUT.addVariant(
  6,
  BufferLayout.struct([
    publicKey('arg'),
  ]),
  'authorizeNonceAccount'
);
LAYOUT.addVariant(
  7,
  BufferLayout.struct([BufferLayout.nu64('space')]),
  'allocate'
);
LAYOUT.addVariant(
  8,
  BufferLayout.struct([
    publicKey('base'),
    BufferLayout.cstr('seed'),
    BufferLayout.nu64('space'),
    publicKey('owner'),
  ]),
  'allocateWithSeed'
);
LAYOUT.addVariant(
  9, 
  BufferLayout.struct([
    publicKey('base'),
    BufferLayout.cstr('seed'),
    publicKey('owner')
  ]), 
  'assignWithSeed'
);
LAYOUT.addVariant(
  10, 
  BufferLayout.struct([
    BufferLayout.nu64('space'),
    BufferLayout.cstr('fromSeed'),
    publicKey('fromOwner'),
  ]), 
  'transferWithSeed'
);

/* function publicKey(property: string): any {
  return BufferLayout.blob(32, property);
}
 */

function encodeData(instruction: any): Buffer {
  const b = Buffer.alloc(instructionMaxSpan);
  const span = LAYOUT.encode(instruction, b);
  return b.slice(0, span);
}

const instructionMaxSpan = Math.max(
  ...Object.values(LAYOUT.registry).map((r: any) => r.span)
);
