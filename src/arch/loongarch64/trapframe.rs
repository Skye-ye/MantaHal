use core::arch::asm;
/// General registers of Loongarch64.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]

pub struct GeneralRegisters {
    pub zero: usize,
    pub ra: usize,
    pub tp: usize,
    pub sp: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub t7: usize,
    pub t8: usize,
    pub u0: usize,
    pub fp: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
}

#[cfg(feature = "fp")]
/// Floating point registers of Loongarch64.
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct FloatingPointRegisters {
    pub fa0: f64,
    pub fa1: f64,
    pub fa2: f64,
    pub fa3: f64,
    pub fa4: f64,
    pub fa5: f64,
    pub fa6: f64,
    pub fa7: f64,
    pub ft0: f64,
    pub ft1: f64,
    pub ft2: f64,
    pub ft3: f64,
    pub ft4: f64,
    pub ft5: f64,
    pub ft6: f64,
    pub ft7: f64,
    pub ft8: f64,
    pub ft9: f64,
    pub ft10: f64,
    pub ft11: f64,
    pub ft12: f64,
    pub ft13: f64,
    pub ft14: f64,
    pub ft15: f64,
    pub fs0: f64,
    pub fs1: f64,
    pub fs2: f64,
    pub fs3: f64,
    pub fs4: f64,
    pub fs5: f64,
    pub fs6: f64,
    pub fs7: f64,
    pub fcsr: u32, // floating point control and status register
    pub f_need_save: u8,  // for lazy save
    pub f_need_restore: u8,
    pub f_signal_dirty: u8
}

/// Saved registers when a trap (interrupt or exception) occurs.
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct TrapFrame {
    pub gr: GeneralRegisters, // general purpose registers
    pub era: usize,           // exception return address
    pub prmd: usize,          // pre-exception mode information
    // #[cfg(feature = "fp")]
    // pub fr: FloatingPointRegisters, // floating point registers
}

impl TrapFrame {
    #[inline]
    pub fn new() -> Self {
        Self {
            prmd: (0b0111),
            ..Default::default()
        }
    }

    /// If the syscall is successful, the return address is incremented by 4.
    pub fn syscall_ok(&mut self) {
        self.era += 4;
    }

    #[inline]
    pub const fn arg0(&self) -> usize {
        self.gr.a0
    }

    #[inline]
    pub const fn arg1(&self) -> usize {
        self.gr.a1
    }

    #[inline]
    pub const fn arg2(&self) -> usize {
        self.gr.a2
    }

    #[inline]
    pub const fn arg3(&self) -> usize {
        self.gr.a3
    }

    #[inline]
    pub const fn arg4(&self) -> usize {
        self.gr.a4
    }

    #[inline]
    pub const fn arg5(&self) -> usize {
        self.gr.a5
    }

    #[inline]
    pub const fn arg6(&self) -> usize {
        self.gr.a6
    }

    #[inline]
    pub const fn arg7(&self) -> usize {
        self.gr.a7
    }

}


impl FloatingPointRegisters {
    // implementation of lazy save for floating point registers
    pub fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }

    pub fn mark_save_if_needed(&mut self, need_save: u8) {
        self.f_need_save |= need_save;
        self.f_signal_dirty |= need_save;
    }
    pub fn yield_task(&mut self) {
        self.save_fr();
        self.f_need_restore = 1;
    }

    pub fn encounter_signal(&mut self) {
        self.save_fr();
    }

    pub fn save_fr(&mut self) {
        if self.f_need_save == 0 {
            return;
        }
        self.f_need_save = 0;

        unsafe {
            let mut _t: usize = 1; // alloc a register but not zero.
            asm!("
            fsd  f0,  0*8({0})
            fsd  f1,  1*8({0})
            fsd  f2,  2*8({0})
            fsd  f3,  3*8({0})
            fsd  f4,  4*8({0})
            fsd  f5,  5*8({0})
            fsd  f6,  6*8({0})
            fsd  f7,  7*8({0})
            fsd  f8,  8*8({0})
            fsd  f9,  9*8({0})
            fsd f10, 10*8({0})
            fsd f11, 11*8({0})
            fsd f12, 12*8({0})
            fsd f13, 13*8({0})
            fsd f14, 14*8({0})
            fsd f15, 15*8({0})
            fsd f16, 16*8({0})
            fsd f17, 17*8({0})
            fsd f18, 18*8({0})
            fsd f19, 19*8({0})
            fsd f20, 20*8({0})
            fsd f21, 21*8({0})
            fsd f22, 22*8({0})
            fsd f23, 23*8({0})
            fsd f24, 24*8({0})
            fsd f25, 25*8({0})
            fsd f26, 26*8({0})
            fsd f27, 27*8({0})
            fsd f28, 28*8({0})
            fsd f29, 29*8({0})
            fsd f30, 30*8({0})
            fsd f31, 31*8({0})
            csrr {1}, fcsr
            sw  {1}, 32*8({0})
        ", in(reg) &self,
                inout(reg) _t
            );
        };
    }
    pub fn restore(&mut self) {
        if self.f_need_restore == 0 {
            return;
        }
        self.f_need_restore = 0;
        unsafe {
            asm!("
            fld  f0,  0*8({0})
            fld  f1,  1*8({0})
            fld  f2,  2*8({0})
            fld  f3,  3*8({0})
            fld  f4,  4*8({0})
            fld  f5,  5*8({0})
            fld  f6,  6*8({0})
            fld  f7,  7*8({0})
            fld  f8,  8*8({0})
            fld  f9,  9*8({0})
            fld f10, 10*8({0})
            fld f11, 11*8({0})
            fld f12, 12*8({0})
            fld f13, 13*8({0})
            fld f14, 14*8({0})
            fld f15, 15*8({0})
            fld f16, 16*8({0})
            fld f17, 17*8({0})
            fld f18, 18*8({0})
            fld f19, 19*8({0})
            fld f20, 20*8({0})
            fld f21, 21*8({0})
            fld f22, 22*8({0})
            fld f23, 23*8({0})
            fld f24, 24*8({0})
            fld f25, 25*8({0})
            fld f26, 26*8({0})
            fld f27, 27*8({0})
            fld f28, 28*8({0})
            fld f29, 29*8({0})
            fld f30, 30*8({0})
            fld f31, 31*8({0})
            lw  {0}, 32*8({0})
            csrw fcsr, {0}
        ", in(reg) &self
            );
        }
    }
}