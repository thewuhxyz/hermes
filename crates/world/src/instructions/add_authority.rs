use crate::state::world::WorldMutate;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

pub fn add_authority(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [authority, new_authority, world_acct, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let _world_id: &u64 = unsafe { &*(data.as_ptr().read_unaligned() as *const u64) };

    let mut world = unsafe { WorldMutate::from_bytes(world_acct.borrow_mut_data_unchecked())? };

    let authorities = world.authorities()?;

    if authorities.is_empty()
        || (authorities.contains(authority.key()) && !authorities.contains(new_authority.key()))
    {
        world.add_new_authority(new_authority.key())?;

        let world_size = world.size()?;
        let rent = Rent::get()?;
        let new_minimum_balance = rent.minimum_balance(world_size);

        let lamports_diff = new_minimum_balance.saturating_sub(world_acct.lamports());

        if lamports_diff > 0 {
            pinocchio_system::instructions::Transfer {
                lamports: lamports_diff,
                from: authority,
                to: world_acct,
            }
            .invoke()?;
        }

        world_acct.realloc(world_size, false)?;
    }

    Ok(())
}
