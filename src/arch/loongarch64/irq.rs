use loongArch64::register::ecfg::LineBasedInterrupt;
use loongArch64::register::{crmd, ecfg, eentry};

fn hardware_interrupts_bits() -> LineBasedInterrupt {
    LineBasedInterrupt::HWI0
        | LineBasedInterrupt::HWI1
        | LineBasedInterrupt::HWI2
        | LineBasedInterrupt::HWI3
        | LineBasedInterrupt::HWI4
        | LineBasedInterrupt::HWI5
        | LineBasedInterrupt::HWI6
        | LineBasedInterrupt::HWI7
}

pub struct Irq;

impl Irq {
    // vs: 3-bit : set the spacing between interrupts entry
    // 0 means share the same entry
    // base_enrty : base(31-12) | 0(11-0)
    pub fn interrupt_init(vs: usize, base_entry: usize) {
        ecfg::set_vs(vs);
        // enable all
        Irq::set_local_interrupt(0x1ff);
        eentry::set_eentry(base_entry);
    }

    // global interrupt status
    pub fn interrupt_enabled() -> bool {
        crmd::read().ie()
    }

    /// global interrupt status
    pub fn enable_interrupt() {
        #[cfg(feature = "irq")]
        crmd::set_ie(true);
    }

    /// disable global interrupt
    pub fn disable_interrupt() {
        #[cfg(feature = "irq")]
        crmd::set_ie(false);
    }

    /// local interrupt
    /// set interrupt status as bits input (the most flexible one)
    pub fn set_local_interrupt(new_set: usize) {
        let interrupt_flags = match LineBasedInterrupt::from_bits(new_set) {
            Some(flags) => flags,
            None => {
                // if the bits are invalid, truncate or error
                // truncating to the valid bits here
                LineBasedInterrupt::from_bits_truncate(new_set)
            }
        };
        // update
        ecfg::set_lie(interrupt_flags);
    }

    pub fn get_local_interrupt() -> usize {
        let cur_lie = ecfg::read().lie();
        cur_lie.bits()
    }

    pub fn enable_timer_interrupt() {
        let cur_lie = ecfg::read().lie();
        let new_lie = cur_lie | LineBasedInterrupt::TIMER;
        ecfg::set_lie(new_lie);
    }

    pub fn disable_timer_interrupt() {
        let cur_lie = ecfg::read().lie();
        let new_lie = cur_lie & !LineBasedInterrupt::TIMER;
        ecfg::set_lie(new_lie);
    }

    pub fn enable_software_interrupt() {
        let cur_lie = ecfg::read().lie();
        let new_lie = cur_lie | LineBasedInterrupt::SWI0 | LineBasedInterrupt::SWI1;
        ecfg::set_lie(new_lie);
    }

    pub fn disable_software_interrupt() {
        let cur_lie = ecfg::read().lie();
        let new_lie = cur_lie & !(LineBasedInterrupt::SWI0 | LineBasedInterrupt::SWI1);
        ecfg::set_lie(new_lie);
    }

    // similar to riscv's eternal interrupt
    pub fn enable_hardware_interrupt() {
        let cur_lie = ecfg::read().lie();
        let new_lie = cur_lie | hardware_interrupts_bits();
        ecfg::set_lie(new_lie);
    }

    pub fn disable_hardware_interrupt() {
        let cur_lie = ecfg::read().lie();
        let new_lie = cur_lie & !hardware_interrupts_bits();
        ecfg::set_lie(new_lie);
    }
}

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
