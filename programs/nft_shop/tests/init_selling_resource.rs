use anchor_client::solana_sdk::transaction::Transaction;
use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::{system_program, sysvar};
use anchor_lang::AccountDeserialize;
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::token::spl_token;
use nft_minter::utils::Creator;
use nft_shop::error::ErrorCode;
use nft_shop::pda::*;
use nft_shop::state::{SellingResource, SellingResourceState};
use solana_program_test::tokio;

mod utils;
use utils::{helpers::*, setup_functions::*};

#[tokio::test]
async fn init_selling_resource_success() {
    let mut context = nft_shop_program_test().start_with_context().await;

    // NOTE: In real case, Store administrator must be the Token owner, if the Token already exists
    //       and will be provided as a resource for sale.

    let (store_admin, store_keypair) = create_store(&mut context).await;

    let metadata_creators = Some(vec![Creator {
        address: store_admin.pubkey(),
        verified: false,
        share: 100,
    }]);
    let max_supply = Some(1);
    let metadata_is_mutable = true;

    // Create Token to be provided as a resource for sale
    let token = create_nft(
        &mut context,
        store_admin.dirty_clone(),
        metadata_creators,
        max_supply,
        metadata_is_mutable,
    )
    .await
    .unwrap();

    let (vault_owner, vault_owner_bump) =
        find_vault_owner_address(&token.mint.pubkey(), &store_keypair.pubkey());

    let vault = Keypair::new();
    create_token_account(&mut context, &vault, &token.mint.pubkey(), &vault_owner).await;

    let selling_resource = Keypair::new();

    let data = nft_shop::instruction::InitSellingResource {
        master_edition_bump: token.master_edition.bump,
        vault_owner_bump,
        max_supply,
    };

    let accounts = nft_shop::accounts::InitSellingResource {
        store: store_keypair.pubkey(),
        store_admin: store_admin.pubkey(),
        selling_resource: selling_resource.pubkey(),
        selling_resource_owner: store_admin.pubkey(),
        resource_mint: token.mint.pubkey(),
        master_edition: token.master_edition.edition,
        metadata: token.metadata,
        vault: vault.pubkey(),
        vault_owner,
        resource_token: token.ata,
        rent: sysvar::rent::id(),
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
        Some(&store_admin.pubkey()),
        &[&store_admin, &selling_resource],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let selling_resource_account = context
        .banks_client
        .get_account(selling_resource.pubkey())
        .await
        .expect("Account not found")
        .expect("Account is empty");

    let selling_resource_data =
        SellingResource::try_deserialize(&mut selling_resource_account.data.as_ref()).unwrap();

    assert_eq!(store_keypair.pubkey(), selling_resource_data.store);
    assert_eq!(store_admin.pubkey(), selling_resource_data.owner);
    assert_eq!(token.mint.pubkey(), selling_resource_data.resource);
    assert_eq!(vault.pubkey(), selling_resource_data.vault);
    assert_eq!(vault_owner, selling_resource_data.vault_owner);
    assert_eq!(0, selling_resource_data.supply);
    assert_eq!(max_supply, selling_resource_data.max_supply);
    assert_eq!(SellingResourceState::Created, selling_resource_data.state);
}

#[tokio::test]
async fn failure_supply_is_gt_than_available() {
    let mut context = nft_shop_program_test().start_with_context().await;

    let (store_admin, store_keypair) = create_store(&mut context).await;

    let metadata_creators = Some(vec![Creator {
        address: store_admin.pubkey(),
        verified: false,
        share: 100,
    }]);
    let max_supply = Some(1);
    let metadata_is_mutable = true;

    // Create Token to be provided as a resource for sale
    let token = create_nft(
        &mut context,
        store_admin.dirty_clone(),
        metadata_creators,
        max_supply,
        metadata_is_mutable,
    )
    .await
    .unwrap();

    let (vault_owner, vault_owner_bump) =
        find_vault_owner_address(&token.mint.pubkey(), &store_keypair.pubkey());

    let vault = Keypair::new();
    create_token_account(&mut context, &vault, &token.mint.pubkey(), &vault_owner).await;

    let selling_resource = Keypair::new();

    let data = nft_shop::instruction::InitSellingResource {
        master_edition_bump: token.master_edition.bump,
        vault_owner_bump,
        max_supply: Some(1000), // Real max_suply = 1
    };

    let accounts = nft_shop::accounts::InitSellingResource {
        store: store_keypair.pubkey(),
        store_admin: store_admin.pubkey(),
        selling_resource: selling_resource.pubkey(),
        selling_resource_owner: store_admin.pubkey(),
        resource_mint: token.mint.pubkey(),
        master_edition: token.master_edition.edition,
        metadata: token.metadata,
        vault: vault.pubkey(),
        vault_owner,
        resource_token: token.ata,
        rent: sysvar::rent::id(),
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
        Some(&store_admin.pubkey()),
        &[&store_admin, &selling_resource],
        context.last_blockhash,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::SupplyIsGtThanAvailable.into());
}

#[tokio::test]
async fn failure_supply_is_not_provided() {
    let mut context = nft_shop_program_test().start_with_context().await;

    let (store_admin, store_keypair) = create_store(&mut context).await;

    let metadata_creators = Some(vec![Creator {
        address: store_admin.pubkey(),
        verified: false,
        share: 100,
    }]);
    let max_supply = Some(1);
    let metadata_is_mutable = true;

    // Create Token to be provided as a resource for sale
    let token = create_nft(
        &mut context,
        store_admin.dirty_clone(),
        metadata_creators,
        max_supply,
        metadata_is_mutable,
    )
    .await
    .unwrap();

    let (vault_owner, vault_owner_bump) =
        find_vault_owner_address(&token.mint.pubkey(), &store_keypair.pubkey());

    let vault = Keypair::new();
    create_token_account(&mut context, &vault, &token.mint.pubkey(), &vault_owner).await;

    let selling_resource = Keypair::new();

    let data = nft_shop::instruction::InitSellingResource {
        master_edition_bump: token.master_edition.bump,
        vault_owner_bump,
        max_supply: None, // Real max_suply = 1
    };

    let accounts = nft_shop::accounts::InitSellingResource {
        store: store_keypair.pubkey(),
        store_admin: store_admin.pubkey(),
        selling_resource: selling_resource.pubkey(),
        selling_resource_owner: store_admin.pubkey(),
        resource_mint: token.mint.pubkey(),
        master_edition: token.master_edition.edition,
        metadata: token.metadata,
        vault: vault.pubkey(),
        vault_owner,
        resource_token: token.ata,
        rent: sysvar::rent::id(),
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
        Some(&store_admin.pubkey()),
        &[&store_admin, &selling_resource],
        context.last_blockhash,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ErrorCode::SupplyIsNotProvided.into());
}
