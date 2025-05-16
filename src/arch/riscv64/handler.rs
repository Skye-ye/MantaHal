
use crate::arch::irq::IRQVector;
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


pub fn specific_handler(tf: &mut TrapFrame, trap_type: TrapType, token: usize) {
    todo!()
}