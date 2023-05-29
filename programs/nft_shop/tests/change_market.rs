use anchor_client::solana_sdk::transaction::Transaction;
use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use anchor_lang::prelude::Clock;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::{system_program, sysvar};
use anchor_lang::AccountDeserialize;
use anchor_lang::{InstructionData, ToAccountMetas};
use nft_minter::utils::Creator;
use nft_shop::error::ErrorCode;
use nft_shop::pda::*;
use nft_shop::state::Market;
use nft_shop::utils::{puffed_out_string, DESCRIPTION_MAX_LEN, NAME_MAX_LEN};
use solana_program_test::tokio;
use std::time::SystemTime;

mod utils;
use utils::{helpers::*, setup_functions::*};

#[tokio::test]
async fn change_market_success() {
    let mut context = nft_shop_program_test().start_with_context().await;

    let (store_admin, store_keypair) = create_store(&mut context).await;

    let selling_resource_owner_keypair = Keypair::new();
    airdrop(
        &mut context,
        &selling_resource_owner_keypair.pubkey(),
        10 * ONE_SOL,
    )
    .await
    .unwrap();

    let metadata_creators = Some(vec![Creator {
        address: selling_resource_owner_keypair.pubkey(),
        verified: false,
        share: 100,
    }]);
    let max_supply = Some(1);
    let metadata_is_mutable = true;

    let (selling_resource_keypair, _, _) = init_selling_resource(
        &mut context,
        &store_admin,
        &store_keypair,
        &selling_resource_owner_keypair,
        metadata_creators,
        max_supply,
        metadata_is_mutable,
    )
    .await;

    let market_keypair = Keypair::new();

    let treasury_mint_keypair = Keypair::new();
    create_mint(
        &mut context,
        &treasury_mint_keypair,
        &store_admin.pubkey(),
        0,
    )
    .await;

    let (treasury_owner, treasyry_owner_bump) = find_treasury_owner_address(
        &treasury_mint_keypair.pubkey(),
        &selling_resource_keypair.pubkey(),
    );

    let treasury_holder_keypair = Keypair::new();
    create_token_account(
        &mut context,
        &treasury_holder_keypair,
        &treasury_mint_keypair.pubkey(),
        &treasury_owner,
    )
    .await;

    let start_date = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp
        + 1;

    let market_name = "1234_1234_".to_string();
    let market_description = "1234_1234_1234_1234_".to_string();
    let mutable = true;
    let price = ONE_SOL;
    let pieces_in_one_wallet = Some(1);

    let data = nft_shop::instruction::CreateMarket {
        _treasury_owner_bump: treasyry_owner_bump,
        name: market_name.to_owned(),
        description: market_description.to_owned(),
        mutable,
        price,
        pieces_in_one_wallet,
        start_date: start_date as u64,
        end_date: None,
    };

    let accounts = nft_shop::accounts::CreateMarket {
        market: market_keypair.pubkey(),
        store: store_keypair.pubkey(),
        selling_resource_owner: selling_resource_owner_keypair.pubkey(),
        selling_resource: selling_resource_keypair.pubkey(),
        treasury_mint: treasury_mint_keypair.pubkey(),
        treasury_holder: treasury_holder_keypair.pubkey(),
        treasury_owner,
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
        &[
            &context.payer,
            &market_keypair,
            &selling_resource_owner_keypair,
        ],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // ChangeMarket
    let data = nft_shop::instruction::ChangeMarket {
        new_name: Some(String::from("1")),
        new_description: Some(String::from("2")),
        mutable: None,
        new_price: None,
        new_pieces_in_one_wallet: None,
    };

    let accounts = nft_shop::accounts::ChangeMarket {
        market: market_keypair.pubkey(),
        selling_resource_owner: selling_resource_owner_keypair.pubkey(),
        clock: sysvar::clock::id(),
    };

    let ix = Instruction {
        program_id: nft_shop::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &selling_resource_owner_keypair],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    context.warp_to_slot(clock.slot + 3).unwrap();

    let market_account = context
        .banks_client
        .get_account(market_keypair.pubkey())
        .await
        .expect("Account not found")
        .expect("Account is empty");

    let market_data = Market::try_deserialize(&mut market_account.data.as_ref()).unwrap();

    assert_eq!(
        puffed_out_string(String::from("1"), NAME_MAX_LEN),
        market_data.name
    );
    assert_eq!(
        puffed_out_string(String::from("2"), DESCRIPTION_MAX_LEN),
        market_data.description
    );
}

#[tokio::test]
async fn failure_market_is_ended() {
    let mut context = nft_shop_program_test().start_with_context().await;

    let (store_admin, store_keypair) = create_store(&mut context).await;

    let selling_resource_owner_keypair = Keypair::new();
    airdrop(
        &mut context,
        &selling_resource_owner_keypair.pubkey(),
        10 * ONE_SOL,
    )
    .await
    .unwrap();

    let metadata_creators = Some(vec![Creator {
        address: selling_resource_owner_keypair.pubkey(),
        verified: false,
        share: 100,
    }]);
    let max_supply = Some(1);
    let metadata_is_mutable = true;

    let (selling_resource_keypair, _, _) = init_selling_resource(
        &mut context,
        &store_admin,
        &store_keypair,
        &selling_resource_owner_keypair,
        metadata_creators,
        max_supply,
        metadata_is_mutable,
    )
    .await;

    let market_keypair = Keypair::new();

    let treasury_mint_keypair = Keypair::new();
    create_mint(
        &mut context,
        &treasury_mint_keypair,
        &store_admin.pubkey(),
        0,
    )
    .await;

    let (treasury_owner, treasyry_owner_bump) = find_treasury_owner_address(
        &treasury_mint_keypair.pubkey(),
        &selling_resource_keypair.pubkey(),
    );

    let treasury_holder_keypair = Keypair::new();
    create_token_account(
        &mut context,
        &treasury_holder_keypair,
        &treasury_mint_keypair.pubkey(),
        &treasury_owner,
    )
    .await;

    let start_date = (SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()) as u64;
    let end_date = (SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
        + 60) as u64;

    let market_name = "1234_1234_".to_string();
    let market_description = "1234_1234_1234_1234_".to_string();
    let mutable = true;
    let price = ONE_SOL;
    let pieces_in_one_wallet = Some(1);

    let data = nft_shop::instruction::CreateMarket {
        _treasury_owner_bump: treasyry_owner_bump,
        name: market_name.to_owned(),
        description: market_description.to_owned(),
        mutable,
        price,
        pieces_in_one_wallet,
        start_date: start_date,
        end_date: Some(end_date),
    };

    let accounts = nft_shop::accounts::CreateMarket {
        market: market_keypair.pubkey(),
        store: store_keypair.pubkey(),
        selling_resource_owner: selling_resource_owner_keypair.pubkey(),
        selling_resource: selling_resource_keypair.pubkey(),
        treasury_mint: treasury_mint_keypair.pubkey(),
        treasury_holder: treasury_holder_keypair.pubkey(),
        treasury_owner,
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
        &[
            &context.payer,
            &market_keypair,
            &selling_resource_owner_keypair,
        ],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    context.warp_to_slot(120 * 400 * 2).unwrap();

    // ChangeMarket
    let data = nft_shop::instruction::ChangeMarket {
        new_name: Some(String::from("1")),
        new_description: Some(String::from("2")),
        mutable: None,
        new_price: None,
        new_pieces_in_one_wallet: None,
    };

    let accounts = nft_shop::accounts::ChangeMarket {
        market: market_keypair.pubkey(),
        selling_resource_owner: selling_resource_owner_keypair.pubkey(),
        clock: sysvar::clock::id(),
    };

    let ix = Instruction {
        program_id: nft_shop::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &selling_resource_owner_keypair],
        context.last_blockhash,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::MarketIsEnded.into());
}

#[tokio::test]
async fn failure_market_is_immutable() {
    let mut context = nft_shop_program_test().start_with_context().await;

    let (store_admin, store_keypair) = create_store(&mut context).await;

    let selling_resource_owner_keypair = Keypair::new();
    airdrop(
        &mut context,
        &selling_resource_owner_keypair.pubkey(),
        10 * ONE_SOL,
    )
    .await
    .unwrap();

    let metadata_creators = Some(vec![Creator {
        address: selling_resource_owner_keypair.pubkey(),
        verified: false,
        share: 100,
    }]);
    let max_supply = Some(1);
    let metadata_is_mutable = true;

    let (selling_resource_keypair, _, _) = init_selling_resource(
        &mut context,
        &store_admin,
        &store_keypair,
        &selling_resource_owner_keypair,
        metadata_creators,
        max_supply,
        metadata_is_mutable,
    )
    .await;

    let market_keypair = Keypair::new();

    let treasury_mint_keypair = Keypair::new();
    create_mint(
        &mut context,
        &treasury_mint_keypair,
        &store_admin.pubkey(),
        0,
    )
    .await;

    let (treasury_owner, treasyry_owner_bump) = find_treasury_owner_address(
        &treasury_mint_keypair.pubkey(),
        &selling_resource_keypair.pubkey(),
    );

    let treasury_holder_keypair = Keypair::new();
    create_token_account(
        &mut context,
        &treasury_holder_keypair,
        &treasury_mint_keypair.pubkey(),
        &treasury_owner,
    )
    .await;

    let start_date = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp
        + 1;

    let market_name = "1234_1234_".to_string();
    let market_description = "1234_1234_1234_1234_".to_string();
    let mutable = false; // Immutable Market
    let price = ONE_SOL;
    let pieces_in_one_wallet = Some(1);

    let data = nft_shop::instruction::CreateMarket {
        _treasury_owner_bump: treasyry_owner_bump,
        name: market_name.to_owned(),
        description: market_description.to_owned(),
        mutable,
        price,
        pieces_in_one_wallet,
        start_date: start_date as u64,
        end_date: None,
    };

    let accounts = nft_shop::accounts::CreateMarket {
        market: market_keypair.pubkey(),
        store: store_keypair.pubkey(),
        selling_resource_owner: selling_resource_owner_keypair.pubkey(),
        selling_resource: selling_resource_keypair.pubkey(),
        treasury_mint: treasury_mint_keypair.pubkey(),
        treasury_holder: treasury_holder_keypair.pubkey(),
        treasury_owner,
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
        &[
            &context.payer,
            &market_keypair,
            &selling_resource_owner_keypair,
        ],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // ChangeMarket
    let data = nft_shop::instruction::ChangeMarket {
        new_name: Some(String::from("1")),
        new_description: Some(String::from("2")),
        mutable: None,
        new_price: None,
        new_pieces_in_one_wallet: None,
    };

    let accounts = nft_shop::accounts::ChangeMarket {
        market: market_keypair.pubkey(),
        selling_resource_owner: selling_resource_owner_keypair.pubkey(),
        clock: sysvar::clock::id(),
    };

    let ix = Instruction {
        program_id: nft_shop::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &selling_resource_owner_keypair],
        context.last_blockhash,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::MarketIsImmutable.into());
}
