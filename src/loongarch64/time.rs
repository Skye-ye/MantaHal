use core::time::Duration;
use loongArch64::register::time;
use crate::config::{board::clock_freq, time::INTERRUPTS_PER_SECOND};

pub fn get_time() -> usize {
    time::read()
}

pub fn get_time_sec() -> usize {
    time::read() / clock_freq()
}

pub fn get_time_us() -> usize {
    time::read() / (clock_freq() / 1_000_000)
}

pub fn get_time_duration() -> Duration {
    Duration::from_micros(get_time_us() as u64)
}

pub unsafe fn set_next_time_irq(){
    let next_trigger: u64 = (time::read() + clock_freq() / INTERRUPTS_PER_SECOND) as u64;
    
}