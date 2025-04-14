use loongArch64::register::{crmd, ecfg};
use::loongArch64::register::ecfg::LineBasedInterrupt;

const HARDWARE_INTERRUPTS_BITS: LineBasedInterrupt = LineBasedInterrupt::HWI0 | 
    LineBasedInterrupt::HWI1 | LineBasedInterrupt::HWI2 | LineBasedInterrupt::HWI3 |
    LineBasedInterrupt::HWI4 | LineBasedInterrupt::HWI5 | LineBasedInterrupt::HWI6 |
    LineBasedInterrupt::HWI7;

// global interrupt status
pub fn is_interrupt_enabled() -> bool {
    crmd::read().ie()
}

// global interrupt status
pub unsafe fn enable_interrupt() {
    #[cfg(feature = "irq")]
    unsafe{
        crmd::set_ie(true);
    }
}

pub unsafe fn disable_interrupt() {
    #[cfg(feature = "irq")]
    unsafe{
        crmd::set_ie(false);
    }
}

// local interrupt
// set interrupt status as bits input (the most flexible one)
pub unsafe fn set_local_interrupt(new_set:usize) {
    let interrupt_flags = match LineBasedInterrupt::from_bits(new_set) {
        Some(flags) => flags,
        None => {
            // if the bits are invalid, truncate or error
            // truncating to the valid bits here
            LineBasedInterrupt::from_bits_truncate(new_set)
        }
    };
    // update
    unsafe {
        ecfg::set_lie(interrupt_flags);
    }
}

pub fn get_local_interrupt() -> usize {
    let cur_lie = ecfg::read().lie();
    cur_lie.bits()
}

pub unsafe fn enable_timer_interrupt() {
    unsafe{
        let cur_lie = ecfg::read().lie();
        let new_lie = cur_lie | LineBasedInterrupt::TIMER;
        ecfg::set_lie(new_lie);
    }
}

pub unsafe fn disable_timer_interrupt() {
    unsafe{
        let cur_lie = ecfg::read().lie();
        let new_lie = cur_lie & !LineBasedInterrupt::TIMER;
        ecfg::set_lie(new_lie);
    }
}   

pub unsafe fn enable_software_interrupt() {
    unsafe{
        let cur_lie = ecfg::read().lie();
        let new_lie = cur_lie | LineBasedInterrupt::SWI0 | LineBasedInterrupt::SWI1;
        ecfg::set_lie(new_lie);
    }
}

pub unsafe fn disable_software_interrupt() {
    unsafe{
        let cur_lie = ecfg::read().lie();
        let new_lie = cur_lie & !(LineBasedInterrupt::SWI0 | LineBasedInterrupt::SWI1);
        ecfg::set_lie(new_lie);
    }
}

// similar to riscv's eternal interrupt
pub unsafe fn enbale_hardware_interrupt() {
    unsafe{
        let cur_lie = ecfg::read().lie();
        let new_lie = cur_lie | HARDWARE_INTERRUPTS_BITS;
        ecfg::set_lie(new_lie);
    }
}

pub unsafe fn disable_hardware_interrupt() {
    unsafe{
        let cur_lie = ecfg::read().lie();
        let new_lie = cur_lie & !HARDWARE_INTERRUPTS_BITS;
        ecfg::set_lie(new_lie);
    }
}

