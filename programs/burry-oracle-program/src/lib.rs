use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use std::convert::TryInto;
use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal, SWITCHBOARD_PROGRAM_ID};

declare_id!("3yU8tgZeBoaTfcqReY6LeDQcekMAnQ1DiwKvmxKPUncb");

#[program]
mod burry_oracle_program {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>, escrow_amt: u64, unlock_price: u64) -> Result<()> {
        msg!("Depositing funds in escrow...");

        let escrow_state = &mut ctx.accounts.escrow_account;
        escrow_state.unlock_price = unlock_price;
        escrow_state.escrow_amt = escrow_amt;
        escrow_state.bump = *ctx.bumps.get("escrow_account").unwrap();


        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &escrow_state.key(),
            escrow_amt
        );

        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.system_program.to_account_info()
            ]
        )?;

        msg!("Transfer complete. Escrow will unlock SOL at {}", &ctx.accounts.escrow_account.unlock_price);

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, params: ReadResultParams) -> Result<()> {
        let feed = &ctx.accounts.feed_aggregator.load()?;
        let escrow_state = &ctx.accounts.escrow_account;

        // get result
        let val: f64 = feed.get_result()?.try_into()?;

        // check whether the feed has been updated in the last 300 seconds
        feed.check_staleness(clock::Clock::get().unwrap().unix_timestamp, 300)
        .map_err(|_| error!(EscrowErrorCode::StaleFeed))?;

        // check feed does not exceed max_confidence_interval
        if let Some(max_confidence_interval) = params.max_confidence_interval {
            feed.check_confidence_interval(SwitchboardDecimal::from_f64(max_confidence_interval))
                .map_err(|_| error!(EscrowErrorCode::ConfidenceIntervalExceeded))?;
        }

        msg!("Current feed result is {}!", val);
        msg!("Unlock price is {}", escrow_state.unlock_price);

        if val < escrow_state.unlock_price as f64 {
            return Err(EscrowErrorCode::SolPriceAboveUnlockPrice.into())
        }

        **escrow_state.to_account_info().try_borrow_mut_lamports()? = escrow_state
            .to_account_info()
            .lamports()
            .checked_sub(escrow_state.escrow_amt)
            .ok_or(ProgramError::InvalidArgument)?;

        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? = ctx.accounts.user
            .to_account_info()
            .lamports()
            .checked_add(escrow_state.escrow_amt)
            .ok_or(ProgramError::InvalidArgument)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    // user account
    #[account(mut)]
    pub user: Signer<'info>,
    // account to store SOL in escrow
    #[account(
        init,
        seeds = [user.key().as_ref(), ESCROW_SEED.as_bytes()],
        bump,
        payer = user,
        space = 8 + 8 + 8 + 1
    )]
    pub escrow_account: Account<'info, EscrowState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: ReadResultParams)]
pub struct Withdraw<'info> {
    // user account
    #[account(mut)]
    pub user: Signer<'info>,
    // escrow account
    #[account(
        mut,
        seeds = [user.key().as_ref(), ESCROW_SEED.as_bytes()],
        bump,
        close = user
    )]
    pub escrow_account: Account<'info, EscrowState>,
    // Switchboard SOL feed aggregator
    #[account(owner = SWITCHBOARD_PROGRAM_ID)]
    pub feed_aggregator: AccountLoader<'info, AggregatorAccountData>,
    pub system_program: Program<'info, System>,
}



#[account]
pub struct EscrowState {
    unlock_price: u64,
    escrow_amt: u64,
    bump: u8
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ReadResultParams {
    pub max_confidence_interval: Option<f64>,
}

pub const ESCROW_SEED: &str = "MICHAEL BURRY";

#[error_code]
#[derive(Eq, PartialEq)]
pub enum EscrowErrorCode {
    #[msg("Not a valid Switchboard account")]
    InvalidSwitchboardAccount,
    #[msg("Switchboard feed has not been updated in 5 minutes")]
    StaleFeed,
    #[msg("Switchboard feed exceeded provided confidence interval")]
    ConfidenceIntervalExceeded,
    #[msg("Current SOL price is not above Escrow unlock price.")]
    SolPriceAboveUnlockPrice,
}