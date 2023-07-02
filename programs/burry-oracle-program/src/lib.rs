pub mod instructions;
pub mod state;
pub mod errors;

pub use {
    anchor_lang::prelude::*,
    anchor_lang::solana_program::{pubkey::Pubkey, pubkey, clock},
    switchboard_v2::{AggregatorAccountData, SwitchboardDecimal, VrfAccountData,
        OracleQueueAccountData, PermissionAccountData, SbState, VrfRequestRandomness
    },
    anchor_spl::token::{Token, TokenAccount},
    std::convert::TryInto,
    state::*,
    instructions::*,
    errors::*,
    bytemuck::*
};


declare_id!("3yU8tgZeBoaTfcqReY6LeDQcekMAnQ1DiwKvmxKPUncb");

#[program]
mod burry_oracle_program {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>, escrow_amt: u64, unlock_price: u64) -> Result<()> {
        deposit::handler(ctx, escrow_amt, unlock_price)
    }

    pub fn withdraw(ctx: Context<Withdraw>, params: ReadResultParams) -> Result<()> {
        withdraw::handler(ctx, params)
    }

    pub fn withdraw_closed_feed_funds(ctx: Context<ClaimEscrowedFunds>) -> Result <()> {
        withdraw_closed_feed::handler(ctx)
    }

    pub fn init_vrf_client(ctx: Context<InitVrfClient>, vrf_params: InitVrfClientParams) -> Result <()> {
        init_vrf_client::handler(ctx, vrf_params)
    }

    pub fn get_out_of_jail_random(ctx: Context<RequestRandomness>, request_params: RequestRandomnessParams) -> Result<()> {
        get_out_of_jail::handler(ctx, request_params)
    }

    // vrf callback instruction
    pub fn consume_randomness(ctx: Context<ConsumeRandomness>) -> Result<()> {
        consume_randomness::handler(ctx)
    }

    pub fn paid_withdraw(ctx: Context<PaidWithdraw>) -> Result<()> {
        paid_withdraw::handler(ctx)
    }
}