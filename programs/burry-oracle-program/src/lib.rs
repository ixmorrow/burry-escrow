pub mod instructions;
pub mod state;
pub mod errors;

use {anchor_lang::prelude::*, instructions::*, state::*};

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
}