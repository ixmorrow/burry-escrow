use {
    crate::{state::*, errors::*},
    anchor_lang::prelude::*
};

pub fn handler(ctx: Context<ClaimEscrowedFunds>) -> Result <()> {

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