
use crate::arch::interrupt::IRQVector;
use crate::arch::trapframe::TrapFrame;

#[derive(Debug, Clone, Copy)]
pub enum TrapType {
    Breakpoint,
    SysCall,
    Timer,
    Unknown,
    SupervisorExternal,
    StorePageFault(usize),
    LoadPageFault(usize),
    InstructionPageFault(usize),
    IllegalInstruction(usize),
    Irq(IRQVector),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeReason {
    NoReason,
    IRQ,
    Timer,
    SysCall,
}

// TODO: Implement Into EscapeReason
impl Into<EscapeReason> for TrapType {
    fn into(self) -> EscapeReason {
        match self {
            TrapType::SysCall => EscapeReason::SysCall,
            _ => EscapeReason::NoReason,
        }
    }
}

pub fn specific_handler(tf: &mut TrapFrame, trap_type: TrapType, token: usize) {
    unimplemented!()
}