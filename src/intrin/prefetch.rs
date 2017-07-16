//! Memory prefetching intrinsics - currently wrapping LLVM's `prefetch` intrinsic.

#![cfg_attr(feature = "cargo-clippy", allow(inline_always))]

extern {
    #[inline(always)]
    #[link_name = "llvm.prefetch"]
    fn llvm_prefetch(address: *const u8, rw: i32, locality: i32, cache_type: i32);
}

// Using consts because somehow enum values are not treated as constant values è_é
const ICACHE  : i32 = 0;
const DCACHE  : i32 = 1;
const LOWEST  : i32 = 0;
const LOW     : i32 = 1;
const HIGH    : i32 = 2;
const HIGHEST : i32 = 3;
const RW      : i32 = 1;
const RD      : i32 = 0;

/// Prefetch a memory location, hinting that it is read-only data that very probably won't be used again later.
#[inline(always)]
pub fn prefetch_lowest<T>(addr: *const T) {
    unsafe { llvm_prefetch(addr as *const u8, RD, LOWEST, DCACHE); }
}
/// Prefetch a memory location, hinting that it is read-only data that probably won't be used again later.
#[inline(always)]
pub fn prefetch_low<T>(addr: *const T) {
    unsafe { llvm_prefetch(addr as *const u8, RD, LOW, DCACHE); }
}
/// Prefetch a memory location, hinting that it is read-only data that probably will be used again later.
#[inline(always)]
pub fn prefetch_high<T>(addr: *const T) {
    unsafe { llvm_prefetch(addr as *const u8, RD, HIGH, DCACHE); }
}
/// Prefetch a memory location, hinting that it is read-only data that very probably will be used again later.
#[inline(always)]
pub fn prefetch_highest<T>(addr: *const T) {
    unsafe { llvm_prefetch(addr as *const u8, RD, HIGHEST, DCACHE); }
}

/// Prefetch a memory location, hinting that it is read-only code that very probably won't be used again later.
#[inline(always)]
pub fn prefetch_insn_lowest<T>(addr: *const T) {
    unsafe { llvm_prefetch(addr as *const u8, RD, LOWEST, ICACHE); }
}
/// Prefetch a memory location, hinting that it is read-only code that probably won't be used again later.
#[inline(always)]
pub fn prefetch_insn_low<T>(addr: *const T) {
    unsafe { llvm_prefetch(addr as *const u8, RD, LOW, ICACHE); }
}
/// Prefetch a memory location, hinting that it is read-only code that probably will be used again later.
#[inline(always)]
pub fn prefetch_insn_high<T>(addr: *const T) {
    unsafe { llvm_prefetch(addr as *const u8, RD, HIGH, ICACHE); }
}
/// Prefetch a memory location, hinting that it is read-only code that very probably will be used again later.
#[inline(always)]
pub fn prefetch_insn_highest<T>(addr: *const T) {
    unsafe { llvm_prefetch(addr as *const u8, RD, HIGHEST, ICACHE); }
}


/// Prefetch a memory location, hinting that it is read-write data that very probably won't be used again later.
#[inline(always)]
pub fn prefetch_lowest_mut<T>(addr: *mut T) {
    unsafe { llvm_prefetch(addr as *const u8, RW, LOWEST, DCACHE); }
}
/// Prefetch a memory location, hinting that it is read-write data that probably won't be used again later.
#[inline(always)]
pub fn prefetch_low_mut<T>(addr: *mut T) {
    unsafe { llvm_prefetch(addr as *const u8, RW, LOW, DCACHE); }
}
/// Prefetch a memory location, hinting that it is read-write data that probably will be used again later.
#[inline(always)]
pub fn prefetch_high_mut<T>(addr: *mut T) {
    unsafe { llvm_prefetch(addr as *const u8, RW, HIGH, DCACHE); }
}
/// Prefetch a memory location, hinting that it is read-write data that very probably will be used again later.
#[inline(always)]
pub fn prefetch_highest_mut<T>(addr: *mut T) {
    unsafe { llvm_prefetch(addr as *const u8, RW, HIGHEST, DCACHE); }
}

/// Prefetch a memory location, hinting that it is read-write code that very probably won't be used again later.
#[inline(always)]
pub fn prefetch_insn_lowest_mut<T>(addr: *mut T) {
    unsafe { llvm_prefetch(addr as *const u8, RW, LOWEST, ICACHE); }
}
/// Prefetch a memory location, hinting that it is read-write code that probably won't be used again later.
#[inline(always)]
pub fn prefetch_insn_low_mut<T>(addr: *mut T) {
    unsafe { llvm_prefetch(addr as *const u8, RW, LOW, ICACHE); }
}
/// Prefetch a memory location, hinting that it is read-write code that probably will be used again later.
#[inline(always)]
pub fn prefetch_insn_high_mut<T>(addr: *mut T) {
    unsafe { llvm_prefetch(addr as *const u8, RW, HIGH, ICACHE); }
}
/// Prefetch a memory location, hinting that it is read-write code that very probably will be used again later.
#[inline(always)]
pub fn prefetch_insn_highest_mut<T>(addr: *mut T) {
    unsafe { llvm_prefetch(addr as *const u8, RW, HIGHEST, ICACHE); }
}

#[cfg(test)]
mod test {
    #[test]
    fn does_it_compile() {
        super::prefetch_high(0xdead as *const i32);
    }
}
