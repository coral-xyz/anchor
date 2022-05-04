use anchor_lang::{
    prelude::*, solana_program::instruction::Instruction, system_program, InstructionData,
};
use basic_1::MyAccount;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

/// Creates and initializes an account in a single atomic transaction (simplified).
#[tokio::test]
async fn create_and_init() {
    let program_test = ProgramTest::new("basic_1", basic_1::id(), None);
    let mut test_context = program_test.start_with_context().await;

    let my_account_key = Keypair::new();

    let ix_accounts = basic_1::accounts::Initialize {
        my_account: my_account_key.pubkey(),
        user: test_context.payer.pubkey(),
        system_program: system_program::ID,
    };

    let ix_arg = basic_1::instruction::Initialize { data: 1234 };

    let ix = Instruction {
        program_id: basic_1::id(),
        accounts: ix_accounts.to_account_metas(Some(true)),
        data: ix_arg.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&test_context.payer.pubkey()),
        &[&test_context.payer, &my_account_key],
        test_context.last_blockhash,
    );

    test_context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    let my_account_ai = test_context
        .banks_client
        .get_account(my_account_key.pubkey())
        .await
        .unwrap()
        .unwrap();
    let my_account = MyAccount::try_deserialize(&mut my_account_ai.data.as_slice()).unwrap();

    assert_eq!(my_account.data, 1234);
}

/// Updates a previously created account
#[tokio::test]
async fn update() {
    let program_test = ProgramTest::new("basic_1", basic_1::id(), None);
    let mut test_context = program_test.start_with_context().await;

    let my_account_pk = {
        let my_account_key = Keypair::new();

        let ix_accounts = basic_1::accounts::Initialize {
            my_account: my_account_key.pubkey(),
            user: test_context.payer.pubkey(),
            system_program: system_program::ID,
        };

        let ix_arg = basic_1::instruction::Initialize { data: 1234 };

        let ix = Instruction {
            program_id: basic_1::id(),
            accounts: ix_accounts.to_account_metas(Some(true)),
            data: ix_arg.data(),
        };

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&test_context.payer.pubkey()),
            &[&test_context.payer, &my_account_key],
            test_context.last_blockhash,
        );

        test_context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap();

        my_account_key.pubkey()
    };

    let ix_accounts = basic_1::accounts::Update {
        my_account: my_account_pk,
    };
    let ix_arg = basic_1::instruction::Update { data: 4321 };

    let ix = Instruction {
        program_id: basic_1::id(),
        accounts: ix_accounts.to_account_metas(Some(false)),
        data: ix_arg.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&test_context.payer.pubkey()),
        &[&test_context.payer],
        test_context.last_blockhash,
    );

    test_context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    let my_account_ai = test_context
        .banks_client
        .get_account(my_account_pk)
        .await
        .unwrap()
        .unwrap();
    let my_account = MyAccount::try_deserialize(&mut my_account_ai.data.as_slice()).unwrap();

    assert_eq!(my_account.data, 4321);
}
