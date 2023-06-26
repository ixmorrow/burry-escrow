use anchor_lang::prelude::*;

#[error_code]
#[derive(Eq, PartialEq)]
pub enum EscrowErrorCode {
    #[msg("Switchboard feed has not been updated in 5 minutes")]
    StaleFeed,
    #[msg("Switchboard feed exceeded provided confidence interval")]
    ConfidenceIntervalExceeded,
    #[msg("Current SOL price is not above Escrow unlock price.")]
    SolPriceAboveUnlockPrice,
    #[msg("Feed account is not closed, must be closed to redeem with the withdraw_closed_feed_funds instruction.")]
    FeedAccountIsNotClosed,
    #[msg("Switchboard VRF Account's authority should be set to the client's state pubkey")]
    InvalidVrfAuthorityError,
    #[msg("The max result must not exceed u64")]
    MaxResultExceedsMaximum,
    #[msg("Invalid VRF account provided.")]
    InvalidVrfAccount,
    #[msg("Not a valid Switchboard account")]
    InvalidSwitchboardAccount,
    #[msg("Invalid withdrawal request")]
    InvalidWithdrawalRequest,
}