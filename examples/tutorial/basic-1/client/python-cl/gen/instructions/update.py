from __future__ import annotations
import typing
from solders.pubkey import Pubkey
from solders.instruction import Instruction, AccountMeta
import borsh_construct as borsh
from ..program_id import PROGRAM_ID


class UpdateArgs(typing.TypedDict):
    data: int


layout = borsh.CStruct("data" / borsh.U64)


class UpdateAccounts(typing.TypedDict):
    my_account: Pubkey


def update(
    args: UpdateArgs,
    accounts: UpdateAccounts,
    program_id: Pubkey = PROGRAM_ID,
    remaining_accounts: typing.Optional[typing.List[AccountMeta]] = None,
) -> Instruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["my_account"], is_signer=False, is_writable=True)
    ]
    if remaining_accounts is not None:
        keys += remaining_accounts
    identifier = b"\xdb\xc8X\xb0\x9e?\xfd\x7f"
    encoded_args = layout.build(
        {
            "data": args["data"],
        }
    )
    data = identifier + encoded_args
    return Instruction(program_id, data, keys)
