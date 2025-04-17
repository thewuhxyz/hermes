use super::transmutable::{Transmutable, TransmutableMut};

#[repr(C)]
pub struct Entity {
    pub discriminator: u64,
    pub id: u64,
}

impl Entity {}

impl TransmutableMut for Entity {}

impl Transmutable for Entity {
    const LEN: usize = core::mem::size_of::<Entity>();
}
