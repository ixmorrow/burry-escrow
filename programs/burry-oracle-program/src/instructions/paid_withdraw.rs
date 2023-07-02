use crate::*;

pub fn handler(ctx: Context<PaidWithdraw>) -> Result<()> {
    let escrow_state = &ctx.accounts.escrow_account;
    
    // transfer 50 lamports to program vault
    let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.user.key(),
        &ctx.accounts.program_vault.key(),
        50
    );

    anchor_lang::solana_program::program::invoke(
        &transfer_ix,
        &[
            ctx.accounts.user.to_account_info(),
            ctx.accounts.program_vault.to_account_info(),
            ctx.accounts.system_program.to_account_info()
        ]
    )?;

    // transfer lamports back to user
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

#[derive(Accounts)]
pub struct PaidWithdraw<'info> {
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
    /// CHECK: progrma vault account
    #[account(
        seeds = [VAULT_SEED.as_bytes()],
        bump
    )]
    pub program_vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}