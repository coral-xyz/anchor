# Tutorial Basic 1

## devnetにデプロイしたアカウント
Explorer上のDevnetを選択してPubkeyを指定して確認可能。
programはinitialize methodとupdate methodを持っている
- https://explorer.solana.com/address/Dysswo9ycPdcCFKsn2NJGRCB9z7FY1rdiJXBhS6iVQB?cluster=devnet

## instruction単位でtransactionを実行する
client.jsを動かす（sh exec_client.sh）ことで可能。  
デプロイしたアカウントを見に行くとTransactionのログも見れる。
- [initializeのtxログの例](https://explorer.solana.com/tx/4K5yL1WAsrboqdEESyCGvaTyZyAEpncuBJX6Jz5vSxM6UXb1iA2ijoTSvnG1fAsoHGaMeDFr4Q3ixK55x39zKDP4?cluster=devnet)
- [updateのtxログの例](https://explorer.solana.com/tx/4MVkgHo5i4ZMB1nCzHJSduuCw6DrjuVpTqe4xL1GzDQ4NSYUPdYcAeBZi3MQhQVN3gseMPdhy7Xd77EsrJBem9Wj?cluster=devnet)

上記の２つのtxはある一度のclient.jsの実行で行ったtxのログである。
それゆえ、同一のデータアカウント（9mDqqmemXr6RJSXUNE788hi7JL38fBn8fYumA8KfN1x5）を内部で扱っていることが分かる。

client.jsでは
>   const programId = new anchor.web3.PublicKey("DzVuV6qMC2oJJEwerpYrenPDhTQuHQRfMe4LdCmMZJYK");


で実行の度にデータアカウントを生成していて、実行内ではinitialize、updateの両方で使い回すのでそのようになっている。逆に言えば、別々の実行ではデータアカウントは別物となっている。



## WIP:instructionをまとめてtransactionを実行する
client-2.jsではinitializeとupdateをまとめて1transactionとして実現したいと考えているが、今のところ出来てない。（つらい）
