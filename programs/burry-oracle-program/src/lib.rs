pub mod instructions;
pub mod state;
pub mod errors;

pub use {
    anchor_lang::prelude::*,
    anchor_lang::solana_program::{pubkey::Pubkey, pubkey, clock},
    switchboard_v2::{AggregatorAccountData, SwitchboardDecimal, OracleQueueAccountData, SbState},
    anchor_spl::token::{Token, TokenAccount},
    std::convert::TryInto,
    state::*,
    instructions::*,
    errors::*
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
}