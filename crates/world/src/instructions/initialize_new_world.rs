use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::state::{
    registry::{registry_signer, Registry},
    transmutable::TransmutableMut,
    world::{world_pda, WorldMutate, NEW_WORLD_SIZE},
};

pub fn initialize_new_world(accounts: &[AccountInfo]) -> ProgramResult {
    let [payer, world, registry, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let reg = unsafe { Registry::load_mut_unchecked(registry.borrow_mut_data_unchecked())? };

    let (_, bump) = world_pda(&reg.worlds);

    let lamports_needed = Rent::get()?.minimum_balance(NEW_WORLD_SIZE);

    CreateAccount {
        from: payer,
        to: world,
        lamports: lamports_needed,
        space: NEW_WORLD_SIZE as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&[registry_signer(&[bump]).as_slice().into()])?;

    WorldMutate::init_new_world(unsafe { world.borrow_mut_data_unchecked() })?;

    reg.worlds += 1;

    Ok(())
}
