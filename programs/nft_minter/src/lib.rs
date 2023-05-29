#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token};

pub mod constants;
pub mod instructions;
pub mod pda;
pub mod utils;

use instructions::*;
use utils::*;

declare_id!("5QYUbqZUA7RexK1XWHPaATLCaTgJfX1Nubf2HcS17VLG");

#[program]
pub mod nft_minter {
    use super::*;

    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
        is_mutable: bool,
    ) -> Result<()> {
        create_mint_account(&ctx)?;
        initialize_mint_account(&ctx)?;
        create_metadata_account(
            &ctx,
            name,
            symbol,
            uri,
            into_mpl_creators(creators),
            seller_fee_basis_points,
            is_mutable,
        )?;

        Ok(())
    }

    pub fn mint_token(ctx: Context<MintToken>, max_supply: Option<u64>) -> Result<()> {
        create_associated_token_account(&ctx)?;
        mint_token_to_associated_token_account(&ctx)?;
        create_master_edition_account(&ctx, max_supply)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub mint_account: Signer<'info>,
    pub mint_authority: Signer<'info>,
    pub update_authority: Signer<'info>,
    /// CHECK: Metaplex will check this
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,
    /// CHECK: Metaplex will check this
    pub token_metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub mint_account: Signer<'info>,
    pub mint_authority: Signer<'info>,
    pub update_authority: Signer<'info>,
    /// CHECK: Anchor will check this
    #[account(mut)]
    pub associated_token_account: UncheckedAccount<'info>,
    /// CHECK: Metaplex will check this
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,
    /// CHECK: Metaplex will check this
    #[account(mut)]
    pub edition_account: UncheckedAccount<'info>,
    /// CHECK: Metaplex will check this
    pub token_metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}
