use {
    crate::{state::*},
    anchor_lang::prelude::*,
    solana_program::{pubkey::Pubkey}
};

pub fn handler(ctx: Context<Deposit>, escrow_amt: u64, unlock_price: u64) -> Result<()> {
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