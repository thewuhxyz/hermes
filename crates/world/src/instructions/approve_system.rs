use crate::state::world::WorldMutate;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

pub fn approve_system(accounts: &[AccountInfo]) -> ProgramResult {
    let [authority, world_acct, system, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let mut world = unsafe { WorldMutate::from_bytes(world_acct.borrow_mut_data_unchecked())? };

    let authorities = world.authorities()?;

    if !authorities.contains(authority.key()) {
        return Err(ProgramError::InvalidAccountData);
    }

    let is_permissionless = world.is_permissionless()?;

    if *is_permissionless {
        *is_permissionless = false
    }

    world.add_system(system.key())?;

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

    Ok(())
}
