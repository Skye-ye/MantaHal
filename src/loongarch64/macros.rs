#[macro_export]
macro_rules! write_csr_loong {
    ($csr_number:literal, $value:expr) => {
        unsafe {core::arch::asm!("csrwr {},{}", in(reg) $value, const $csr_number);}
    };
    ($csr_number:expr, $value:expr) => {
        unsafe {core::arch::asm!("csrwr {},{}", in(reg) $value, in(reg) $csr_number);}
    };
}
