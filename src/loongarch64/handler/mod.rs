// mod.rs
pub mod exc_addr;
pub mod exc_debug;
pub mod exc_instr;
pub mod exc_page;     // page fault
pub mod exc_syscall;
pub mod exc_tlb;
pub mod irq_time;

use crate::arch::interrupt::IRQVector;
use crate::arch::trapframe::TrapFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeReason {
    NoReason,    // unexpected error?
    IRQ,
    Timer,
    SysCall,
}


/// transfrom trap type to handler-suitable type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapType {
    NoReason = 0,
    IRQ = 1,
    Time = 2,
    SysCall = 3,
    DeBug = 4,          // to avoid keyword conflict
    PageFault = 5,      // 0-6
    AddressError = 6,   // 0-3
    InstrError = 7,     // 0-2
    TLBRefill = 8      // 0
}

impl Into<EscapeReason> for TrapType {
    fn into(self) -> EscapeReason {
        match self {
            TrapType::SysCall => EscapeReason::SysCall,
            TrapType::Time => EscapeReason::Timer,
            TrapType::IRQ => EscapeReason::IRQ,
            _ => EscapeReason::NoReason,
        }
    }
}

pub type HandlerFn = fn(&mut TrapFrame, usize);

/// handler array
static HANDLERS: [HandlerFn; 9] = [
    |_, _| {},                // NoReason
    |_, _| {},                // IRQ
    irq_time::handler,        // Time
    exc_syscall::handler,     // SysCall
    exc_debug::handler,       // DeBug
    exc_page::handler,        // PageFault
    exc_addr::handler,        // AddressError
    exc_instr::handler,       // InstrError
    exc_tlb::handler,         // TLBRefill
];

/// tf: context
/// handle_type: which handler to call
/// token: specific token for the handler
pub fn specify_handler(tf: &mut TrapFrame, trap_type: TrapType, token: usize) {
    let handler = HANDLERS[trap_type as usize];
    handler(tf, token);
}