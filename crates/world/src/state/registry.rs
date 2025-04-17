use pinocchio::instruction::Seed;

use super::transmutable::{Transmutable, TransmutableMut};

#[repr(C)]
pub struct Registry {
    pub discriminator: u64,
    pub worlds: u64,
}

impl Registry {
    pub fn seeds() -> &'static [u8] {
        b"registry".as_ref()
    }
}

pub fn registry_signer(bump: &[u8; 1]) -> [Seed; 2] {
    [Registry::seeds().as_ref().into(), bump.as_ref().into()]
}

impl TransmutableMut for Registry {}

impl Transmutable for Registry {
    const LEN: usize = core::mem::size_of::<Registry>();
}
