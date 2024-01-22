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
    options = types.TxOpts(skip_confirmation=False, preflight_commitment=Finalized)
    connection = AsyncClient(url, options.preflight_commitment)
    wallet = Wallet.local()
    # await connection.get_block_height()
    print(await connection.get_latest_blockhash())

    # provider = Provider.env()  # get provider
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
    tx.sign(*[payer, my_account])

    import time; time.sleep(5)
    await provider.send(tx)

if __name__ == '__main__':
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main())
    loop.close()
