<<<<<<< HEAD:src/arch/riscv64/irq.rs
pub struct Irq;
=======
use riscv::register::{
    sie, sstatus,
    stvec::{self, TrapMode},
};

pub struct IRQ;
>>>>>>> c44d8d7 (fix some bugs after rebase):src/riscv64/irq.rs

impl Irq {
    pub fn interrupt_init(vs: usize, base_entry: usize) {
        todo!();
    }
    pub fn interrupt_enabled() -> bool {
        sstatus::read().sie()
    }

    pub unsafe fn enable_interrupt() {
        #[cfg(feature = "irq")]
        unsafe {
            sstatus::set_sie();
        }
    }
    pub unsafe fn disable_interrupt() {
        #[cfg(feature = "irq")]
        unsafe {
            sstatus::clear_sie();
        }
    }
    pub unsafe fn enable_timer_interrupt() {
        unsafe {
            sie::set_stimer();
        }
        pub fn disable_timer_interrupt() {
            todo!()
        }

        pub fn enable_software_interrupt() {
            todo!()
        }

        pub fn disable_software_interrupt() {
            todo!()
        }

        // similar to riscv's eternal interrupt
        pub fn enable_hardware_interrupt() {
            todo!()
        }

        pub fn disable_hardware_interrupt() {
            todo!()
        }
    }
}

pub unsafe fn enable_external_interrupt() {
    unsafe {
        sie::set_sext();
    }
}

pub fn get_trap_handler() -> usize {
    stvec::read().bits()
}

pub unsafe fn set_trap_handler(handler_addr: usize) {
    let mut vec = stvec::read();
    vec.set_address(handler_addr);
    vec.set_trap_mode(TrapMode::Direct);
    unsafe { stvec::write(vec) }
}

pub unsafe fn set_trap_handler_vector(handler_addr: usize) {
    let mut vec = stvec::read();
    vec.set_address(handler_addr);
    vec.set_trap_mode(TrapMode::Vectored);
    unsafe { stvec::write(vec) }
}

/// Disable interrupt and resume to the interrupt state before when it gets
/// dropped.

pub struct TrapHandlerGuard {
    trap_handler_before: usize,
}

impl TrapHandlerGuard {
    pub fn new(new_trap_handler: usize) -> Self {
        let trap_handler_before = get_trap_handler();
        unsafe { set_trap_handler(new_trap_handler) }
        Self {
            trap_handler_before,
        }
    }
}

impl Drop for TrapHandlerGuard {
    fn drop(&mut self) {
        unsafe { set_trap_handler(self.trap_handler_before) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IRQVector(pub(crate) usize);

impl IRQVector {
    /// Get the irq number in this vector
    #[inline]
    pub fn irq_num(&self) -> usize {
        unimplemented!()
    }

    /// Acknowledge the irq
    pub fn irq_ack(&self) {
        unimplemented!()
    }
}
