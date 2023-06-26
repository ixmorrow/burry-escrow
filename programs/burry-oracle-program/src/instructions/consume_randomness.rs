use crate::*;

pub fn handler(ctx: Context<ConsumeRandomness>) -> Result <()> {
    msg!("Successfully consumed randomness.");

    let vrf = ctx.accounts.vrf.load()?;
    let result_buffer = vrf.get_result()?;

    if result_buffer == [0u8; 32] {
        msg!("vrf buffer empty");
        return Ok(());
    }

    let vrf_state = &mut ctx.accounts.vrf_state.load_mut()?;
    let max_result = vrf_state.max_result;
    if result_buffer == vrf_state.result_buffer {
        msg!("result_buffer unchanged");
        return Ok(());
    }

    msg!("Result buffer is {:?}", result_buffer);
    let value: &[u128] = bytemuck::cast_slice(&result_buffer[..]);
    msg!("u128 buffer {:?}", value);
    let result = value[0] % max_result as u128 + 1;
    msg!("Current VRF Value [1 - {}) = {}!", max_result, result);

    if vrf_state.result != result {
        msg!("Updating VRF State with random value...");
        vrf_state.result_buffer = result_buffer;
        vrf_state.result = result;
        vrf_state.timestamp = clock::Clock::get().unwrap().unix_timestamp;

        emit!(VrfClientUpdated {
            vrf_client: ctx.accounts.vrf_state.key(),
            max_result: vrf_state.max_result,
            result: vrf_state.result,
            result_buffer: result_buffer,
            timestamp: vrf_state.timestamp,
        });
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ConsumeRandomness<'info> {
    /// CHECK: 
    // #[account(mut)]
    // pub user: AccountInfo<'info>,
    // burry escrow account
    #[account(
        mut,
        // seeds = [user.key().as_ref(), ESCROW_SEED.as_bytes()],
        // bump,
    )]
    pub escrow_account: Account<'info, EscrowState>,
    // vrf client state
    #[account(mut)]
    pub vrf_state: AccountLoader<'info, VrfClientState>,
    // switchboard vrf account
    #[account(
        mut,
        constraint = vrf.load()?.authority == vrf_state.key() @ EscrowErrorCode::InvalidVrfAuthorityError
    )]
    pub vrf: AccountLoader<'info, VrfAccountData>
}