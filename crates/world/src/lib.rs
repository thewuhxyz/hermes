// #![no_std]
#![allow(unexpected_cfgs)]

mod consts;
mod instructions;
mod state;
mod utils;

use consts::DISCRIMATOR_LENGTH;
use instructions::*;
use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult};

pinocchio_pubkey::declare_id!("WorLD15A7CrDwLcLy4fRqtaTb9fbd8o8iqiEMUDse2n");

pinocchio::entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (instruction_bytes, data) = instruction_data.split_at(DISCRIMATOR_LENGTH);

    let instruction = unsafe { &*(instruction_bytes.as_ptr() as *const _ as *const u64) };

    match get_instruction(instruction)? {
        WorldInstruction::InitializeRegistry => initialize_registry(accounts),
        WorldInstruction::InitializeNewWorld => initialize_new_world(accounts),
        WorldInstruction::AddAuthority => add_authority(accounts, data),
        WorldInstruction::RemoveAuthority => remove_authority(accounts, data),
        WorldInstruction::InitilizeComponent => initialize_component(accounts),
        WorldInstruction::DestroyComponent => destroy_component(accounts),
        WorldInstruction::ApproveSystem => approve_system(accounts),
        WorldInstruction::RemoveSystem => remove_system(accounts),
        WorldInstruction::Apply => apply_system(accounts, data),
        WorldInstruction::ApplyWithSession => apply_system_session(accounts, data),
        WorldInstruction::AddEntity => add_entity(accounts),
        // _ => Err(ProgramError::InvalidInstructionData),
    }
}
