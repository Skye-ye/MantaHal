#[macro_export]
macro_rules! write_csr_loong {
    ($csr_number:expr, $value:expr) => {
        {
            let val = $value;
            unsafe { core::arch::asm!("csrwr {},{}", in(reg) val, const $csr_number) }
        }
    };
}
