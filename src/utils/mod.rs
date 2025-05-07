pub mod macro_utils;
pub mod once_cell;

pub type OnceCell<T = ()> = once_cell::OnceCell<T>;
