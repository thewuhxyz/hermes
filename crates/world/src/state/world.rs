use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

use super::transmutable::{Transmutable, TransmutableMut};

#[repr(C)]
pub struct WorldMetadata {
    pub discriminator: u64,
    pub id: u64,
    pub entities: u64,
}

impl WorldMetadata {}

impl TransmutableMut for WorldMetadata {}

impl Transmutable for WorldMetadata {
    const LEN: usize = core::mem::size_of::<WorldMetadata>();
}

pub struct World<'a> {
    pub world_metadata: &'a WorldMetadata,
    pub authorities_len: u32,
    pub authorities: &'a [Pubkey],
    pub permissionless: &'a bool,
    pub systems_len: u32,
    pub systems: &'a [u8],
}

impl<'a> World<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ProgramError> {
        let world = unsafe {
            let world_metadata = WorldMetadata::load_unchecked(&bytes[..WorldMetadata::LEN])?;

            let authorities_len_ptr =
                &bytes.as_ptr().add(WorldMetadata::LEN) as *const _ as *const u32;

            //  aligned up to this point, shouldn't error
            let authorities_len = authorities_len_ptr.read();
            let authorities_ptr = authorities_len_ptr.add(1) as *const _ as *const Pubkey;

            let permissionless_ptr =
                authorities_ptr.add(authorities_len as usize) as *const _ as *const bool;

            let systems_len_ptr = permissionless_ptr.add(1) as *const _ as *const u32;
            let systems_len = systems_len_ptr.read_unaligned();

            let systems_ptr = systems_len_ptr.add(1) as *const _ as *const u8;

            Self {
                world_metadata,
                authorities: core::slice::from_raw_parts(authorities_ptr, authorities_len as usize),
                authorities_len,
                permissionless: &*permissionless_ptr,
                systems_len,
                systems: core::slice::from_raw_parts(systems_ptr, systems_len as usize),
            }
        };

        Ok(world)
    }
}

pub struct WorldMut<'a> {
    pub world_metadata: &'a mut WorldMetadata,
    pub authorities_len: u32,
    pub authorities: &'a mut [Pubkey],
    pub permissionless: &'a mut bool,
    pub systems_len: u32,
    pub systems: &'a mut [u8],
}

impl<'a> WorldMut<'a> {
    pub fn from_bytes(bytes: &'a mut [u8]) -> Result<Self, ProgramError> {
        let world = unsafe {
            let (world_metadata_bytes, rest) = bytes.split_at_mut(WorldMetadata::LEN);
            let world_metadata = WorldMetadata::load_mut_unchecked(world_metadata_bytes)?;

            let authorities_len_ptr =
                &rest.as_mut_ptr().add(WorldMetadata::LEN) as *const _ as *const u32;

            //  aligned up to this point, shouldn't error
            let authorities_len = authorities_len_ptr.read();
            let authorities_ptr = authorities_len_ptr.add(1) as *const _ as *mut Pubkey;

            let permissionless_ptr =
                authorities_ptr.add(authorities_len as usize) as *const _ as *mut bool;

            let systems_len_ptr = permissionless_ptr.add(1) as *const _ as *mut u32;
            let systems_len = systems_len_ptr.read_unaligned();

            let systems_ptr = systems_len_ptr.add(1) as *const _ as *mut u8;

            Self {
                world_metadata,
                authorities: core::slice::from_raw_parts_mut(
                    authorities_ptr,
                    authorities_len as usize,
                ),
                authorities_len,
                permissionless: &mut *permissionless_ptr,
                systems_len,
                systems: core::slice::from_raw_parts_mut(systems_ptr, systems_len as usize),
            }
        };

        Ok(world)
    }
}
