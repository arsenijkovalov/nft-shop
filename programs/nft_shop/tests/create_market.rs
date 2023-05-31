use anchor_client::solana_sdk::transaction::Transaction;
use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use anchor_lang::prelude::Clock;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::system_program;
use anchor_lang::AccountDeserialize;
use anchor_lang::{InstructionData, ToAccountMetas};
use nft_minter::utils::Creator;
use nft_shop::error::ErrorCode;
use nft_shop::pda::*;
use nft_shop::state::{Market, MarketState};
use nft_shop::utils::{puffed_out_string, DESCRIPTION_MAX_LEN, NAME_MAX_LEN};
use solana_program_test::tokio;

mod utils;
use utils::{helpers::*, setup_functions::*};

#[tokio::test]
async fn create_market_success() {
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

    let market_account = context
        .banks_client
        .get_account(market_keypair.pubkey())
        .await
        .expect("Account not found")
        .expect("Account is empty");

    let market_data = Market::try_deserialize(&mut market_account.data.as_ref()).unwrap();

    assert_eq!(store_keypair.pubkey(), market_data.store);
    assert_eq!(
        selling_resource_keypair.pubkey(),
        market_data.selling_resource
    );
    assert_eq!(treasury_mint_keypair.pubkey(), market_data.treasury_mint);
    assert_eq!(
        treasury_holder_keypair.pubkey(),
        market_data.treasury_holder
    );
    assert_eq!(treasury_owner, market_data.treasury_owner);
    assert_eq!(selling_resource_owner_keypair.pubkey(), market_data.owner);
    assert_eq!(
        puffed_out_string(market_name, NAME_MAX_LEN),
        market_data.name
    );
    assert_eq!(
        puffed_out_string(market_description, DESCRIPTION_MAX_LEN),
        market_data.description
    );
    assert_eq!(mutable, market_data.mutable);
    assert_eq!(price, market_data.price);
    assert_eq!(pieces_in_one_wallet, market_data.pieces_in_one_wallet);
    assert_eq!(MarketState::Created, market_data.state);
}

#[tokio::test]
async fn failure_market_name_is_long() {
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

    let market_name = "123456789_123456789_123456789_123456789_1".to_string();
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

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::NameIsTooLong.into());
}

#[tokio::test]
async fn failure_description_name_is_long() {
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
    let market_description =
        "123456789_123456789_123456789_123456789_123456789_123456789_1".to_string();
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

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::DescriptionIsTooLong.into());
}

#[tokio::test]
#[should_panic]
async fn failure_market_signer_is_missed() {
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
            // Market keypair is missed
            &selling_resource_owner_keypair,
        ],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();
}

#[tokio::test]
async fn failure_market_start_date_is_in_the_past() {
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
        - 1;

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

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::StartDateIsInPast.into());
}

#[tokio::test]
async fn failure_market_end_date_is_earlier_than_start_date() {
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

    let end_date = start_date - 2;

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
        end_date: Some(end_date as u64),
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

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::EndDateIsEarlierThanBeginDate.into());
}

#[tokio::test]
#[should_panic]
async fn failure_treasury_mint_uninitialized() {
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
}

#[tokio::test]
#[should_panic]
async fn failure_treasury_holder_uninitialized() {
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
}
