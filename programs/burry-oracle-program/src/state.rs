use crate::*;


#[account]
pub struct EscrowState {
    pub unlock_price: u64,
    pub escrow_amt: u64,
    pub bump: u8
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ReadResultParams {
    pub max_confidence_interval: Option<f64>,
}

pub const ESCROW_SEED: &str = "MICHAEL BURRY";
pub static SOL_USDC_FEED: Pubkey = pubkey!("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR");