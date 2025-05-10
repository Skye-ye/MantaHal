use core::arch::{global_asm, asm};
use core::ops::{Index, IndexMut};


/// trap handle process
///  | hardware : save crmd in prmd \ change privilege to 0 (highest level) \ disable irq \ save pc in era
///  | goto traphandler according to eentry
///  | retrurn

#[allow(missing_docs)]
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct TrapFrame {
    //user to kernel
    pub gr : [usize; 32],  //general purpose registers 0-31
    pub era : usize,       //return address 32
    pub prmd : usize,      //user status 33
    pub fg : FloatContext, //float registers
}

/// kernel trap
/// only save callee saved registers
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct KernelTrapFrame {
    pub kr : [usize; 9],  // callee saved register 0-8
    pub sp : usize,  // stack pointer 9
    pub ra : usize,  // return address 10
    pub fp : usize,  // frame ptr 11
    pub tp : usize,  // thread pointer (hart id) 12
    pub fg : FloatContext, // float registers
}


#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct FloatContext {
    pub user_fx: [f64; 32],
    pub fcsr : u32, // floating point control and status register
    pub need_save: u8,
    pub need_restore: u8,
    pub signal_dirty: u8,
}

impl FloatContext {
    pub fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }

    pub fn mark_save_if_needed(&mut self, need_save: u8) {
        self.need_save |= need_save;
        self.signal_dirty |= need_save;
    }

    pub fn yield_task(&mut self) {
        self.save();
        self.need_restore = 1;
    }
 
    pub fn encounter_signal(&mut self) {
        self.save();
    }

    /// Save reg -> mem
    pub fn save(&mut self) {
        if self.need_save == 0 {
            return;
        }
        self.need_save = 0;
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
        ", in(reg) self,
                inout(reg) _t
            );
        };
    }

    /// Restore mem -> reg
    pub fn restore(&mut self) {
        if self.need_restore == 0 {
            return;
        }
        self.need_restore = 0;
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
        ", in(reg) self
            );
        }
    }

}

impl TrapFrame {
    #[inline]
    pub fn new() -> Self{
        Self {
            prmd: (0b0111),
            ..Default::default()
        }
    }

    pub fn syscall_ok(&mut self) {
        self.era += 4;
    }

    #[inline]
    pub fn args(&self) -> [usize; 6] {
        [
            self.gr[4],
            self.gr[5],
            self.gr[6],
            self.gr[7],
            self.gr[8],
            self.gr[9],
        ]
    }

}

pub enum TrapFrameArgs {
    SEPC,
    RA,
    SP,
    RET,
    ARG0,
    ARG1,
    ARG2,
    TLS,
    SYSCALL,
}

impl Index<TrapFrameArgs> for TrapFrame {
    type Output = usize;

    fn index(&self, index: TrapFrameArgs) -> &Self::Output {
        match index {
            TrapFrameArgs::SEPC => &self.era,
            TrapFrameArgs::RA => &self.gr[1],
            TrapFrameArgs::SP => &self.gr[3],
            TrapFrameArgs::RET => &self.gr[4],
            TrapFrameArgs::ARG0 => &self.gr[4],
            TrapFrameArgs::ARG1 => &self.gr[5],
            TrapFrameArgs::ARG2 => &self.gr[6],
            TrapFrameArgs::TLS => &self.gr[2],
            TrapFrameArgs::SYSCALL => &self.gr[11],
        }
    }
}

impl IndexMut<TrapFrameArgs> for TrapFrame {
    fn index_mut(&mut self, index: TrapFrameArgs) -> &mut Self::Output {
        match index {
            TrapFrameArgs::SEPC => &mut self.era,
            TrapFrameArgs::RA => &mut self.gr[1],
            TrapFrameArgs::SP => &mut self.gr[3],
            TrapFrameArgs::RET => &mut self.gr[4],
            TrapFrameArgs::ARG0 => &mut self.gr[4],
            TrapFrameArgs::ARG1 => &mut self.gr[5],
            TrapFrameArgs::ARG2 => &mut self.gr[6],
            TrapFrameArgs::TLS => &mut self.gr[2],
            TrapFrameArgs::SYSCALL => &mut self.gr[11],
        }
    }
}

global_asm!{
    r"
        .altmacro

        .macro SAVE_FLOAT_REGS base_reg, offset
            fst.d $f0,  \base_reg, \offset + 0*8
            fst.d $f1,  \base_reg, \offset + 1*8
            fst.d $f2,  \base_reg, \offset + 2*8
            fst.d $f3,  \base_reg, \offset + 3*8
            fst.d $f4,  \base_reg, \offset + 4*8
            fst.d $f5,  \base_reg, \offset + 5*8
            fst.d $f6,  \base_reg, \offset + 6*8
            fst.d $f7,  \base_reg, \offset + 7*8
            fst.d $f8,  \base_reg, \offset + 8*8
            fst.d $f9,  \base_reg, \offset + 9*8
            fst.d $f10, \base_reg, \offset + 10*8
            fst.d $f11, \base_reg, \offset + 11*8
            fst.d $f12, \base_reg, \offset + 12*8
            fst.d $f13, \base_reg, \offset + 13*8
            fst.d $f14, \base_reg, \offset + 14*8
            fst.d $f15, \base_reg, \offset + 15*8
            fst.d $f16, \base_reg, \offset + 16*8
            fst.d $f17, \base_reg, \offset + 17*8
            fst.d $f18, \base_reg, \offset + 18*8
            fst.d $f19, \base_reg, \offset + 19*8
            fst.d $f20, \base_reg, \offset + 20*8
            fst.d $f21, \base_reg, \offset + 21*8
            fst.d $f22, \base_reg, \offset + 22*8
            fst.d $f23, \base_reg, \offset + 23*8
            fst.d $f24, \base_reg, \offset + 24*8
            fst.d $f25, \base_reg, \offset + 25*8
            fst.d $f26, \base_reg, \offset + 26*8
            fst.d $f27, \base_reg, \offset + 27*8
            fst.d $f28, \base_reg, \offset + 28*8
            fst.d $f29, \base_reg, \offset + 29*8
            fst.d $f30, \base_reg, \offset + 30*8
            fst.d $f31, \base_reg, \offset + 31*8

            movfcsr2gr  $t0, $fcsr
            st.d $t0, \base_reg, \offset + 32*8
        .endm
        
        .macro LOAD_FLOAT_REGS base_reg, offset
            fld.d $f0,  \base_reg, \offset + 0*8
            fld.d $f1,  \base_reg, \offset + 1*8
            fld.d $f2,  \base_reg, \offset + 2*8
            fld.d $f3,  \base_reg, \offset + 3*8
            fld.d $f4,  \base_reg, \offset + 4*8
            fld.d $f5,  \base_reg, \offset + 5*8
            fld.d $f6,  \base_reg, \offset + 6*8
            fld.d $f7,  \base_reg, \offset + 7*8
            fld.d $f8,  \base_reg, \offset + 8*8
            fld.d $f9,  \base_reg, \offset + 9*8
            fld.d $f10, \base_reg, \offset + 10*8
            fld.d $f11, \base_reg, \offset + 11*8
            fld.d $f12, \base_reg, \offset + 12*8
            fld.d $f13, \base_reg, \offset + 13*8
            fld.d $f14, \base_reg, \offset + 14*8
            fld.d $f15, \base_reg, \offset + 15*8
            fld.d $f16, \base_reg, \offset + 16*8
            fld.d $f17, \base_reg, \offset + 17*8
            fld.d $f18, \base_reg, \offset + 18*8
            fld.d $f19, \base_reg, \offset + 19*8
            fld.d $f20, \base_reg, \offset + 20*8
            fld.d $f21, \base_reg, \offset + 21*8
            fld.d $f22, \base_reg, \offset + 22*8
            fld.d $f23, \base_reg, \offset + 23*8
            fld.d $f24, \base_reg, \offset + 24*8
            fld.d $f25, \base_reg, \offset + 25*8
            fld.d $f26, \base_reg, \offset + 26*8
            fld.d $f27, \base_reg, \offset + 27*8
            fld.d $f28, \base_reg, \offset + 28*8
            fld.d $f29, \base_reg, \offset + 29*8
            fld.d $f30, \base_reg, \offset + 30*8
            fld.d $f31, \base_reg, \offset + 31*8
            
            ld.d $t0, \base_reg, \offset + 32*8
            movgr2fcsr $fcsr, $t0
        .endm
    "
}