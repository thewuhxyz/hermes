#![no_std]
#![allow(unexpected_cfgs)]

mod consts;
mod instructions;
mod state;

use consts::DISCRIMATOR_LENGTH;
use instructions::*;
use pinocchio::{
    account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

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
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
