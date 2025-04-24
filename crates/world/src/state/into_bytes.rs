use pinocchio::program_error::ProgramError;

pub trait IntoBytes {
    fn into_bytes(&self) -> Result<&[u8], ProgramError>;
}
