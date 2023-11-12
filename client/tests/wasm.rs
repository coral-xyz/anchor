use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig, signature::Keypair, signer::Signer, system_instruction,
    },
    Client, Cluster,
};
use anchor_lang::{prelude::Pubkey, system_program};
use basic_2::instruction as basic_2_instruction;
use basic_2::{accounts as basic_2_accounts, Counter};
use basic_4::accounts as basic_4_accounts;
use basic_4::instruction as basic_4_instruction;
use basic_4::Counter as CounterAccount;
use composite::accounts::{Bar, CompositeUpdate, Fred, Initialize};
use composite::instruction as composite_instruction;
use composite::{DummyA, DummyB};
use events::instruction as events_instruction;
use events::MyEvent;
use optional::account::{DataAccount, DataPda};
use optional::accounts::Initialize as OptionalInitialize;
use optional::instruction as optional_instruction;
use solana_client_wasm::WasmClient;
use std::{rc::Rc, str::FromStr};
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn get_client() -> (Client<Rc<Keypair>>, CommitmentConfig) {
    // Wallet and cluster params.
    let payer = Keypair::new();
    let url = Cluster::Custom(
        "http://localhost:8899".to_string(),
        "ws://127.0.0.1:8900".to_string(),
    );
    let config = CommitmentConfig::processed();

    let payer = Rc::new(payer);
    let client = Client::new_with_options(url.clone(), payer.clone(), config);

    (client, config)
}

async fn airdrop(rpc: &WasmClient, config: CommitmentConfig, user: Pubkey) {
    let signature = rpc.request_airdrop(&user, 100000000).await.unwrap();

    rpc.confirm_transaction_with_commitment(&signature, config)
        .await
        .unwrap();
}

#[wasm_bindgen_test]
async fn basic_2() {
    let (client, config) = get_client();

    // Program client.
    let program = client
        .program(Pubkey::from_str("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS").unwrap())
        .unwrap();

    airdrop(&program.async_rpc(), config, program.payer()).await;

    // `Create` parameters.
    let counter = Keypair::new();
    let authority = program.payer();

    // Build and send a transaction.
    program
        .request()
        .signer(&counter)
        .accounts(basic_2_accounts::Create {
            counter: counter.pubkey(),
            user: authority,
            system_program: system_program::ID,
        })
        .args(basic_2_instruction::Create { authority })
        .send()
        .await
        .unwrap();

    let counter_account: Counter = program.account(counter.pubkey()).await.unwrap();

    assert_eq!(counter_account.authority, authority);
    assert_eq!(counter_account.count, 0);
}

#[wasm_bindgen_test]
async fn composite() {
    let (client, config) = get_client();

    // Program client.
    let program = client
        .program(Pubkey::from_str("EHthziFziNoac9LBGxEaVN47Y3uUiRoXvqAiR6oes4iU").unwrap())
        .unwrap();

    airdrop(&program.async_rpc(), config, program.payer()).await;

    // `Initialize` parameters.
    let dummy_a = Keypair::new();
    let dummy_b = Keypair::new();

    // Build and send a transaction.
    program
        .request()
        .instruction(system_instruction::create_account(
            &program.payer(),
            &dummy_a.pubkey(),
            program
                .async_rpc()
                .get_minimum_balance_for_rent_exemption(500)
                .await
                .unwrap(),
            500,
            &program.id(),
        ))
        .instruction(system_instruction::create_account(
            &program.payer(),
            &dummy_b.pubkey(),
            program
                .async_rpc()
                .get_minimum_balance_for_rent_exemption(500)
                .await
                .unwrap(),
            500,
            &program.id(),
        ))
        .signer(&dummy_a)
        .signer(&dummy_b)
        .accounts(Initialize {
            dummy_a: dummy_a.pubkey(),
            dummy_b: dummy_b.pubkey(),
        })
        .args(composite_instruction::Initialize)
        .send()
        .await
        .unwrap();

    // Assert the transaction worked.
    let dummy_a_account: DummyA = program.account(dummy_a.pubkey()).await.unwrap();
    let dummy_b_account: DummyB = program.account(dummy_b.pubkey()).await.unwrap();
    assert_eq!(dummy_a_account.data, 0);
    assert_eq!(dummy_b_account.data, 0);

    // Build and send another transaction, using composite account parameters.
    program
        .request()
        .accounts(CompositeUpdate {
            fred: Fred {
                dummy_a: dummy_a.pubkey(),
            },
            bar: Bar {
                dummy_b: dummy_b.pubkey(),
            },
        })
        .args(composite_instruction::CompositeUpdate {
            dummy_a: 1234,
            dummy_b: 4321,
        })
        .send()
        .await
        .unwrap();

    // Assert the transaction worked.
    let dummy_a_account: DummyA = program.account(dummy_a.pubkey()).await.unwrap();
    let dummy_b_account: DummyB = program.account(dummy_b.pubkey()).await.unwrap();

    assert_eq!(dummy_a_account.data, 1234);
    assert_eq!(dummy_b_account.data, 4321);
}

#[wasm_bindgen_test]
async fn events() {
    let (client, config) = get_client();

    // Program client.
    let program = client
        .program(Pubkey::from_str("2dhGsWUzy5YKUsjZdLHLmkNpUDAXkNa9MYWsPc4Ziqzy").unwrap())
        .unwrap();

    airdrop(&program.async_rpc(), config, program.payer()).await;

    let (sender, mut receiver) = futures::channel::mpsc::unbounded();
    let event_unsubscriber = program
        .on(move |_, event: MyEvent| {
            if sender.unbounded_send(event).is_err() {
                println!("Error while transferring the event.")
            }
        })
        .await;

    program
        .request()
        .args(events_instruction::Initialize {})
        .send()
        .await
        .unwrap();

    let event = receiver.try_next().unwrap().unwrap();
    assert_eq!(event.data, 5);
    assert_eq!(event.label, "hello".to_string());

    event_unsubscriber.unsubscribe().await;
}

#[wasm_bindgen_test]
async fn basic_4() {
    let (client, config) = get_client();

    let pid = Pubkey::from_str("CwrqeMj2U8tFr1Rhkgwc84tpAsqbt9pTt2a4taoTADPr").unwrap();
    // Program client.
    let program = client.program(pid).unwrap();

    airdrop(&program.async_rpc(), config, program.payer()).await;

    let authority = program.payer();
    let (counter, _) = Pubkey::find_program_address(&[b"counter"], &pid);

    program
        .request()
        .accounts(basic_4_accounts::Initialize {
            counter,
            authority,
            system_program: system_program::ID,
        })
        .args(basic_4_instruction::Initialize {})
        .send()
        .await
        .unwrap();
    let counter_account: CounterAccount = program.account(counter).await.unwrap();
    assert_eq!(counter_account.authority, authority);
    assert_eq!(counter_account.count, 0);

    program
        .request()
        .accounts(basic_4_accounts::Increment { counter, authority })
        .args(basic_4_instruction::Increment {})
        .send()
        .await
        .unwrap();

    let counter_account: CounterAccount = program.account(counter).await.unwrap();
    assert_eq!(counter_account.authority, authority);
    assert_eq!(counter_account.count, 1);
}

#[wasm_bindgen_test]
async fn optional() {
    let (client, config) = get_client();

    let pid = Pubkey::from_str("FNqz6pqLAwvMSds2FYjR4nKV3moVpPNtvkfGFrqLKrgG").unwrap();
    // Program client.
    let program = client.program(pid).unwrap();

    airdrop(&program.async_rpc(), config, program.payer()).await;

    // `Initialize` parameters.
    let data_account_keypair = Keypair::new();

    let data_account_key = data_account_keypair.pubkey();

    let data_pda_seeds = &[DataPda::PREFIX.as_ref(), data_account_key.as_ref()];
    let data_pda_key = Pubkey::find_program_address(data_pda_seeds, &pid).0;
    let required_keypair = Keypair::new();
    let value: u64 = 10;

    // Build and send a transaction.

    program
        .request()
        .instruction(system_instruction::create_account(
            &program.payer(),
            &required_keypair.pubkey(),
            program
                .async_rpc()
                .get_minimum_balance_for_rent_exemption(DataAccount::LEN)
                .await
                .unwrap(),
            DataAccount::LEN as u64,
            &program.id(),
        ))
        .signer(&data_account_keypair)
        .signer(&required_keypair)
        .accounts(OptionalInitialize {
            payer: Some(program.payer()),
            required: required_keypair.pubkey(),
            system_program: Some(system_program::ID),
            optional_account: Some(data_account_keypair.pubkey()),
            optional_pda: None,
        })
        .args(optional_instruction::Initialize { value, key: pid })
        .send()
        .await
        .unwrap();

    // Assert the transaction worked.
    let required: DataAccount = program.account(required_keypair.pubkey()).await.unwrap();
    assert_eq!(required.data, 0);

    let optional_pda = program.account::<DataPda>(data_pda_key).await;
    assert!(optional_pda.is_err());

    let optional_account: DataAccount = program
        .account(data_account_keypair.pubkey())
        .await
        .unwrap();
    assert_eq!(optional_account.data, value * 2);
}
