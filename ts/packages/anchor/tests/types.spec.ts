import { IdlTypes } from "../src/program/namespace/types";

type IdlWithEnum = {
  version: "0.1.0";
  name: "blockchain";
  instructions: [];
  types: [
    {
      name: "Currency";
      type: {
        kind: "enum";
        variants: [
          {
            name: "Sol";
          },
          {
            name: "Usdc";
          }
        ];
      };
    }
  ];
};

describe("types", () => {
  describe("IdlTypes", () => {
    it("covers enum with only name", () => {
      type Currency = IdlTypes<IdlWithEnum>["Currency"];
      typeCheck<
        Currency,
        {
          sol?: { sol: {} };
          usdc?: { usdc: {} };
        }
      >("ok");

      expect(1).toEqual(1);
    });
  });
});

export type FunctionParameters = unknown[];

type IsAny<T, Yes, No> = 0 extends 1 & T ? Yes : No;
type FunctionType<R = any, P extends FunctionParameters = any[]> = (
  ...args: P
) => R;
type IfElse<T, R, Yes> = IsAny<
  T,
  // (1) return Yes if R is any and T otherwise
  IsAny<R, Yes, T>,
  // (2) return T if Result is any and Yes otherwise
  IsAny<R, T, Yes>
>;
type EqualReformat<T> = T extends FunctionType
  ? [
      "$$FunctionType$$",
      EqualReformat<ReturnType<T>>,
      EqualReformat<Parameters<T>>
    ]
  : T extends object
  ? {
      [K in keyof T]: EqualReformat<T[K]>;
    }
  : // never is an error, so if any we want to throw an error
    IsAny<T, never, T>;
type IfEqual<T1, T2, Yes, No> = [T2] extends [T1]
  ? [T1] extends [T2]
    ? Yes
    : No
  : No;

type IfDeepEqual<T, R, Yes, No> = IfEqual<
  EqualReformat<T>,
  EqualReformat<R>,
  Yes,
  No
>;

function typeCheck<T, R, Yes = "ok">(
  ok: IfDeepEqual<T, R, IfElse<T, R, Yes>, T>
): unknown {
  return ok;
}
