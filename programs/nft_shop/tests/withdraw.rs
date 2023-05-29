use anchor_client::solana_sdk::transaction::Transaction;
use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use anchor_lang::prelude::{AccountMeta, Clock};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program_pack::Pack;
use anchor_lang::solana_program::{system_program, sysvar};
use anchor_lang::{prelude::Pubkey, AccountDeserialize};
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::spl_token;
use nft_minter::utils::Creator;
use nft_shop::error::ErrorCode;
use nft_shop::pda::*;
use nft_shop::state::SellingResource;
use solana_program_test::tokio;
use std::time::SystemTime;

mod utils;
use utils::{helpers::*, setup_functions::*};

#[tokio::test]
async fn withdraw_success() {
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

    // CreateMarket

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

    let market_name = "1234_1234_".to_string();
    let market_description = "1234_1234_1234_1234_".to_string();
    let mutable = true;
    let price = 2 * ONE_SOL;
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

    // Waiting for Market`s start
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    context.warp_to_slot(clock.slot + 1500).unwrap();

    // Buy setup
    let selling_resource_data = context
        .banks_client
        .get_account(selling_resource_keypair.pubkey())
        .await
        .unwrap()
        .unwrap()
        .data;

    let selling_resource =
        SellingResource::try_deserialize(&mut selling_resource_data.as_ref()).unwrap();

    let (trade_history, trade_history_bump) =
        find_trade_history_address(&context.payer.pubkey(), &market_keypair.pubkey());
    let (vault_owner, vault_owner_bump) =
        find_vault_owner_address(&selling_resource.resource, &selling_resource.store);

    let user_wallet = context.payer.dirty_clone();

    let user_token_account = Keypair::new();
    create_token_account(
        &mut context,
        &user_token_account,
        &treasury_mint_keypair.pubkey(),
        &user_wallet.pubkey(),
    )
    .await;
    mint_to(
        &mut context,
        &treasury_mint_keypair.pubkey(),
        &user_token_account.pubkey(),
        &store_admin,
        price, // Selling Token price
    )
    .await;

    let new_mint_keypair = Keypair::new();
    create_mint(&mut context, &new_mint_keypair, &user_wallet.pubkey(), 0).await;

    let new_mint_token_account = Keypair::new();
    create_token_account(
        &mut context,
        &new_mint_token_account,
        &new_mint_keypair.pubkey(),
        &user_wallet.pubkey(),
    )
    .await;
    mint_to(
        &mut context,
        &new_mint_keypair.pubkey(),
        &new_mint_token_account.pubkey(),
        &user_wallet,
        1,
    )
    .await;

    let (metadata, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            selling_resource.resource.as_ref(),
        ],
        &mpl_token_metadata::id(),
    );

    let (master_edition, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            selling_resource.resource.as_ref(),
            mpl_token_metadata::state::EDITION.as_bytes(),
        ],
        &mpl_token_metadata::id(),
    );

    let (edition_marker, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            selling_resource.resource.as_ref(),
            mpl_token_metadata::state::EDITION.as_bytes(),
            selling_resource.supply.to_string().as_bytes(),
        ],
        &mpl_token_metadata::id(),
    );

    let (new_metadata, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            new_mint_keypair.pubkey().as_ref(),
        ],
        &mpl_token_metadata::id(),
    );

    let (new_edition, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            new_mint_keypair.pubkey().as_ref(),
            mpl_token_metadata::state::EDITION.as_bytes(),
        ],
        &mpl_token_metadata::id(),
    );

    // SavePrimaryMetadataCreators

    let primary_royalties_holder = Keypair::new();

    let (primary_metadata_creators, primary_metadata_creators_bump) =
        find_primary_metadata_creators(&metadata);

    let data = nft_shop::instruction::SavePrimaryMetadataCreators {
        primary_metadata_creators_bump,
        creators: vec![nft_shop::state::Creator {
            address: primary_royalties_holder.pubkey(),
            verified: false,
            share: 100,
        }],
    };

    let accounts = nft_shop::accounts::SavePrimaryMetadataCreators {
        metadata_update_authority: token.owner.pubkey(),
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
        &[&context.payer, &token.owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Buy

    let data = nft_shop::instruction::Buy {
        _trade_history_bump: trade_history_bump,
        vault_owner_bump,
    };

    let accounts = nft_shop::accounts::Buy {
        market: market_keypair.pubkey(),
        selling_resource: selling_resource_keypair.pubkey(),
        user_token_account: user_token_account.pubkey(),
        user_wallet: user_wallet.pubkey(),
        trade_history,
        treasury_holder: treasury_holder_keypair.pubkey(),
        new_metadata,
        new_edition,
        master_edition,
        new_mint: new_mint_keypair.pubkey(),
        edition_marker,
        vault: selling_resource.vault,
        vault_owner,
        new_token_account: new_mint_token_account.pubkey(),
        metadata,
        clock: sysvar::clock::id(),
        rent: sysvar::rent::id(),
        token_metadata_program: mpl_token_metadata::id(),
        token_program: spl_token::id(),
        system_program: system_program::id(),
    };

    let ix = Instruction {
        program_id: nft_shop::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&user_wallet.pubkey()),
        &[&user_wallet],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    context.warp_to_slot(clock.slot + 3).unwrap();

    // CloseMarket

    let accounts = nft_shop::accounts::CloseMarket {
        market: market_keypair.pubkey(),
        selling_resource_owner: selling_resource_owner_keypair.pubkey(),
        clock: sysvar::clock::id(),
    }
    .to_account_metas(None);

    let data = nft_shop::instruction::CloseMarket {}.data();

    let ix = Instruction {
        program_id: nft_shop::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &selling_resource_owner_keypair],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Withdraw

    let (payout_ticket, payout_ticket_bump) =
        find_payout_ticket_address(&market_keypair.pubkey(), &primary_royalties_holder.pubkey());

    let destination = get_associated_token_address(
        &primary_royalties_holder.pubkey(),
        &treasury_mint_keypair.pubkey(),
    );

    let (metadata, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            selling_resource.resource.as_ref(),
        ],
        &mpl_token_metadata::id(),
    );

    let data = nft_shop::instruction::Withdraw {
        payout_ticket_bump,
        treasury_owner_bump: treasyry_owner_bump,
    };

    let accounts = nft_shop::accounts::Withdraw {
        market: market_keypair.pubkey(),
        selling_resource: selling_resource_keypair.pubkey(),
        metadata,
        treasury_holder: treasury_holder_keypair.pubkey(),
        treasury_mint: treasury_mint_keypair.pubkey(),
        treasury_owner,
        destination,
        funder: primary_royalties_holder.pubkey(),
        selling_resource_owner: selling_resource_owner_keypair.pubkey(),
        payout_ticket,
        rent: sysvar::rent::id(),
        clock: sysvar::clock::id(),
        token_program: spl_token::id(),
        associated_token_program: anchor_spl::associated_token::ID,
        system_program: system_program::id(),
    };

    let mut account_metas = accounts.to_account_metas(None);
    account_metas.push(AccountMeta::new(primary_metadata_creators, false));

    let ix = Instruction {
        program_id: nft_shop::id(),
        data: data.data(),
        accounts: account_metas,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&selling_resource_owner_keypair.pubkey()),
        &[&selling_resource_owner_keypair],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let payout_ticket_account = context
        .banks_client
        .get_account(payout_ticket)
        .await
        .unwrap();

    let destination_account = context
        .banks_client
        .get_account(destination)
        .await
        .unwrap()
        .unwrap();
    let destination_token_account =
        spl_token::state::Account::unpack(&destination_account.data).unwrap();

    assert!(payout_ticket_account.is_some());
    assert_eq!(destination_token_account.amount, price);
}

#[tokio::test]
async fn failure_withdraw_twice() {
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

    // CreateMarket

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

    let market_name = "1234_1234_".to_string();
    let market_description = "1234_1234_1234_1234_".to_string();
    let mutable = true;
    let price = 2 * ONE_SOL;
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

    // Waiting for Market`s start
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    context.warp_to_slot(clock.slot + 1500).unwrap();

    // Buy setup
    let selling_resource_data = context
        .banks_client
        .get_account(selling_resource_keypair.pubkey())
        .await
        .unwrap()
        .unwrap()
        .data;

    let selling_resource =
        SellingResource::try_deserialize(&mut selling_resource_data.as_ref()).unwrap();

    let (trade_history, trade_history_bump) =
        find_trade_history_address(&context.payer.pubkey(), &market_keypair.pubkey());
    let (vault_owner, vault_owner_bump) =
        find_vault_owner_address(&selling_resource.resource, &selling_resource.store);

    let user_wallet = context.payer.dirty_clone();

    let user_token_account = Keypair::new();
    create_token_account(
        &mut context,
        &user_token_account,
        &treasury_mint_keypair.pubkey(),
        &user_wallet.pubkey(),
    )
    .await;
    mint_to(
        &mut context,
        &treasury_mint_keypair.pubkey(),
        &user_token_account.pubkey(),
        &store_admin,
        price, // Selling Token price
    )
    .await;

    let new_mint_keypair = Keypair::new();
    create_mint(&mut context, &new_mint_keypair, &user_wallet.pubkey(), 0).await;

    let new_mint_token_account = Keypair::new();
    create_token_account(
        &mut context,
        &new_mint_token_account,
        &new_mint_keypair.pubkey(),
        &user_wallet.pubkey(),
    )
    .await;
    mint_to(
        &mut context,
        &new_mint_keypair.pubkey(),
        &new_mint_token_account.pubkey(),
        &user_wallet,
        1,
    )
    .await;

    let (metadata, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            selling_resource.resource.as_ref(),
        ],
        &mpl_token_metadata::id(),
    );

    let (master_edition, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            selling_resource.resource.as_ref(),
            mpl_token_metadata::state::EDITION.as_bytes(),
        ],
        &mpl_token_metadata::id(),
    );

    let (edition_marker, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            selling_resource.resource.as_ref(),
            mpl_token_metadata::state::EDITION.as_bytes(),
            selling_resource.supply.to_string().as_bytes(),
        ],
        &mpl_token_metadata::id(),
    );

    let (new_metadata, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            new_mint_keypair.pubkey().as_ref(),
        ],
        &mpl_token_metadata::id(),
    );

    let (new_edition, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            new_mint_keypair.pubkey().as_ref(),
            mpl_token_metadata::state::EDITION.as_bytes(),
        ],
        &mpl_token_metadata::id(),
    );

    // SavePrimaryMetadataCreators

    let primary_royalties_holder = Keypair::new();

    let (primary_metadata_creators, primary_metadata_creators_bump) =
        find_primary_metadata_creators(&metadata);

    let data = nft_shop::instruction::SavePrimaryMetadataCreators {
        primary_metadata_creators_bump,
        creators: vec![nft_shop::state::Creator {
            address: primary_royalties_holder.pubkey(),
            verified: false,
            share: 100,
        }],
    };

    let accounts = nft_shop::accounts::SavePrimaryMetadataCreators {
        metadata_update_authority: token.owner.pubkey(),
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
        &[&context.payer, &token.owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Buy

    let data = nft_shop::instruction::Buy {
        _trade_history_bump: trade_history_bump,
        vault_owner_bump,
    };

    let accounts = nft_shop::accounts::Buy {
        market: market_keypair.pubkey(),
        selling_resource: selling_resource_keypair.pubkey(),
        user_token_account: user_token_account.pubkey(),
        user_wallet: user_wallet.pubkey(),
        trade_history,
        treasury_holder: treasury_holder_keypair.pubkey(),
        new_metadata,
        new_edition,
        master_edition,
        new_mint: new_mint_keypair.pubkey(),
        edition_marker,
        vault: selling_resource.vault,
        vault_owner,
        new_token_account: new_mint_token_account.pubkey(),
        metadata,
        clock: sysvar::clock::id(),
        rent: sysvar::rent::id(),
        token_metadata_program: mpl_token_metadata::id(),
        token_program: spl_token::id(),
        system_program: system_program::id(),
    };

    let ix = Instruction {
        program_id: nft_shop::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&user_wallet.pubkey()),
        &[&user_wallet],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    context.warp_to_slot(clock.slot + 3).unwrap();

    // CloseMarket

    let accounts = nft_shop::accounts::CloseMarket {
        market: market_keypair.pubkey(),
        selling_resource_owner: selling_resource_owner_keypair.pubkey(),
        clock: sysvar::clock::id(),
    }
    .to_account_metas(None);

    let data = nft_shop::instruction::CloseMarket {}.data();

    let ix = Instruction {
        program_id: nft_shop::id(),
        data,
        accounts,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &selling_resource_owner_keypair],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // First Withdraw

    let (payout_ticket, payout_ticket_bump) =
        find_payout_ticket_address(&market_keypair.pubkey(), &primary_royalties_holder.pubkey());

    let destination = get_associated_token_address(
        &primary_royalties_holder.pubkey(),
        &treasury_mint_keypair.pubkey(),
    );

    let (metadata, _) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            selling_resource.resource.as_ref(),
        ],
        &mpl_token_metadata::id(),
    );

    let data = nft_shop::instruction::Withdraw {
        payout_ticket_bump,
        treasury_owner_bump: treasyry_owner_bump,
    };

    let accounts = nft_shop::accounts::Withdraw {
        market: market_keypair.pubkey(),
        selling_resource: selling_resource_keypair.pubkey(),
        metadata,
        treasury_holder: treasury_holder_keypair.pubkey(),
        treasury_mint: treasury_mint_keypair.pubkey(),
        treasury_owner,
        destination,
        funder: primary_royalties_holder.pubkey(),
        selling_resource_owner: selling_resource_owner_keypair.pubkey(),
        payout_ticket,
        rent: sysvar::rent::id(),
        clock: sysvar::clock::id(),
        token_program: spl_token::id(),
        associated_token_program: anchor_spl::associated_token::ID,
        system_program: system_program::id(),
    };

    let mut account_metas = accounts.to_account_metas(None);
    account_metas.push(AccountMeta::new(primary_metadata_creators, false));

    let ix = Instruction {
        program_id: nft_shop::id(),
        data: data.data(),
        accounts: account_metas,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix.clone()],
        Some(&selling_resource_owner_keypair.pubkey()),
        &[&selling_resource_owner_keypair],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    context.warp_to_slot(clock.slot + 3).unwrap();

    // Second Withdraw

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&selling_resource_owner_keypair.pubkey()),
        &[&selling_resource_owner_keypair],
        context.last_blockhash,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::PayoutTicketExists.into());
}
