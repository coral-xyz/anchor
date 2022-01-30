import BN from "bn.js";
import * as BufferLayout from "buffer-layout";
import { Layout } from "buffer-layout";
import { PublicKey } from "@solana/web3.js";
import * as utils from "../../utils";

export function uint64(property?: string): Layout<u64 | null> {
  return new WrappedLayout(
    BufferLayout.blob(8),
    (b: Buffer) => u64.fromBuffer(b),
    (n: u64) => n.toBuffer(),
    property
  );
}

export function bool(property?: string): Layout<boolean> {
  return new WrappedLayout(BufferLayout.u8(), decodeBool, encodeBool, property);
}

export function publicKey(property?: string): Layout<PublicKey> {
  return new WrappedLayout(
    BufferLayout.blob(32),
    (b: Buffer) => new PublicKey(b),
    (key: PublicKey) => key.toBuffer(),
    property
  );
}

export function coption<T>(
  layout: Layout<T>,
  property?: string
): Layout<T | null> {
  return new COptionLayout<T>(layout, property);
}

class WrappedLayout<T, U> extends Layout<U> {
  layout: Layout<T>;
  decoder: (data: T) => U;
  encoder: (src: U) => T;

  constructor(
    layout: Layout<T>,
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

export class COptionLayout<T> extends Layout<T | null> {
  layout: Layout<T>;
  discriminator: Layout<number>;

  constructor(layout: Layout<T>, property?: string) {
    super(-1, property);
    this.layout = layout;
    this.discriminator = BufferLayout.u32();
  }

  encode(src: T | null, b: Buffer, offset = 0): number {
    if (src === null || src === undefined) {
      return this.layout.span + this.discriminator.encode(0, b, offset);
    }
    this.discriminator.encode(1, b, offset);
    return this.layout.encode(src, b, offset + 4) + 4;
  }

  decode(b: Buffer, offset = 0): T | null {
    const discriminator = this.discriminator.decode(b, offset);
    if (discriminator === 0) {
      return null;
    } else if (discriminator === 1) {
      return this.layout.decode(b, offset + 4);
    }
    throw new Error("Invalid coption " + this.property);
  }

  getSpan(b: Buffer, offset = 0): number {
    return this.layout.getSpan(b, offset + 4) + 4;
  }
}

function decodeBool(value: number): boolean {
  if (value === 0) {
    return false;
  } else if (value === 1) {
    return true;
  }
  throw new Error("Invalid bool: " + value);
}

function encodeBool(value: boolean): number {
  return value ? 1 : 0;
}

export class u64 extends BN {
  /**
   * Convert to Buffer representation
   */
  toBuffer(): Buffer {
    const a = super.toArray().reverse();
    const b = Buffer.from(a);
    if (b.length === 8) {
      return b;
    }
    if (b.length >= 8) {
      throw new Error("u64 too large");
    }

    const zeroPad = Buffer.alloc(8);
    b.copy(zeroPad);
    return zeroPad;
  }

  /**
   * Construct a u64 from Buffer representation
   */
  static fromBuffer(buffer: Buffer): u64 {
    if (buffer.length !== 8) {
      throw new Error(`Invalid buffer length: ${buffer.length}`);
    }
    return new u64(
      [...buffer]
        .reverse()
        .map((i) => `00${i.toString(16)}`.slice(-2))
        .join(""),
      16
    );
  }
}
