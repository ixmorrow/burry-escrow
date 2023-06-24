use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use std::convert::TryInto;
use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal};
use solana_program::{pubkey, pubkey::Pubkey};

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
        let mut valid_transfer: bool = false;

        // check feed does not exceed max_confidence_interval
        if let Some(max_confidence_interval) = params.max_confidence_interval {
            feed.check_confidence_interval(SwitchboardDecimal::from_f64(max_confidence_interval))
                .map_err(|_| error!(EscrowErrorCode::ConfidenceIntervalExceeded))?;
        }

        msg!("Current feed result is {}!", val);
        msg!("Unlock price is {}", escrow_state.unlock_price);

        if val > escrow_state.unlock_price as f64 {
            valid_transfer = true;
        }
        else if (clock::Clock::get().unwrap().unix_timestamp - feed.latest_confirmed_round.round_open_timestamp) > 86400 {
            valid_transfer = true;
        }
        else if **ctx.accounts.feed_aggregator.to_account_info().try_borrow_lamports()? == 0 {
            valid_transfer = true;
        }
        
        if valid_transfer{
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
        }

        Ok(())
    }

    pub fn withdraw_closed_feed_funds(ctx: Context<ClaimEscrowedFunds>) -> Result <()> {

        let escrow_state = &ctx.accounts.escrow_account;
        let user = &ctx.accounts.user;

        msg!("Feed account lamports: {}", **ctx.accounts.closed_feed_account.try_borrow_lamports()?);

        **escrow_state.to_account_info().try_borrow_mut_lamports()? = escrow_state
            .to_account_info()
            .lamports()
            .checked_sub(escrow_state.escrow_amt)
            .ok_or(ProgramError::InvalidArgument)?;

        **user.to_account_info().try_borrow_mut_lamports()? = user
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
    #[account(
        address = SOL_USDC_FEED
    )]
    pub feed_aggregator: AccountLoader<'info, AggregatorAccountData>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimEscrowedFunds<'info> {
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
    /// CHECK: comment out the address=SOL_USDC_FEED to test this instruction
    #[account(
        address = SOL_USDC_FEED,
        constraint = **closed_feed_account.to_account_info().try_borrow_lamports()? == 0
        @ EscrowErrorCode::FeedAccountIsNotClosed
    )]
    pub closed_feed_account: AccountInfo<'info>
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
pub static SOL_USDC_FEED: Pubkey = pubkey!("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR");


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
    #[msg("Feed account is not closed, must be closed to redeem with the withdraw_closed_feed_funds instruction.")]
    FeedAccountIsNotClosed
}