use riscv::register::{
    sie, sstatus,
    stvec::{self, TrapMode},
};

pub struct Irq;

impl Irq {
    pub fn interrupt_init(is_vectored: bool, base_entry: usize) {
        // 1. 设置中断处理入口地址和模式
        if is_vectored {
            unsafe {
                Irq::set_trap_handler_vector(base_entry);
            }
        } else {
            unsafe {
                Irq::set_trap_handler(base_entry);
            }
        }

        // 2. 启用特定中断类型（外部/定时器/软件）
        unsafe {
            Irq::enable_timer_interrupt();
            Irq::enable_software_interrupt();
            Irq::enable_hardware_interrupt();
        }

        // 3. 全局中断开关
        unsafe {
            Irq::enable_interrupt();
        }
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
    }
    pub fn disable_timer_interrupt() {
        unsafe {
            sie::clear_stimer();
        }
    }

    pub fn enable_software_interrupt() {
        unsafe {
            sie::set_ssoft();
        }
    }

    pub fn disable_software_interrupt() {
        unsafe {
            sie::clear_ssoft();
        }
    }

    // similar to riscv's eternal interrupt
    pub fn enable_hardware_interrupt() {
        unsafe {
            sie::set_sext();
        }
    }

    pub fn disable_hardware_interrupt() {
        unsafe {
            sie::clear_sext();
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
}

/// Disable interrupt and resume to the interrupt state before when it gets
/// dropped.

#[derive(Debug, Clone, Copy)]
pub struct IRQVector(usize);

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
