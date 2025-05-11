pub mod macro_utils;
pub mod mutex_no_irq;
pub mod once_cell;

pub type OnceCell<T = ()> = once_cell::OnceCell<T>;
pub type MutexNoIrq<T = ()> = mutex_no_irq::MutexNoIrq<T>;
