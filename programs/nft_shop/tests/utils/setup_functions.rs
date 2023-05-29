#![allow(dead_code)]

use anchor_client::solana_sdk::{
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use anchor_lang::{
    prelude::Pubkey,
    solana_program::{
        instruction::{Instruction, InstructionError},
        system_program, sysvar,
    },
    InstructionData, ToAccountMetas,
};
use anchor_spl::associated_token::get_associated_token_address;
use nft_minter::{
    pda::{find_master_edition_account, find_metadata_account},
    utils::{token_metadata_program_id, Creator},
};
use nft_shop::pda::find_vault_owner_address;
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext};

use crate::utils::helpers::airdrop;

use super::helpers::{create_token_account, DirtyClone};

pub const ONE_SOL: u64 = 1_000_000_000;

pub fn assert_error(error: BanksClientError, expected_error: u32) {
    match error {
        BanksClientError::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::Custom(e),
        )) => assert_eq!(e, expected_error),
        _ => assert!(false),
    }
}

pub fn nft_shop_program_test() -> ProgramTest {
    let mut program = ProgramTest::new("nft_shop", nft_shop::id(), None);
    program.add_program("nft_minter", nft_minter::id(), None);
    program.add_program("mpl_token_metadata", token_metadata_program_id(), None);
    program
}

pub async fn create_store(context: &mut ProgramTestContext) -> (Keypair, Keypair) {
    let store_admin = Keypair::new();
    airdrop(context, &store_admin.pubkey(), 10_000_000_000)
        .await
        .unwrap();

    let store = Keypair::new();

    let name = "Test store".to_string();
    let description = "Just a test store".to_string();

    let data = nft_shop::instruction::CreateStore {
        name: name.clone(),
        description: description.clone(),
    };

    let accounts = nft_shop::accounts::CreateStore {
        store_admin: store_admin.pubkey(),
        store: store.pubkey(),
        system_program: system_program::id(),
    };

    let instruction = Instruction {
        program_id: nft_shop::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &store_admin, &store],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    (store_admin, store)
}

#[derive(Debug)]
pub struct MasterEdition {
    pub edition: Pubkey, // Metaplex Master Edition
    pub bump: u8,        // Metaplex Master Edition bump
}

#[derive(Debug)]
pub struct NFT {
    pub mint: Keypair,                 // Resource Mint
    pub owner: Keypair,                // Resource Owner
    pub ata: Pubkey,                   // Resource Token
    pub metadata: Pubkey,              // Metaplex Metadata
    pub master_edition: MasterEdition, // Master Edition
}

pub async fn create_nft(
    context: &mut ProgramTestContext,
    owner: Keypair,
    metadata_creators: Option<Vec<Creator>>,
    max_supply: Option<u64>,
    is_mutable: bool,
) -> Result<NFT, BanksClientError> {
    let mint = Keypair::new();

    let ata = get_associated_token_address(&owner.pubkey(), &mint.pubkey());
    let (metadata, _) = find_metadata_account(&mint.pubkey());
    let (master_edition, master_edition_bump) = find_master_edition_account(&mint.pubkey());

    // CreateToken
    let create_token_ix = Instruction {
        program_id: nft_minter::id(),
        data: nft_minter::instruction::CreateToken {
            name: "Solana Course NFT".to_string(),
            symbol: "SOLC".to_string(),
            uri: "https://raw.githubusercontent.com/arsenijkovalov/nft-assets/main/assets/nft.json"
                .to_string(),
            creators: metadata_creators,
            seller_fee_basis_points: 10,
            is_mutable,
        }
        .data(),
        accounts: nft_minter::accounts::CreateToken {
            payer: owner.pubkey(),
            mint_account: mint.pubkey(),
            mint_authority: owner.pubkey(),
            update_authority: owner.pubkey(),
            metadata_account: metadata,
            token_metadata_program: token_metadata_program_id(),
            system_program: system_program::id(),
            token_program: anchor_spl::token::ID,
            rent: sysvar::rent::id(),
        }
        .to_account_metas(None),
    };

    // MintToken
    let mint_token_ix = Instruction {
        program_id: nft_minter::id(),
        data: nft_minter::instruction::MintToken { max_supply }.data(),
        accounts: nft_minter::accounts::MintToken {
            payer: owner.pubkey(),
            mint_account: mint.pubkey(),
            mint_authority: owner.pubkey(),
            update_authority: owner.pubkey(),
            associated_token_account: ata,
            metadata_account: metadata,
            edition_account: master_edition,
            token_metadata_program: token_metadata_program_id(),
            system_program: system_program::id(),
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            rent: sysvar::rent::id(),
        }
        .to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_token_ix, mint_token_ix],
        Some(&owner.pubkey()),
        &[&mint, &owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await?;

    Ok(NFT {
        mint,
        owner,
        ata,
        metadata,
        master_edition: MasterEdition {
            edition: master_edition,
            bump: master_edition_bump,
        },
    })
}

pub async fn init_selling_resource(
    context: &mut ProgramTestContext,
    store_admin: &Keypair,
    store_keypair: &Keypair,
    selling_resource_owner_keypair: &Keypair,
    creators: Option<Vec<Creator>>,
    max_supply: Option<u64>,
    is_mutable: bool,
) -> (Keypair, Keypair, NFT) {
    // Create Token to be provided as a resource for sale
    let token = create_nft(
        context,
        store_admin.dirty_clone(),
        creators,
        max_supply,
        is_mutable,
    )
    .await
    .unwrap();

    let (vault_owner, vault_owner_bump) =
        find_vault_owner_address(&token.mint.pubkey(), &store_keypair.pubkey());

    let vault = Keypair::new();
    create_token_account(context, &vault, &token.mint.pubkey(), &vault_owner).await;

    let data = nft_shop::instruction::InitSellingResource {
        master_edition_bump: token.master_edition.bump,
        vault_owner_bump,
        max_supply,
    };

    let selling_resource_keypair = Keypair::new();

    let accounts = nft_shop::accounts::InitSellingResource {
        store: store_keypair.pubkey(),
        store_admin: store_admin.pubkey(),
        selling_resource: selling_resource_keypair.pubkey(),
        selling_resource_owner: selling_resource_owner_keypair.pubkey(),
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
        &[&store_admin, &selling_resource_keypair],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    (selling_resource_keypair, vault, token)
}
