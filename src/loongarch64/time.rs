use core::time::Duration;
use loongArch64::register::tcfg;
use loongArch64::time::Time;
use crate::config::{board::clock_freq, time::INTERRUPTS_PER_SECOND};

pub fn get_time() -> usize {
    Time::read()
}

pub fn get_time_ms() -> usize {
    Time::read() / (clock_freq() / 1_000)
}

pub fn get_time_sec() -> usize {
    Time::read() / clock_freq()
}

pub fn get_time_us() -> usize {
    Time::read() / (clock_freq() / 1_000_000)
}

pub fn get_time_duration() -> Duration {
    Duration::from_micros(get_time_us() as u64)
}

// reset
pub unsafe fn set_next_time_irq(){
    let next_trigger: usize = ((clock_freq() / INTERRUPTS_PER_SECOND) + 3) & !3 as usize;
    tcfg::set_init_val(next_trigger);
}

// set a longer time slice
pub unsafe fn set_timer_irq(times: usize) {
    let next_trigger: usize = ((times * clock_freq() / INTERRUPTS_PER_SECOND) + 3) & !3 as usize;
    tcfg::set_init_val(next_trigger);
}

pub fn init_timer() {
    let ticks: usize = ((clock_freq() / INTERRUPTS_PER_SECOND) + 3) & !3;  // round up to 4
    tcfg::set_periodic(true); // set timer to one-shot mode
    tcfg::set_init_val(ticks); // set timer initial value
    tcfg::set_en(true); // enable timer

    // interrupt enable implemented in other file
}