import { Program } from "../index.js";
import { program as tokenProgram, SplToken } from "./token.js";
import { program as dexProgram, Dex } from "./dex.js";

export { SplToken } from "./token.js";
export { Dex } from "./dex.js";

export class Spl {
  public static token(): Program<SplToken> {
    return tokenProgram();
  }

  public static dex(): Program<Dex> {
    return dexProgram();
  }
}
