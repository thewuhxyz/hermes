use crate::state::{
    entity::Entity,
    world::{World, WorldMut},
};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

pub fn add_entity(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [payer, entity_acct, world_acct, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let world = WorldMut::from_account_info(world_acct)?;

    let (_, bump) = Entity::pda(
        &world.world_metadata.id.to_be_bytes(),
        &world.world_metadata.entities.to_be_bytes(),
        data,
    )?;

    let lamports_needed = Rent::get()?.minimum_balance(World::INIT_SIZE);

    CreateAccount {
        from: payer,
        to: entity_acct,
        lamports: lamports_needed,
        space: World::INIT_SIZE as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&[Entity::signer(
        &world.world_metadata.id.to_be_bytes(),
        &world.world_metadata.entities.to_be_bytes(),
        data,
        &[bump],
    )?
    .as_slice()
    .into()])?;

    Entity::init(
        unsafe { entity_acct.borrow_mut_data_unchecked() },
        world.world_metadata.entities,
    )?;

    world.world_metadata.entities += 1;

    Ok(())
}
