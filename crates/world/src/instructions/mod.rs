mod add_authority;
pub use add_authority::*;

mod add_entity;
pub use add_entity::*;

mod approve_system;
pub use approve_system::*;

mod initialize_registry;
pub use initialize_registry::*;

mod initialize_new_world;
pub use initialize_new_world::*;

mod remove_authority;
pub use remove_authority::*;

mod remove_system;
pub use remove_system::*;



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
            1001 => Ok(WorldInstruction::InitializeNewWorld),
            1002 => Ok(WorldInstruction::AddAuthority),
            1003 => Ok(WorldInstruction::RemoveAuthority),
            1004 => Ok(WorldInstruction::ApproveSystem),
            1005 => Ok(WorldInstruction::RemoveSystem),
            1006 => Ok(WorldInstruction::AddEntity),
            1007 => Ok(WorldInstruction::InitilizeComponent),
            1008 => Ok(WorldInstruction::Apply),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

pub fn get_instruction(raw: &u64) -> Result<&WorldInstruction, ProgramError> {
    WorldInstruction::try_from(raw)?;
    let ptr = raw as *const u64 as *const WorldInstruction;
    unsafe { Ok(&*ptr) }
}
