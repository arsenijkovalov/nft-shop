//! Module provide program defined errors

use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // 6000
    #[msg("Name string variable is longer than allowed")]
    NameIsTooLong,
    // 6001
    #[msg("Description string variable is longer than allowed")]
    DescriptionIsTooLong,
    // 6002
    #[msg("Provided supply is gt than available")]
    SupplyIsGtThanAvailable,
    // 6003
    #[msg("Supply is not provided")]
    SupplyIsNotProvided,
    // 6004
    #[msg("Derived key invalid")]
    DerivedKeyInvalid,
    // 6005
    #[msg("PublicKeyMismatch")]
    PublicKeyMismatch,
    // 6006
    #[msg("Pieces in one wallet cannot be greater than Max Supply value")]
    PiecesInOneWalletIsTooMuch,
    // 6007
    #[msg("StartDate cannot be in the past")]
    StartDateIsInPast,
    // 6008
    #[msg("EndDate should not be earlier than StartDate")]
    EndDateIsEarlierThanBeginDate,
    // 6009
    #[msg("Market is not started")]
    MarketIsNotStarted,
    // 6010
    #[msg("Market is ended")]
    MarketIsEnded,
    // 6011
    #[msg("User reach buy limit")]
    UserReachBuyLimit,
    // 6012
    #[msg("Math overflow")]
    MathOverflow,
    // 6013
    #[msg("Supply is gt than max supply")]
    SupplyIsGtThanMaxSupply,
    // 6014
    #[msg("Market duration is not unlimited")]
    MarketDurationIsNotUnlimited,
    // 6015
    #[msg("Market is immutable")]
    MarketIsImmutable,
    // 6016
    #[msg("Market in invalid state")]
    MarketInInvalidState,
    // 6017
    #[msg("Price is zero")]
    PriceIsZero,
    // 6018
    #[msg("Funder is invalid")]
    FunderIsInvalid,
    // 6019
    #[msg("Payout ticket exists")]
    PayoutTicketExists,
    // 6020
    #[msg("Funder provide invalid destination")]
    InvalidFunderDestination,
    // 6021
    #[msg("Treasury is not empty")]
    TreasuryIsNotEmpty,
    // 6022
    #[msg("Selling resource already taken by other market")]
    SellingResourceAlreadyTaken,
    // 6023
    #[msg("Metadata creators is empty")]
    MetadataCreatorsIsEmpty,
    // 6024
    #[msg("User wallet must match user token account")]
    UserWalletMustMatchUserTokenAccount,
    // 6025
    #[msg("Metadata should be mutable")]
    MetadataShouldBeMutable,
    // 6026
    #[msg("Primary sale is not allowed")]
    PrimarySaleIsNotAllowed,
    // 6027
    #[msg("Creators is gt than allowed")]
    CreatorsIsGtThanAvailable,
    // 6028
    #[msg("Creators is empty")]
    CreatorsIsEmpty,
    // 6029
    #[msg("Market owner doesn't receive shares at primary sale")]
    MarketOwnerDoesntHaveShares,
    // 6030
    #[msg("PrimaryMetadataCreatorsNotProvided")]
    PrimaryMetadataCreatorsNotProvided,
}
