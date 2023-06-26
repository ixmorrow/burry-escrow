use crate::*;

pub fn handler(ctx: Context<Withdraw>, params: ReadResultParams) -> Result<()> {
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
    else if escrow_state.out_of_jail {
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
    } else {
        return Err(error!(EscrowErrorCode::InvalidWithdrawalRequest));
    }

    Ok(())
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