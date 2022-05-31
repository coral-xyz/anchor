import { Program, Provider } from "../index.js";
import { program as serumDexProgram, SerumDex } from "./dex.js";

export { SerumDex } from "./dex.js";

export class Serum {
  public static dex(provider?: Provider): Program<SerumDex> {
    return serumDexProgram(provider);
  }
}
