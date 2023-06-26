use crate::*;

// accounts
#[account]
pub struct EscrowState {
    pub unlock_price: u64,
    pub escrow_amt: u64,
    pub bump: u8
}

#[repr(packed)]
#[account(zero_copy(unsafe))]
#[derive(Default)]
pub struct VrfClientState {
    pub bump: u8,
    pub max_result: u64,
    pub result_buffer: [u8; 32],
    pub result: u128,
    pub timestamp: i64,
    pub vrf: Pubkey,
    pub escrow: Pubkey
}

// input params
#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ReadResultParams {
    pub max_confidence_interval: Option<f64>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitVrfClientParams {
    pub max_result: u64,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct RequestRandomnessParams {
    pub permission_bump: u8,
    pub switchboard_state_bump: u8,
}

pub const ESCROW_SEED: &str = "MICHAEL BURRY";
pub const VRF_STATE_SEED: &[u8] = b"VRFCLIENT";
pub static SOL_USDC_FEED: Pubkey = pubkey!("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR");

// events
#[event]
pub struct VrfClientCreated {
    pub vrf_client: Pubkey,
    pub max_result: u64,
    pub timestamp: i64,
}

#[event]
pub struct RandomnessRequested {
    pub vrf_client: Pubkey,
    pub max_result: u64,
    pub timestamp: i64,
}

#[event]
pub struct VrfClientUpdated {
    pub vrf_client: Pubkey,
    pub max_result: u64,
    pub result_buffer: [u8; 32],
    pub result: u128,
    pub timestamp: i64,
}