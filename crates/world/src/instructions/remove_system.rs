use crate::state::world::WorldMut;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

pub fn remove_system(accounts: &[AccountInfo]) -> ProgramResult {
    let [authority, world_acct, system, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let mut world = unsafe { WorldMut::from_bytes(world_acct.borrow_mut_data_unchecked())? };

    let authorities = world.authorities()?;

    if !authorities.contains(authority.key()) {
        return Err(ProgramError::InvalidAccountData);
    }

    let size = world.remove_system(system.key())?;

    if size > 0 {
        let world_size = world.size()?;
        let rent = Rent::get()?;
        let new_minimum_balance = rent.minimum_balance(world_size);

        let lamports_diff = world_acct.lamports().saturating_sub(new_minimum_balance);

        if lamports_diff > 0 {
            unsafe {
                *world_acct.borrow_mut_lamports_unchecked() -= lamports_diff;
                *authority.borrow_mut_lamports_unchecked() += lamports_diff
            }
        }

        world_acct.realloc(world_size, false)?
    }

    Ok(())
}
