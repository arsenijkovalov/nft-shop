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
use nft_shop::state::PrimaryMetadataCreators;
use solana_program_test::tokio;

mod utils;
use utils::{helpers::*, setup_functions::*};

#[tokio::test]
async fn save_primary_metadata_creators_success() {
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

    let (selling_resource_keypair, _, token) = init_selling_resource(
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

    // SavePrimaryMetadataCreators
    let (primary_metadata_creators, primary_metadata_creators_bump) =
        find_primary_metadata_creators(&token.metadata);

    let data = nft_shop::instruction::SavePrimaryMetadataCreators {
        primary_metadata_creators_bump,
        creators: vec![nft_shop::state::Creator {
            address: store_admin.pubkey(),
            share: 100,
            verified: false,
        }],
    };

    let accounts = nft_shop::accounts::SavePrimaryMetadataCreators {
        metadata_update_authority: store_admin.pubkey(),
        metadata: token.metadata,
        primary_metadata_creators,
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
        &[&context.payer, &store_admin],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    let primary_metadata_creators_account = context
        .banks_client
        .get_account(primary_metadata_creators)
        .await
        .expect("Account not found")
        .expect("Account is empty");

    let primary_metadata_creators_data = PrimaryMetadataCreators::try_deserialize(
        &mut primary_metadata_creators_account.data.as_ref(),
    )
    .unwrap();

    assert!(!primary_metadata_creators_data.creators.is_empty());
}

#[tokio::test]
async fn failure_metadata_creators_is_gt_than_available() {
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

    let (selling_resource_keypair, _, token) = init_selling_resource(
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

    // SavePrimaryMetadataCreators
    let (primary_metadata_creators, primary_metadata_creators_bump) =
        find_primary_metadata_creators(&token.metadata);

    let data = nft_shop::instruction::SavePrimaryMetadataCreators {
        primary_metadata_creators_bump,
        creators: vec![
            nft_shop::state::Creator {
                address: store_admin.pubkey(),
                share: 10,
                verified: false,
            },
            nft_shop::state::Creator {
                address: store_admin.pubkey(),
                share: 10,
                verified: false,
            },
            nft_shop::state::Creator {
                address: store_admin.pubkey(),
                share: 10,
                verified: false,
            },
            nft_shop::state::Creator {
                address: store_admin.pubkey(),
                share: 10,
                verified: false,
            },
            nft_shop::state::Creator {
                address: store_admin.pubkey(),
                share: 10,
                verified: false,
            },
            nft_shop::state::Creator {
                address: store_admin.pubkey(),
                share: 10,
                verified: false,
            },
        ],
    };

    let accounts = nft_shop::accounts::SavePrimaryMetadataCreators {
        metadata_update_authority: store_admin.pubkey(),
        metadata: token.metadata,
        primary_metadata_creators,
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
        &[&context.payer, &store_admin],
        context.last_blockhash,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::CreatorsIsGtThanAvailable.into());
}

#[tokio::test]
async fn failure_metadata_creators_is_empty() {
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

    let (selling_resource_keypair, _, token) = init_selling_resource(
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

    // SavePrimaryMetadataCreators
    let (primary_metadata_creators, primary_metadata_creators_bump) =
        find_primary_metadata_creators(&token.metadata);

    let data = nft_shop::instruction::SavePrimaryMetadataCreators {
        primary_metadata_creators_bump,
        creators: vec![],
    };

    let accounts = nft_shop::accounts::SavePrimaryMetadataCreators {
        metadata_update_authority: store_admin.pubkey(),
        metadata: token.metadata,
        primary_metadata_creators,
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
        &[&context.payer, &store_admin],
        context.last_blockhash,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::CreatorsIsEmpty.into());
}

#[tokio::test]
async fn failure_metadata_is_immutable() {
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
    let metadata_is_mutable = false;

    let (selling_resource_keypair, _, token) = init_selling_resource(
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

    // SavePrimaryMetadataCreators
    let (primary_metadata_creators, primary_metadata_creators_bump) =
        find_primary_metadata_creators(&token.metadata);

    let data = nft_shop::instruction::SavePrimaryMetadataCreators {
        primary_metadata_creators_bump,
        creators: vec![nft_shop::state::Creator {
            address: store_admin.pubkey(),
            share: 100,
            verified: false,
        }],
    };

    let accounts = nft_shop::accounts::SavePrimaryMetadataCreators {
        metadata_update_authority: store_admin.pubkey(),
        metadata: token.metadata,
        primary_metadata_creators,
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
        &[&context.payer, &store_admin],
        context.last_blockhash,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::MetadataShouldBeMutable.into());
}
