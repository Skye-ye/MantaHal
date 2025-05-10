use crate::common::addr::{PhysAddr, VirtAddr};

/// Task Context
///
/// Task Context is used to switch context between tasks
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct Context {
    /// Return Address
    ra: usize,
    /// Stack Pointer
    sp: usize,
    /// Static registers, r22 - r31 (r22 is s9, s0 - s8)
    regs: [usize; 10],
    /// Thread Pointer
    tp: usize,
    /// Page Table Root
    pt_root: usize,
    /// Floating Point Static Registers, f24 - f31(fs0 - fs7)
    #[cfg(feature = "fp")]
    fregs: [f64; 8],
}

impl Context {
    /// Create a new context
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Initialize the context with the given entry, kernel stack pointer and tls area
    pub fn init(&mut self, entry: usize, ksp: VirtAddr, tls_area: VirtAddr) {
        self.ra = entry;
        self.sp = ksp.into();
        self.tp = tls_area.into();
    }

    /// Set the page table root
    pub fn set_pagetable_root(&mut self, pt_root: PhysAddr) {
        self.pt_root = pt_root.into();
    }
}

/// Save the task context registers
macro_rules! save_general_regs {
    () => {
        "
            st.d      $ra, $a0,  0*8
            st.d      $sp, $a0,  1*8
            st.d      $s9, $a0,  2*8
            st.d      $s0, $a0,  3*8
            st.d      $s1, $a0,  4*8
            st.d      $s2, $a0,  5*8
            st.d      $s3, $a0,  6*8
            st.d      $s4, $a0,  7*8
            st.d      $s5, $a0,  8*8
            st.d      $s6, $a0,  9*8
            st.d      $s7, $a0, 10*8
            st.d      $s8, $a0, 11*8
            st.d      $tp, $a0, 12*8
        "
    };
}

/// Restore the task context registers
macro_rules! load_general_regs {
    () => {
        "
            ld.d      $ra, $a1,  0*8
            ld.d      $sp, $a1,  1*8
            ld.d      $s9, $a1,  2*8
            ld.d      $s0, $a1,  3*8
            ld.d      $s1, $a1,  4*8
            ld.d      $s2, $a1,  5*8
            ld.d      $s3, $a1,  6*8
            ld.d      $s4, $a1,  7*8
            ld.d      $s5, $a1,  8*8
            ld.d      $s6, $a1,  9*8
            ld.d      $s7, $a1, 10*8
            ld.d      $s8, $a1, 11*8
            ld.d      $tp, $a1, 12*8
        "
    };
}

#[cfg(feature = "fp")]
/// Save the floating point static registers
macro_rules! save_fp_regs {
    () => {
        "
            fst.d   $f24, $a0, 14*8 + 0*8
            fst.d   $f25, $a0, 14*8 + 1*8
            fst.d   $f26, $a0, 14*8 + 2*8
            fst.d   $f27, $a0, 14*8 + 3*8
            fst.d   $f28, $a0, 14*8 + 4*8
            fst.d   $f29, $a0, 14*8 + 5*8
            fst.d   $f30, $a0, 14*8 + 6*8
            fst.d   $f31, $a0, 14*8 + 7*8
        "
    };
}

#[cfg(feature = "fp")]
/// Load the floating point static registers
macro_rules! load_fp_regs {
    () => {
        "
            fld.d   $f24, $a1, 14*8 + 0*8
            fld.d   $f25, $a1, 14*8 + 1*8
            fld.d   $f26, $a1, 14*8 + 2*8
            fld.d   $f27, $a1, 14*8 + 3*8
            fld.d   $f28, $a1, 14*8 + 4*8
            fld.d   $f29, $a1, 14*8 + 5*8
            fld.d   $f30, $a1, 14*8 + 6*8
            fld.d   $f31, $a1, 14*8 + 7*8
        "
    };
}

/// Context Switch
///
/// Save the context of current task and switch to new task. If the new task is using a different page table,
/// the page table root will be updated.
#[naked]
extern "C" fn context_switch(from: *mut Context, to: *const Context) {
    //TODO: kernel task context switch
    unsafe {
        core::arch::naked_asm!(
            // Save Context
            save_general_regs!(),
            #[cfg(feature = "fp")]
            save_fp_regs!(),
            // Check if the new task is using a different page table
            // If so, update the page table root
            "
            ld.d $t0, $a0, 13*8
            ld.d $t1, $a1, 13*8
            beq $t0, $t1, 1f
            csrwr $t1, {pgdl}
            dbar 0
            invtlb 0x00, $r0, $r0
            1:
            ",
            // Restore Context
            load_general_regs!(),
            #[cfg(feature = "fp")]
            load_fp_regs!(),
            // Return to the caller
            "ret",
            pgdl = const crate::loongarch64::config::csr::PGDL,
        );
    }
}
