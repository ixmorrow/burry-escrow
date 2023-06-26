use crate::*;

pub fn handler(ctx: Context<InitVrfClient>, vrf_params: InitVrfClientParams) -> Result<()> {
    msg!("init_client validate");
    if vrf_params.max_result > 3 {
        return Err(error!(EscrowErrorCode::MaxResultExceedsMaximum));
    }

    let mut vrf_state = ctx.accounts.vrf_state.load_init()?;
    *vrf_state = VrfClientState::default();
    vrf_state.bump = ctx.bumps.get("vrf_state").unwrap().clone();
    vrf_state.vrf = ctx.accounts.vrf.key();
    vrf_state.escrow = ctx.accounts.escrow_account.key();
    vrf_state.die_result_1 = 0;
    vrf_state.die_result_2 = 0;

    if vrf_params.max_result == 0 {
        vrf_state.max_result = 3;
    } else {
        vrf_state.max_result = vrf_params.max_result;
    }

    emit!(VrfClientCreated{
        vrf_client: ctx.accounts.vrf_state.key(),
        max_result: vrf_params.max_result,
        timestamp: clock::Clock::get().unwrap().unix_timestamp
    });

    drop(vrf_state);

    Ok(())
}

#[derive(Accounts)]
#[instruction(vrf_params: InitVrfClientParams)]
pub struct InitVrfClient<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    // burry escrow account
    #[account(
        mut,
        seeds = [user.key().as_ref(), ESCROW_SEED.as_bytes()],
        bump,
    )]
    pub escrow_account: Account<'info, EscrowState>,
    // vrf client state
    #[account(
        init,
        seeds = [
            user.key.as_ref(),
            escrow_account.key().as_ref(),
            vrf.key().as_ref(),
            VRF_STATE_SEED
        ],
        payer = user,
        space = 8 + std::mem::size_of::<VrfClientState>(),
        bump
    )]
    pub vrf_state: AccountLoader<'info, VrfClientState>,
    // switchboard vrf account
    #[account(
        mut,
        constraint = vrf.load()?.authority == vrf_state.key() @ EscrowErrorCode::InvalidVrfAuthorityError
    )]
    pub vrf: AccountLoader<'info, VrfAccountData>,
    pub system_program: Program<'info, System>
}