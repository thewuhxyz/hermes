use core::mem::MaybeUninit;

use pinocchio::syscalls::sol_memcmp_;

pub fn sol_assert_bytes_eq(left: &[u8], right: &[u8]) -> bool {
    unsafe {
        let mut result = MaybeUninit::<i32>::uninit();
        sol_memcmp_(
            left.as_ptr(),
            right.as_ptr(),
            left.len() as u64,
            result.as_mut_ptr(),
        );
        result.assume_init() == 0
    }
}
