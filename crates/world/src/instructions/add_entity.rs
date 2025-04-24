use crate::state::{entity::Entity, transmutable::TransmutableMut, world::WorldMutate};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

pub fn add_entity(accounts: &[AccountInfo]) -> ProgramResult {
    let [_payer, entity_acct, world_acct, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let world = unsafe { WorldMutate::from_bytes(world_acct.borrow_mut_data_unchecked())? };
    let entity = unsafe { Entity::load_mut_unchecked(entity_acct.borrow_mut_data_unchecked())? };

    entity.id = world.world_metadata.entities;
    world.world_metadata.entities += 1;

    Ok(())
}
