//! ## TLS layout for AArch64 and RISC-V
//!
//! ```text
//!             +-------------------------+
//!             |                         | \
//!             | Custom TCB format       |  |
//!             | (might be used          |   > Thread Control Block (TCB)
//!             |  by a libC)             |  |  (length: TCB_SIZE)
//!             |                         | /
//!    tls_ptr -+-------------------------+
//! (tp_offset) | GAP_ABOVE_TP            |
//!             +-------------------------+- static_tls_offset
//!             |                         | \
//!             | .tdata                  |  |
//!             |                         |  |
//!             + - - - - - - - - - - - - +   > Static TLS block
//!             |                         |  |  (length: static_tls_size)
//!             | .tbss                   |  |
//!             |                         | /
//!             +-------------------------+- (total length: tls_area_size)
//! ```
//!
//! Reference:
//! 1. <https://github.com/unikraft/unikraft/blob/staging/arch/x86/x86_64/tls.c>
//! 2. <https://github.com/unikraft/unikraft/blob/staging/arch/arm/arm64/tls.c>
extern crate alloc;

use memory_addr::align_up;

use core::alloc::Layout;
use core::ptr::NonNull;

const TLS_ALIGN: usize = 0x10;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "riscv64")] {
        const TCB_SIZE: usize = 0;
        const GAP_ABOVE_TP: usize = 0;
    } else if #[cfg(target_arch = "loongarch64")] {
        const TCB_SIZE: usize = 0;
        const GAP_ABOVE_TP: usize = 0;
    }
}

unsafe extern "C" {
    fn _stdata();
    fn _etdata();
    fn _etbss();
}

/// The memory region for thread-local storage.
pub struct TlsArea {
    base: NonNull<u8>,
    layout: Layout,
}

impl Drop for TlsArea {
    fn drop(&mut self) {
        unsafe {
            alloc::alloc::dealloc(self.base.as_ptr(), self.layout);
        }
    }
}

impl TlsArea {
    /// Returns the pointer to the TLS static area.
    ///
    /// One should set the hardware thread pointer register to this value.
    pub fn tls_ptr(&self) -> *mut u8 {
        unsafe { self.base.as_ptr().add(tp_offset()) }
    }

    /// Allocates the memory region for TLS, and initializes it.
    pub fn alloc() -> Self {
        let layout = Layout::from_size_align(tls_area_size(), TLS_ALIGN).unwrap();
        let area_base = unsafe { alloc::alloc::alloc_zeroed(layout) };

        let tls_load_base = _stdata as *mut u8;
        let tls_load_size = _etbss as usize - _stdata as usize;
        unsafe {
            // copy data from .tbdata section
            core::ptr::copy_nonoverlapping(
                tls_load_base,
                area_base.add(static_tls_offset()),
                tls_load_size,
            );
        }

        Self {
            base: NonNull::new(area_base).unwrap(),
            layout,
        }
    }
}

/// Returns the size of the static TLS block.
fn static_tls_size() -> usize {
    align_up(_etbss as usize - _stdata as usize, TLS_ALIGN)
}

/// Returns the offset of the static TLS block.
fn static_tls_offset() -> usize {
    if cfg!(any(target_arch = "riscv64", target_arch = "loongarch64")) {
        TCB_SIZE + GAP_ABOVE_TP
    } else {
        unreachable!()
    }
}

/// Returns the offset of the thread pointer.
fn tp_offset() -> usize {
    if cfg!(any(target_arch = "riscv64", target_arch = "loongarch64")) {
        TCB_SIZE
    } else {
        unreachable!()
    }
}

/// Returns the size of the TLS area.
fn tls_area_size() -> usize {
    if cfg!(any(target_arch = "riscv64", target_arch = "loongarch64")) {
        TCB_SIZE + GAP_ABOVE_TP + static_tls_size()
    } else {
        unreachable!()
    }
}
