use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::find_program_address,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::state::{
    registry::{registry_signer, Registry},
    transmutable::Transmutable,
};

pub fn initialize_registry(accounts: &[AccountInfo]) -> ProgramResult {
    let [registry, payer, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let lamports_needed = Rent::get()?.minimum_balance(Registry::LEN);

    let (_, bump) = find_program_address(&[Registry::seeds()], &crate::ID);

    CreateAccount {
        from: payer,
        to: registry,
        lamports: lamports_needed,
        space: Registry::LEN as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&[registry_signer(&[bump]).as_slice().into()])?;

    Ok(())
}
