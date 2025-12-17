use std::mem;

pub static LOOKUP: &[LookupParams] = unsafe {
    let bytes = &LOOKUP_RAW.0;
    std::slice::from_raw_parts(
        bytes.as_ptr() as *const LookupParams,
        bytes.len() / mem::size_of::<LookupParams>()
    )
};

pub static ATTACK_TABLE: &[u64] = unsafe {
    let bytes = &ATTACKS_RAW.0;
    std::slice::from_raw_parts(
        bytes.as_ptr() as *const u64,
        bytes.len() / mem::size_of::<u64>()
    )
};

#[repr(C, align(8))]
struct AlignedData<T: ?Sized>(T);

#[cfg(target_feature = "bmi2")]
static LOOKUP_RAW: &AlignedData<[u8]> = &AlignedData(*include_bytes!("slider_tables/pext_lookups.table"));

#[cfg(target_feature = "bmi2")]
static ATTACKS_RAW: &AlignedData<[u8]> = &AlignedData(*include_bytes!("slider_tables/pext_attacks.table"));

#[cfg(not(target_feature = "bmi2"))]
static LOOKUP_RAW: &AlignedData<[u8]> = &AlignedData(*include_bytes!("slider_tables/plain_lookups.table"));

#[cfg(not(target_feature = "bmi2"))]
static ATTACKS_RAW: &AlignedData<[u8]> = &AlignedData(*include_bytes!("slider_tables/plain_attacks.table"));

#[cfg(target_feature = "bmi2")]
#[repr(C)]
pub struct LookupParams { mask: u64, pub offset: u32 }

#[cfg(not(target_feature = "bmi2"))]
#[repr(C)]
pub struct LookupParams { magic: u64, mask: u64, pub offset: u32, shift: u8 }

#[inline(always)]
#[cfg(target_feature = "bmi2")]
pub fn calculate_index(occupancy: u64, params: &LookupParams) -> usize {
    unsafe { core::arch::x86_64::_pext_u64(occupancy, params.mask) as usize }
}

#[inline(always)]
#[cfg(not(target_feature = "bmi2"))]
pub const fn calculate_index(occupancy: u64, params: &LookupParams) -> usize {
    (occupancy | !params.mask).wrapping_mul(params.magic).wrapping_shr(params.shift as u32) as usize
}