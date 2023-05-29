use anchor_lang::prelude::Pubkey;

use crate::id;
use crate::utils::{
    HISTORY_PREFIX, HOLDER_PREFIX, PAYOUT_TICKET_PREFIX, PRIMARY_METADATA_CREATORS_PREFIX,
    VAULT_OWNER_PREFIX,
};

pub fn find_vault_owner_address(resource_mint: &Pubkey, store: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            VAULT_OWNER_PREFIX.as_bytes(),
            resource_mint.as_ref(),
            store.as_ref(),
        ],
        &id(),
    )
}

pub fn find_treasury_owner_address(
    treasury_mint: &Pubkey,
    selling_resource: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            HOLDER_PREFIX.as_bytes(),
            treasury_mint.as_ref(),
            selling_resource.as_ref(),
        ],
        &id(),
    )
}

pub fn find_primary_metadata_creators(metadata: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PRIMARY_METADATA_CREATORS_PREFIX.as_bytes(),
            metadata.as_ref(),
        ],
        &id(),
    )
}

pub fn find_trade_history_address(wallet: &Pubkey, market: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[HISTORY_PREFIX.as_bytes(), wallet.as_ref(), market.as_ref()],
        &id(),
    )
}

pub fn find_payout_ticket_address(market: &Pubkey, funder: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PAYOUT_TICKET_PREFIX.as_bytes(),
            market.as_ref(),
            funder.as_ref(),
        ],
        &id(),
    )
}
