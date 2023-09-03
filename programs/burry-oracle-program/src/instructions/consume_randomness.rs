use crate::*;

pub fn handler(ctx: Context<ConsumeRandomness>) -> Result <()> {
    msg!("Consuming randomness...");

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
    let dice_1 = value[0] % max_result as u128 + 1;
    let dice_2 = value[1] % max_result as u128 + 1;

    msg!("Current Die 1 Value [1 - {}) = {}!", max_result, dice_1);
    msg!("Current Die 2 Value [1 - {}) = {}!", max_result, dice_2);
    msg!("Roll total: {}", dice_1 + dice_2);

    msg!("Updating VRF State with random value...");
    vrf_state.result_buffer = result_buffer;
    vrf_state.die_result_1 = dice_1;
    vrf_state.die_result_2 = dice_2;
    vrf_state.timestamp = clock::Clock::get().unwrap().unix_timestamp;
    vrf_state.roll_total = dice_1 + dice_2;

    if dice_1 == dice_2 {
        msg!("Rolled snake eyes, get out of jail free!");
        let escrow_state = &mut ctx.accounts.escrow_account;
        escrow_state.out_of_jail = true;
    }

	Ok(())
}

#[derive(Accounts)]
pub struct ConsumeRandomness<'info> {
    // burry escrow account
    #[account(mut)]
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