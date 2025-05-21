use core::arch::global_asm;

use riscv::{interrupt::{supervisor, Exception, Trap}, register::{scause, sepc, sstatus, stval}};

use super::{handler::{self, TrapType}, time::set_next_timer_irq, trapframe::{self, TrapFrame}};

use super::irq::Irq;


global_asm!(include_str!("trap.asm"));

unsafe extern "C" {
    fn __trap_from_user();
    fn __trap_from_kernel();
    fn __return_to_user(cx: *mut TrapFrame);
}

pub fn init() {
    unsafe { set_kernel_trap() };
}

pub unsafe fn set_kernel_trap() {
    unsafe { Irq::set_trap_handler(__trap_from_kernel as usize) };
}


pub fn run_user_task(context: &mut trapframe::TrapFrame) -> TrapType {
    unsafe {
        __return_to_user(context);
    }
    // user trap arrive here
    trap_handler(context)
}

pub fn trap_handler(cx: &mut TrapFrame) -> TrapType {
    let scause = scause::read();
    let stval = stval::read();
    let sepc = sepc::read();
    let cause = scause.cause();

    // unsafe { enable_interrupt() };

    let trap_type = match cause.try_into() {
        Ok(Trap::Exception(e)) => match e {
            Exception::Breakpoint => {
                cx.sepc += 2;
                TrapType::Breakpoint
            }
            Exception::UserEnvCall => TrapType::SysCall,
            Exception::StorePageFault => TrapType::StorePageFault(stval),
            Exception::InstructionPageFault => TrapType::InstructionPageFault(stval),
            Exception::LoadPageFault => TrapType::LoadPageFault(stval),
            Exception::IllegalInstruction => {
                TrapType::IllegalInstruction(stval)
            }
            e => {
                log::warn!("Unknown user exception: {:?}", e);
                TrapType::Unknown
            }
        },

        Ok(Trap::Interrupt(i)) => {
            match i {
                supervisor::Interrupt::SupervisorTimer => {
                    // NOTE: User may trap into kernel frequently. As a consequence, this timer are
                    // likely not triggered in user mode but rather be triggered in supervisor mode,
                    // which will cause user program running on the cpu for a quite long time.
                    TrapType::Timer
                }
                supervisor::Interrupt::SupervisorExternal => TrapType::SupervisorExternal,
                _ => {
                    panic!(
                        "[trap_handler] Unsupported trap {cause:?}, stval = {stval:#x}!, sepc = {sepc:#x}"
                    );
                }
            }
        }
        Err(_) => {
            panic!(
                "[trap_handler] Error when converting trap to target-specific trap cause {:?}",
                cause
            );
        }
    };
    handler::specific_handler(cx, trap_type, 0);
    trap_type
}

pub fn panic_on_unknown_trap() {
    panic!(
        "[kernel] sstatus sum {}, {:?}(scause:{}) in application, bad addr = {:#x}, bad instruction = {:#x}, kernel panicked!!",
        sstatus::read().sum(),
        scause::read().cause(),
        scause::read().bits(),
        stval::read(),
        sepc::read(),
    );
}

pub fn kernel_trap_handler() -> TrapType {
    let stval = stval::read();
    let scause = scause::read();
    let sepc = sepc::read();
    let trap = scause.cause();
    match trap.try_into() {
        Ok(Trap::Interrupt(i)) => match i {
            supervisor::Interrupt::SupervisorExternal => TrapType::SupervisorExternal,
            supervisor::Interrupt::SupervisorTimer => {
                // log::error!("[kernel_trap] receive timer interrupt");
                TIMER_MANAGER.check();
                unsafe { set_next_timer_irq() };
                TrapType::Timer
            }
            _ => TrapType::Unknown,
        },
        Ok(Trap::Exception(e)) => match e {
            Exception::StorePageFault
            | Exception::InstructionPageFault
            | Exception::LoadPageFault => {
                log::info!(
                    "[trap_handler] encounter page fault, addr {stval:#x}, instruction {sepc:#x} cause {:?}",
                    e,
                );

                match e {
                    Exception::StorePageFault => TrapType::StorePageFault(stval),
                    Exception::InstructionPageFault => TrapType::InstructionPageFault(stval),
                    Exception::LoadPageFault => TrapType::LoadPageFault(stval),
                    _ => TrapType::Unknown,
                }
            }
            _ => TrapType::Unknown,
        },
        Err(_) => TrapType::Unknown,
    }
}