use core::arch::global_asm;
use core::ops::{Index, IndexMut};

/// User to kernel trap frame
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct TrapFrame {
    //user to kernel
    pub gr: [usize; 32],  //general purpose registers 0-31
    pub era: usize,       //return address 32
    pub prmd: usize,      //user status 33
    pub fg: FloatContext, //float registers
}

/// Kernel trap frame
/// only save callee saved registers
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct KernelTrapFrame {
    pub kr: [usize; 9],   // callee saved register 0-8
    pub sp: usize,        // stack pointer 9
    pub ra: usize,        // return address 10
    pub fp: usize,        // frame ptr 11
    pub tp: usize,        // thread pointer (hart id) 12
    pub fg: FloatContext, // float registers
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct FloatContext {
    pub user_fx: [f64; 32],
    pub fcsr: u32, // floating point control and status register
}

impl TrapFrame {
    #[inline]
    pub fn new() -> Self {
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
            self.gr[4], self.gr[5], self.gr[6], self.gr[7], self.gr[8], self.gr[9],
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

global_asm! {
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
