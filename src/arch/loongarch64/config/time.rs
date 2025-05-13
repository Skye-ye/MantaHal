use core::time::Duration;
pub const INTERRUPTS_PER_SECOND: usize = 100;
pub const NANOSECONDS_PER_SECOND: usize = 1_000_000_000;
pub const TIME_SLICE_DUATION: Duration =
    Duration::new(0, (NANOSECONDS_PER_SECOND / INTERRUPTS_PER_SECOND) as u32);

/// Timer IRQ of loongarch64
pub const TIMER_IRQ: usize = 11;
