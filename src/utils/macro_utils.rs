/// Declares architecture-specific modules at the crate root based on `target_arch`
/// and creates a `pub mod arch` that re-exports the contents of the active
/// architecture's module.
#[macro_export]
macro_rules! define_arch_mods_and_api {
    ($(($arch:literal, $module:ident)),+ $(,)?) => {
        $(
            #[cfg(target_arch = $arch)]
            mod $module;
        )*

        pub mod arch {
            $(
                #[cfg(target_arch = $arch)]
                pub use $crate::$module::*;
            )*
        }
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
