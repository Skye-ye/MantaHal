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
}

/// Saved registers when a trap (interrupt or exception) occurs.
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct TrapFrame {
    pub gr: GeneralRegisters, // general purpose registers
    pub era: usize,           // exception return address
    pub prmd: usize,          // pre-exception mode information
    #[cfg(feature = "fp")]
    pub fr: FloatingPointRegisters, // floating point registers
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
