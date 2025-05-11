use quote::quote;
use syn::{Ident, Type};

pub fn gen_current_ptr(_symbol: &Ident, ty: &Type) -> proc_macro2::TokenStream {
    quote! {
        let base: usize;
        #[cfg(target_arch = "riscv64")]
        ::core::arch::asm!("mv {}, gp", out(reg) base);
        #[cfg(target_arch = "loongarch64")]
        ::core::arch::asm!("move {}, $r21", out(reg) base);
        (base + self.offset()) as *const #ty
    }
}
