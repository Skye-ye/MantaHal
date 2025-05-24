// mod.rs
//! This crate is designed for loongarch-specific traptype
pub mod exc_addr;
pub mod exc_debug;
pub mod exc_instr;
pub mod exc_page; // page fault
pub mod exc_syscall;
pub mod exc_tlb;
pub mod irq_time;

pub use super::trapframe::TrapFrame;

/// transfrom trap type to handler-suitable type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapType {
    NoReason = 0,
    Irq = 1,
    Time = 2,
    SysCall = 3,
    Breakpoint = 4,        // to avoid keyword conflict
    PageFault = 5,    // 0-6
    AddressError = 6, // 0-3
    InstrError = 7,   // 0-2
    TLBRefill = 8,    // 0
}

pub type HandlerFn = fn(&mut TrapFrame, usize);

/// handler array
static HANDLERS: [HandlerFn; 9] = [
    |_, _| {},            // NoReason
    |_, _| {},            // IRQ
    irq_time::handler,    // Time
    exc_syscall::handler, // SysCall
    exc_debug::handler,   // Breakpoint
    exc_page::handler,    // PageFault
    exc_addr::handler,    // AddressError
    exc_instr::handler,   // InstrError
    exc_tlb::handler,     // TLBRefill
];

/// tf: context
/// handle_type: which handler to call
/// token: specific token for the handler
pub fn specify_handler(tf: &mut TrapFrame, handle_type: TrapType, token: usize) {
    let handler = HANDLERS[handle_type as usize];
    handler(tf, token);
}
