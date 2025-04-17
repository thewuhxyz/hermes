mod initialize_registry;
pub use initialize_registry::*;

use pinocchio::program_error::ProgramError;

// will include anchor discrimator later in u64
pub const INITIALIZE_REGISTRY_DISCRIMINATOR: u64 = 1000;

#[repr(u64)]
pub enum WorldInstruction {
    InitializeRegistry = INITIALIZE_REGISTRY_DISCRIMINATOR,
    InitializeNewWorld,
    AddAuthority,
    RemoveAuthority,
    ApproveSystem,
    RemoveSystem,
    AddEntity,
    InitilizeComponent,
    Apply,
}

impl TryFrom<&u64> for WorldInstruction {
    type Error = ProgramError;

    fn try_from(byte: &u64) -> Result<Self, Self::Error> {
        match byte {
            1000 => Ok(WorldInstruction::InitializeRegistry),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

pub fn get_instruction(raw: &u64) -> Result<&WorldInstruction, ProgramError> {
    WorldInstruction::try_from(raw)?;
    let ptr = raw as *const u64 as *const WorldInstruction;
    unsafe { Ok(&*ptr) }
}
