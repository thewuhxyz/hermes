use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::state::{registry::Registry, transmutable::Transmutable};

pub fn initialize_registry(accounts: &[AccountInfo]) -> ProgramResult {
    let [registry, payer, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let lamports_needed = Rent::get()?.minimum_balance(Registry::LEN);

    let (_, bump) = Registry::pda();

    CreateAccount {
        from: payer,
        to: registry,
        lamports: lamports_needed,
        space: Registry::LEN as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&[Registry::signer(&[bump]).as_slice().into()])?;

    Registry::init(unsafe { registry.borrow_mut_data_unchecked() })?;

    Ok(())
}
