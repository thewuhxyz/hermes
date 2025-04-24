use crate::utils::init_execute_cpi_accounts;
use core::mem::MaybeUninit;
use pinocchio::{
    account_info::AccountInfo,
    cpi::{get_return_data, MAX_CPI_ACCOUNTS},
    program_error::ProgramError,
    ProgramResult,
};

pub fn apply_system_session(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [system, authority, instruction_sysvar_account, _world, session_token, remaining @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();

    let mut ctx_accounts = [UNINIT_INFO; MAX_CPI_ACCOUNTS];

    let (components, sep_idx, remaining_accounts) =
        init_execute_cpi_accounts(remaining, &mut ctx_accounts)?;

    hermes_cpi_interface::system::Execute {
        authority,
        components,
        remaining_accounts,
        instruction_data: data,
        system: system.key(),
    }
    .invoke()?;

    let return_data = get_return_data().ok_or(ProgramError::InvalidAccountData)?;

    let components_pair = &remaining[..sep_idx.unwrap_or(remaining.len())];

    // let offset = core::mem::size_of::<u32>();

    let (_, data) = return_data.as_slice().split_at(core::mem::size_of::<u32>());

    let mut cursor = 0;

    for pair in components_pair.chunks_exact(2) {
        let [component_program, component] = pair else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        let mut size = core::mem::size_of::<u32>();

        let len_bytes = &data[cursor..cursor + size];

        let len = unsafe { (len_bytes.as_ptr() as *const u32).read_unaligned() } as usize;

        size += len;

        let instruction_data = &data[cursor..cursor + size];

        cursor += size;

        hermes_cpi_interface::component::UpdateWithSession {
            authority,
            component,
            component_program: component_program.key(),
            instruction_data,
            instruction_sysvar_account,
            session_token,
        }
        .invoke()?;
    }

    Ok(())
}
