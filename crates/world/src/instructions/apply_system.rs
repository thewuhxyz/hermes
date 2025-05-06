use crate::{
    consts::DISCRIMATOR_LENGTH, error::WorldError, state::world::WorldRef,
    utils::init_execute_cpi_accounts,
};
use core::mem::MaybeUninit;
use pinocchio::{
    account_info::AccountInfo,
    cpi::{get_return_data, MAX_CPI_ACCOUNTS},
    program_error::ProgramError,
    ProgramResult,
};

pub fn apply_system(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [system, authority, instruction_sysvar_account, world_acct, remaining @ ..] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority.is_signer() && authority.key() != &crate::ID {
        return Err(WorldError::InvalidAuthority.into());
    }

    let world = WorldRef::from_account_info(world_acct)?;

    if !world.permissionless && world.systems.binary_search(system.key()).is_err() {
        return Err(WorldError::SystemNotApproved.into());
    }

    const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();

    let mut ctx_accounts = [UNINIT_INFO; MAX_CPI_ACCOUNTS];

    let (components, sep_idx, remaining_accounts) =
        init_execute_cpi_accounts(remaining, &mut ctx_accounts)?;

    let mut cpi_data = [0u8; 1024];

    cpi_data[0..DISCRIMATOR_LENGTH]
        .copy_from_slice(hermes_cpi_interface::system::Execute::DISCRIMINATOR.as_slice());
    cpi_data[DISCRIMATOR_LENGTH..DISCRIMATOR_LENGTH + data.len()].copy_from_slice(data);

    hermes_cpi_interface::system::Execute {
        authority,
        components,
        remaining_accounts,
        instruction_data: &cpi_data[..DISCRIMATOR_LENGTH + data.len()],
        system: system.key(),
    }
    .invoke()?;

    let return_data = get_return_data().ok_or(ProgramError::InvalidAccountData)?;

    let components_pair = &remaining[..sep_idx.unwrap_or(remaining.len())];

    let (_, data) = return_data.as_slice().split_at(core::mem::size_of::<u32>());

    let mut cursor = 0;

    for pair in components_pair.chunks_exact(2) {
        let [component_program, component] = pair else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        let mut size = core::mem::size_of::<u32>();

        let len_bytes = &data[cursor..cursor + size];

        let len =
            u32::from_le_bytes(unsafe { (len_bytes.as_ptr() as *const [u8; 4]).read() }) as usize;

        size += len;

        let mut instruction_data = [0u8; 256 + DISCRIMATOR_LENGTH];

        instruction_data[0..DISCRIMATOR_LENGTH]
            .copy_from_slice(hermes_cpi_interface::component::Update::DISCRIMINATOR.as_slice());
        instruction_data[DISCRIMATOR_LENGTH..DISCRIMATOR_LENGTH + size]
            .copy_from_slice(&data[cursor..cursor + size]);

        cursor += size;

        hermes_cpi_interface::component::Update {
            authority,
            component,
            component_program: component_program.key(),
            instruction_data: &instruction_data[..DISCRIMATOR_LENGTH + size],
            instruction_sysvar_account,
        }
        .invoke()?;
    }

    Ok(())
}
