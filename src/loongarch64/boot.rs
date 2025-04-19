use crate::arch::config::{board::MAX_HARTS, csr, mm::KERNEL_STACK_SIZE};

#[unsafe(link_section = ".bss.stack")]
static mut BOOT_STACK: [u8; KERNEL_STACK_SIZE * MAX_HARTS] = [0u8; KERNEL_STACK_SIZE * MAX_HARTS];

#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start() -> ! {
    unsafe {
        core::arch::naked_asm!("
            ori         $t0, $zero, 0x1     # CSR_DMW0_PLV0
            lu52i.d     $t0, $t0, -2048     # UC, PLV0, 0x8000 0000 0000 0001
            csrwr       $t0, {dmw0}         # LOONGARCH_CSR_DMW0
            ori         $t0, $zero, 0x11    # CSR_DMW1_MAT | CSR_DMW1_PLV0
            lu52i.d     $t0, $t0, -1792     # CA, PLV0, 0x9000 0000 0000 0011
            csrwr       $t0, {dmw1}         # LOONGARCH_CSR_DMW1
            ori         $t0, $zero, 0x0     
            csrwr       $t0, {dmw2}         # LOONGARCH_CSR_DMW2
            csrwr       $t0, {dmw3}         # LOONGARCH_CSR_DMW3
            csrwr       $t0, {tlb_rentry}    # LOONGARCH_CSR_TLBRENTRY
            # Goto 1 if hart is not 0
            csrrd       $t1, {cpu_id}       # read cpu from csr
            bnez        $t1, 1f

            # Enable PG 
            li.w		$t0, 0xb0		# PLV=0, IE=0, PG=1
            csrwr		$t0, {crmd}     # LOONGARCH_CSR_CRMD
            li.w		$t0, 0x00		# PLV=0, PIE=0, PWE=0
            csrwr		$t0, {prmd}     # LOONGARCH_CSR_PRMD
            li.w		$t0, 0x00		# FPE=0, SXE=0, ASXE=0, BTE=0
            csrwr		$t0, {euen}     # LOONGARCH_CSR_EUEN
            invtlb      0x0, $zero, $zero


            la.global   $sp, {boot_stack}
            li.d        $t0, {boot_stack_size}
            add.d       $sp, $sp, $t0       # setup boot stack
            csrrd       $a0, {cpu_id}       # cpuid
            la.global   $t0, {entry}
            jirl        $zero,$t0,0

        1:
            li.w        $s0, {MBUF0}
            iocsrrd.d   $t0, $s0
            la.global   $t1, {sec_entry}
            bne         $t0, $t1, 1b
            jirl        $zero, $t1, 0
            ",
            dmw0 = const csr::DMW0,
            dmw1 = const csr::DMW1,
            dmw2 = const csr::DMW2,
            dmw3 = const csr::DMW3,
            tlb_rentry = const csr::TLBRENTRY,
            cpu_id = const csr::CPUID,
            crmd = const csr::CRMD,
            prmd = const csr::PRMD,
            euen = const csr::EUEN,
            boot_stack_size = const KERNEL_STACK_SIZE,
            boot_stack = sym BOOT_STACK,
            MBUF0 = const loongArch64::consts::LOONGARCH_CSR_MAIL_BUF0,
            entry = sym rust_main,
            sec_entry = sym rust_secondary_main,
        )
    }
}

// Main entry point after initialization
#[unsafe(no_mangle)]
pub fn rust_main(hart_id: usize) -> ! {
    // Placeholder
    loop {}
}

#[unsafe(no_mangle)]
pub fn rust_secondary_main(hart_id: usize) -> ! {
    // Placeholder
    loop {}
}
