import os

import asyncio

from solana.rpc import types
from solana.rpc.async_api import AsyncClient
from solana.rpc.commitment import Finalized
from solana.transaction import Transaction
from solders.keypair import Keypair
from anchorpy import Provider, Wallet

from gen.instructions import initialize


async def main():

    # call an instruction
    url = os.environ["ANCHOR_PROVIDER_URL"]
    connection = AsyncClient(url, Finalized)
    wallet = Wallet.local()
    block_height = await connection.get_latest_blockhash()
    options = types.TxOpts(
        skip_confirmation=False,
        preflight_commitment=Finalized,
        max_retries=None,
        last_valid_block_height=block_height.value.last_valid_block_height
    )
    provider = Provider(connection, wallet, options)

    payer = provider.wallet.payer
    my_account = Keypair()  # the account to create（generating a new key-pair）

    ix = initialize(
        args={
            "data": 1234
        },
        accounts={
            "my_account": my_account.pubkey(),
            "user": payer.pubkey(),
        }
    )
    tx = Transaction().add(ix)
    tx.recent_blockhash = (await connection.get_latest_blockhash()).value.blockhash
    tx.sign(*[payer, my_account])

    signature = await provider.send(tx)
    if signature is not None:
        print(f"success: transaction signature is {signature}")

if __name__ == '__main__':
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main())
    loop.close()
