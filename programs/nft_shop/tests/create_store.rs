use anchor_client::solana_sdk::transaction::Transaction;
use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::system_program;
use anchor_lang::AccountDeserialize;
use anchor_lang::{InstructionData, ToAccountMetas};
use nft_shop::error::ErrorCode;
use nft_shop::state::Store;
use nft_shop::utils::{puffed_out_string, DESCRIPTION_MAX_LEN, NAME_MAX_LEN};
use solana_program_test::tokio;

mod utils;
use utils::{helpers::*, setup_functions::*};

#[tokio::test]
async fn create_store_success() {
    let mut context = nft_shop_program_test().start_with_context().await;

    let store_admin = Keypair::new();
    airdrop(&mut context, &store_admin.pubkey(), 10 * ONE_SOL)
        .await
        .unwrap();

    let store = Keypair::new();

    let name = String::from("123456789_123456789_");
    let description = String::from("123456789_123456789_123456789_");

    let data = nft_shop::instruction::CreateStore {
        name: name.clone(),
        description: description.clone(),
    };

    let accounts = nft_shop::accounts::CreateStore {
        store_admin: store_admin.pubkey(),
        store: store.pubkey(),
        system_program: system_program::id(),
    };

    let ix = Instruction {
        program_id: nft_shop::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &store_admin, &store],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let store_account = context
        .banks_client
        .get_account(store.pubkey())
        .await
        .expect("Account not found")
        .expect("Account is empty");

    let store_account_data = Store::try_deserialize(&mut store_account.data.as_ref()).unwrap();

    assert_eq!(store_admin.pubkey(), store_account_data.admin);
    assert_eq!(
        puffed_out_string(name, NAME_MAX_LEN),
        store_account_data.name
    );
    assert_eq!(
        puffed_out_string(description, DESCRIPTION_MAX_LEN),
        store_account_data.description
    );
}

#[tokio::test]
#[should_panic]
async fn failure_store_signer_is_missed() {
    let mut context = nft_shop_program_test().start_with_context().await;

    let store_admin = Keypair::new();
    airdrop(&mut context, &store_admin.pubkey(), 10 * ONE_SOL)
        .await
        .unwrap();

    let store = Keypair::new();

    let name = String::from("123456789_123456789_");
    let description = String::from("123456789_123456789_123456789_");

    let data = nft_shop::instruction::CreateStore {
        name: name.clone(),
        description: description.clone(),
    };

    let accounts = nft_shop::accounts::CreateStore {
        store_admin: store_admin.pubkey(),
        store: store.pubkey(),
        system_program: system_program::id(),
    };

    let ix = Instruction {
        program_id: nft_shop::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &store_admin], // Store keypair is missed
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();
}

#[tokio::test]
async fn failure_store_name_is_too_long() {
    let mut context = nft_shop_program_test().start_with_context().await;

    let store_admin = Keypair::new();
    airdrop(&mut context, &store_admin.pubkey(), 10 * ONE_SOL)
        .await
        .unwrap();

    let store = Keypair::new();

    let name = String::from("123456789_123456789_123456789_123456789_1");
    let description = String::from("123456789_123456789_123456789_");

    let data = nft_shop::instruction::CreateStore {
        name: name.clone(),
        description: description.clone(),
    };

    let accounts = nft_shop::accounts::CreateStore {
        store_admin: store_admin.pubkey(),
        store: store.pubkey(),
        system_program: system_program::id(),
    };

    let ix = Instruction {
        program_id: nft_shop::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &store_admin, &store],
        context.last_blockhash,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::NameIsTooLong.into());
}

#[tokio::test]
async fn failure_store_description_is_too_long() {
    let mut context = nft_shop_program_test().start_with_context().await;

    let store_admin = Keypair::new();
    airdrop(&mut context, &store_admin.pubkey(), 10 * ONE_SOL)
        .await
        .unwrap();

    let store = Keypair::new();

    let name = String::from("123456789_123456789_");
    let description = String::from("123456789_123456789_123456789_123456789_123456789_123456789_1");

    let data = nft_shop::instruction::CreateStore {
        name: name.clone(),
        description: description.clone(),
    };

    let accounts = nft_shop::accounts::CreateStore {
        store_admin: store_admin.pubkey(),
        store: store.pubkey(),
        system_program: system_program::id(),
    };

    let ix = Instruction {
        program_id: nft_shop::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &store_admin, &store],
        context.last_blockhash,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::DescriptionIsTooLong.into());
}
