import BN from "bn.js";
import * as BufferLayout from "buffer-layout";
import { Layout } from "buffer-layout";

export class RustStringLayout extends Layout<string | null> {
  layout = BufferLayout.struct<
    Readonly<{
      length?: number;
      lengthPadding?: number;
      chars: Buffer;
    }>
  >(
    [
      BufferLayout.u32("length"),
      BufferLayout.u32("lengthPadding"),
      BufferLayout.blob(BufferLayout.offset(BufferLayout.u32(), -8), "chars"),
    ],
    this.property
  );

  constructor(public property?: string) {
    super(-1, property);
  }

  encode(src: string | null, b: Buffer, offset = 0): number {
    if (src === null || src === undefined) {
      return this.layout.span;
    }

    const data = {
      chars: Buffer.from(src, "utf8"),
    };

    return this.layout.encode(data, b, offset);
  }

  decode(b: Buffer, offset = 0): string | null {
    const data = this.layout.decode(b, offset);
    return data["chars"].toString();
  }

  getSpan(b: Buffer, offset = 0): number {
    return (
      BufferLayout.u32().span +
      BufferLayout.u32().span +
      new BN(new Uint8Array(b).slice(offset, offset + 4), 10, "le").toNumber()
    );
  }
}
