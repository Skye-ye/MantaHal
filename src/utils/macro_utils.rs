/// A macro to conditionally import modules based on target architecture
/// and re-export their contents.
#[macro_export]
macro_rules! arch_modules {
    ($(($arch:literal, $module:ident)),*) => {
        $(
            #[cfg(target_arch = $arch)]
            mod $module;
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

/// Define a default implementation for architecture-specific traits with customizable methods
///
/// This macro creates a default struct and implements the specified trait
/// with methods that panic when called. The implementation is only created
/// when none of the specified target architectures are active.
///
/// # Arguments
///
/// * `$trait_name` - The name of the trait to implement
/// * `$default_impl` - The name of the default implementation struct
/// * `[$($arch),+]` - A list of architecture strings that have specific implementations
/// * `{fn $method(&self, $param: $param_type) -> $ret_type;}` - Method signatures to implement
///
/// # Example
///
/// ```
/// define_arch_specific_trait_impl!(PTOps, DefaultPTOps, ["x86_64", "riscv64"], {
///     fn find_pte(&self, vpn: VirtPageNum) -> Option<&PageTableEntry>;
///     fn map(&self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) -> Result<(), MapError>;
/// });
/// ```
///
/// This creates:
/// - A `DefaultPTOps` struct (only when target_arch is not "x86_64" or "riscv64")
/// - Implements `PTOps` for `DefaultPTOps` with panicking `find_pte` and `map` methods
#[macro_export]
macro_rules! define_arch_specific_trait_impl {
    ($trait_name:ident, $default_impl:ident, [$($arch:expr),+], {
        $(fn $method:ident(&self, $($param:ident : $param_type:ty),*) -> $ret_type:ty;)+
    }) => {
        #[cfg(not(any($(target_arch = $arch),+)))]
        pub struct $default_impl;
        #[cfg(not(any($(target_arch = $arch),+)))]
        impl $trait_name for $default_impl {
            $(
                fn $method(&self, $($param: $param_type),*) -> $ret_type {
                    panic!(concat!(
                        "No architecture-specific implementation available for ",
                        stringify!($trait_name), "::", stringify!($method)
                    ))
                }
            )+
        }
    };
}
