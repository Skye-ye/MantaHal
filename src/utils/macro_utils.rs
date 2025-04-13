/// A macro to conditionally import modules based on target architecture
/// and re-export their contents.
#[macro_export]
macro_rules! arch_modules {
    ($(($arch:literal, $module:ident)),*) => {
        $(
            #[cfg(target_arch = $arch)]
            mod $module;

            #[cfg(target_arch = $arch)]
            pub use $module::*;
        )*
    };
}

/// A macro to create a bitmask from a bit index.
///
/// # Examples
///
/// ```rust
/// let mask = bit!(3);
/// assert_eq!(mask, 0b1000);
#[macro_export]
macro_rules! bit {
    ($x:expr) => {
        1 << $x
    };
}

/// A macro to create a mutable constant register.
///
/// # Examples
///
/// ```rust
/// register_mut_const!(pub CLOCK_FREQ, usize, 10000000);
/// ```
#[macro_export]
macro_rules! register_mut_const {
    ($(#[$meta:meta])*$name:ident, $type:ty, $value:expr) => {
        $(#[$meta])*
        static mut $name: $type = $value;
        paste::paste! {
            $(#[$meta])*
            pub fn [<$name:lower>]() -> $type {
                unsafe { $name }
            }
        }
        paste::paste! {
            pub fn [<set_ $name:lower>](num: $type) {
                unsafe {
                    $name = num;
                }
            }
        }
    };
    ($(#[$meta:meta])*pub $name:ident, $type:ty, $value:expr) => {
        $(#[$meta])*
        pub static mut $name: $type = $value;
        paste::paste! {
            $(#[$meta])*
            pub fn [<$name:lower>]() -> $type {
                unsafe { $name }
            }
        }
        paste::paste! {
            pub fn [<set_ $name:lower>](num: $type) {
                unsafe {
                    $name = num;
                }
            }
        }
    };
    () => {};
}
