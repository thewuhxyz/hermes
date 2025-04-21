use super::{
    into_bytes::IntoBytes,
    transmutable::{Transmutable, TransmutableMut},
};
use pinocchio::{
    memory::sol_memmove,
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
};
use std::collections::BTreeSet;

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

pub struct WorldMutate<'a> {
    pub world_metadata: &'a mut WorldMetadata,
    pub world_data: &'a mut [u8],
}

impl<'a> WorldMutate<'a> {
    pub fn from_bytes(bytes: &'a mut [u8]) -> Result<Self, ProgramError> {
        let world = unsafe {
            let (world_metadata_bytes, world_data) = bytes.split_at_mut(WorldMetadata::LEN);
            let world_metadata = WorldMetadata::load_mut_unchecked(world_metadata_bytes)?;

            Self {
                world_metadata,
                world_data,
            }
        };

        Ok(world)
    }

    pub fn init_new_world(account_data: &'a mut [u8]) -> Result<Self, ProgramError> {
        let (world_metadata_bytes, world_data) = account_data.split_at_mut(WorldMetadata::LEN);

        let world_metadata = unsafe { WorldMetadata::load_mut_unchecked(world_metadata_bytes)? };
        *world_metadata = WorldMetadata::default();

        let mut world = Self {
            world_data,
            world_metadata,
        };

        *world.is_permissionless()? = true;

        Ok(world)
    }

    pub fn add_new_authority(&mut self, authority: &Pubkey) -> Result<(), ProgramError> {
        let data_ptr = self.world_data as *mut _ as *mut u8;
        let offset = self.authority_size()?;

        unsafe {
            let new_authority_ptr = data_ptr.add(offset) as *mut Pubkey;
            let permissionles_ptr = new_authority_ptr.add(1) as *mut u8;
            sol_memmove(
                permissionles_ptr,
                new_authority_ptr as *mut u8,
                self.permissionless_len()? + self.systems_size()?,
            );
            *new_authority_ptr = *authority;
        };

        let authorities_len = self.authorities_len()?;

        *authorities_len += 1;

        Ok(())
    }

    pub fn remove_authority(&mut self, index: usize) -> Result<(), ProgramError> {
        let authorities_len = *self.authorities_len()? as usize;
        let remaining_athorities = authorities_len - index - 1;

        let size_to_move = self.permissionless_len()?
            + self.systems_size()?
            + authorities_size(remaining_athorities);

        unsafe {
            let authorities_ptr = self.authorities_mut()?.as_mut_ptr();
            let src_ptr = authorities_ptr.add(index + 1);
            let dst_ptr = authorities_ptr.add(index);
            sol_memmove(dst_ptr as *mut u8, src_ptr as *mut u8, size_to_move);
        };

        *self.authorities_len()? -= 1;

        Ok(())
    }

    pub fn authorities_len(&mut self) -> Result<&mut u32, ProgramError> {
        Ok(unsafe { &mut *(self.world_data.as_mut_ptr() as *mut u32) })
    }

    pub fn permissionless_len(&mut self) -> Result<usize, ProgramError> {
        Ok(core::mem::size_of::<bool>())
    }

    pub fn permissionless(&mut self) -> Result<&mut u8, ProgramError> {
        let byte = &mut self.world_data[self.authority_size()?];
        Ok(byte)
    }

    pub fn is_permissionless(&mut self) -> Result<&mut bool, ProgramError> {
        let byte = self.permissionless()?;
        match byte {
            0 | 1 => {
                let ptr = byte as *mut u8 as *mut bool;
                Ok(unsafe { &mut *ptr })
            }
            _ => Err(ProgramError::InvalidAccountData),
        }
    }

    pub fn authority_size(&mut self) -> Result<usize, ProgramError> {
        let authorities_len = *self.authorities_len()?;
        Ok(authorities_size(authorities_len as usize))
    }

    fn authorities_mut(&mut self) -> Result<&mut [Pubkey], ProgramError> {
        let authorities_len = *self.authorities_len()?;
        let authorities = unsafe {
            let authorities_ptr =
                self.world_data.as_mut_ptr().add(authorities_len as usize) as *mut _ as *mut Pubkey;
            core::slice::from_raw_parts_mut(authorities_ptr, authorities_len as usize)
        };
        Ok(authorities)
    }

    pub fn authorities(&mut self) -> Result<&[Pubkey], ProgramError> {
        let authorities_len = *self.authorities_len()?;
        let authorities = unsafe {
            let authorities_ptr = self.world_data.as_mut_ptr().add(authorities_len as usize)
                as *mut _ as *const Pubkey;
            core::slice::from_raw_parts(authorities_ptr, authorities_len as usize)
        };
        Ok(authorities)
    }

    pub fn systems_size(&mut self) -> Result<usize, ProgramError> {
        let systems_len = *self.systems_len()?;
        Ok(systems_size(systems_len as usize))
    }

    pub fn systems_len(&mut self) -> Result<&mut u32, ProgramError> {
        Ok(unsafe {
            let permissionless_ptr = self.is_permissionless()? as *mut bool;
            &mut *(permissionless_ptr.add(1) as *mut u32)
        })
    }

    pub fn systems_pubkey_slice(&mut self) -> Result<&mut [Pubkey], ProgramError> {
        let systems_len = self.systems_len()?;

        let systems_ptr = unsafe { (systems_len as *mut _ as *mut u8).add(4) as *mut Pubkey };

        Ok(unsafe {
            core::slice::from_raw_parts_mut(
                systems_ptr,
                *systems_len as usize / core::mem::size_of::<Pubkey>(),
            )
        })
    }

    pub fn systems_slice(&mut self) -> Result<&mut [u8], ProgramError> {
        let systems_len = self.systems_len()?;

        let offset = core::mem::size_of::<u32>();

        let systems_ptr = unsafe { (systems_len as *mut _ as *mut u8).add(offset) };

        Ok(unsafe { core::slice::from_raw_parts_mut(systems_ptr, *systems_len as usize) })
    }

    pub fn add_system(&mut self, system: &Pubkey) -> Result<usize, ProgramError> {
        let system_slice = self.systems_pubkey_slice()?;
        let original_len = system_slice.len();

        let mut vec = Vec::with_capacity(original_len + 1);
        vec.extend_from_slice(system_slice);

        let mut system_set: BTreeSet<Pubkey> = vec.into_iter().collect();
        system_set.insert(*system);

        let new_len = system_set.len();

        let mut size = 0;

        if new_len.saturating_sub(original_len) > 0 {
            let vec_slice: Vec<Pubkey> = system_set.into_iter().collect();

            let new_slice =
                unsafe { core::slice::from_raw_parts_mut(system_slice.as_mut_ptr(), new_len) };

            new_slice[0..].copy_from_slice(&vec_slice);

            let added_size = core::mem::size_of::<Pubkey>();

            let systems_len = self.systems_len()?;
            *systems_len += added_size as u32;

            size += added_size;
        }

        Ok(size)
    }

    pub fn remove_system(&mut self, index: usize) -> Result<(), ProgramError> {
        let authorities_len = self.systems_pubkey_slice()?.len();

        let remaining_systems = authorities_len - index - 1;

        let size_to_move = remaining_systems * core::mem::size_of::<Pubkey>();

        if remaining_systems > 0 {
            unsafe {
                let authorities_ptr = self.systems_pubkey_slice()?.as_mut_ptr();
                let src_ptr = authorities_ptr.add(index + 1);
                let dst_ptr = authorities_ptr.add(index);
                sol_memmove(dst_ptr as *mut u8, src_ptr as *mut u8, size_to_move);
            };
        }

        *self.authorities_len()? -= 1;

        Ok(())
    }

    pub fn size(&mut self) -> Result<usize, ProgramError> {
        Ok(24 + self.authority_size()? + self.permissionless_len()? + self.systems_size()?)
    }
}

pub fn systems_size(count: usize) -> usize {
    core::mem::size_of::<u32>() + count
}

pub fn authorities_size(count: usize) -> usize {
    core::mem::size_of::<u32>() + (count * core::mem::size_of::<Pubkey>())
}
