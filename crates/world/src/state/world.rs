use pinocchio::{
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
};

use super::{
    into_bytes::IntoBytes,
    transmutable::{Transmutable, TransmutableMut},
};

pub const WORLD_DISCRIMINATOR: u64 = 0;

pub const NEW_WORLD_SIZE: usize = 8 + 8 + 8 + 4 + 1 + 4;

pub fn world_seed() -> &'static [u8] {
    b"world"
}

pub fn world_size() -> usize {
    16 + 8 + 1 + 8
}

pub fn world_pda(id: &u64) -> (Pubkey, u8) {
    find_program_address(&[world_seed(), &id.to_be_bytes()], &crate::ID)
}

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

impl IntoBytes for WorldMetadata {
    fn into_bytes(&self) -> Result<&[u8], ProgramError> {
        let bytes =
            unsafe { core::slice::from_raw_parts(self as *const Self as *const u8, Self::LEN) };
        Ok(bytes)
    }
}

impl Default for WorldMetadata {
    fn default() -> Self {
        Self {
            discriminator: WORLD_DISCRIMINATOR,
            entities: 0,
            id: 0,
        }
    }
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

    pub fn init_new_world(account_data: &'a mut [u8]) -> Result<(), ProgramError> {
        let (world_metadata_bytes, authorities_bytes) =
            account_data.split_at_mut(WorldMetadata::LEN);
        world_metadata_bytes.copy_from_slice(WorldMetadata::default().into_bytes()?);

        let permissionless_offset = core::mem::size_of::<u32>();

        authorities_bytes[permissionless_offset] = 1; // set permissionless: true

        Ok(())
    }
}
