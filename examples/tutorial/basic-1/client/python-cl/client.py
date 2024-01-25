import os

import asyncio

from solana.rpc import types
from solana.rpc.async_api import AsyncClient
from solana.rpc.commitment import Finalized
from solana.transaction import Transaction
from solders.keypair import Keypair
from anchorpy import Provider, Wallet

from gen.instructions import initialize, update


async def main():

    # call an instruction
    url = os.environ["ANCHOR_PROVIDER_URL"]
    connection = AsyncClient(url, Finalized)
    wallet = Wallet.local()
    options = types.TxOpts(
        skip_confirmation=False,
        preflight_commitment=Finalized,
        max_retries=None,
        last_valid_block_height=(await connection.get_latest_blockhash()).value.last_valid_block_height
    )
    provider = Provider(connection, wallet, options)

    payer = provider.wallet.payer
    my_account = Keypair()  # the account to create（generating a new key-pair）

    ix1 = initialize(
        args={
            "data": 1234
        },
        accounts={
            "my_account": my_account.pubkey(),
            "user": payer.pubkey(),
        }
    )
    ix2 = update(
        args={
            "data": 4321
        },
        accounts={
            "my_account": my_account.pubkey(),
        }
    )
    tx = Transaction(fee_payer=payer.pubkey()).add(ix1).add(ix2)
    tx.recent_blockhash = (await connection.get_latest_blockhash()).value.blockhash
    tx.sign(*[payer, my_account])

    signature = await provider.send(tx, opts=options)
    if signature is not None:
        print(f"success: transaction signature is {signature}")

if __name__ == '__main__':
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main())
    loop.close()
