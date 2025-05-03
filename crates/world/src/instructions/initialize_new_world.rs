use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::state::{
    registry::Registry,
    transmutable::TransmutableMut,
    world::{World, WorldMut},
};

pub fn initialize_new_world(accounts: &[AccountInfo]) -> ProgramResult {
    let [payer, world, registry, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let reg = unsafe { Registry::load_mut_unchecked(registry.borrow_mut_data_unchecked())? };

    if Registry::seeds() != *registry.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    let (_, bump) = World::pda(&reg.worlds.to_le_bytes());

    let lamports_needed = Rent::get()?.minimum_balance(World::INIT_SIZE);

    CreateAccount {
        from: payer,
        to: world,
        lamports: lamports_needed,
        space: World::INIT_SIZE as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&[World::signer(&reg.worlds.to_be_bytes(), &[bump])
        .as_slice()
        .into()])?;

    WorldMut::init_new_world(unsafe { world.borrow_mut_data_unchecked() })?;

    reg.worlds += 1;

    Ok(())
}
