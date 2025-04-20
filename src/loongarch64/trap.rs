use core::arch::{global_asm, naked_asm};
use crate::{interrupt, trapframe, time, handler};
use crate::handler::{EscapeReason, TrapType};
use loongArch64::register::estat::{self, Exception, Trap};
use loongArch64::register::badv;


/// set fundamental trap settings
pub fn set_trap_vector_base() {
    interrupt::interrupt_init(0, trap_vector_base as usize);
}

#[naked]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_vec() {
    unsafe {
        naked_asm!(
            "
                csrrd   $sp,  KSAVE_CTX    // get the context ptr
                SAVE_REGS                  // save user trapframe

                csrrd   $sp,  KSAVE_KSP    // get previous saved kernel stack pointer
                KERNEL_LOAD_REGS
                addi.d  $sp, $sp, {kernel_trapframe_size}
                ret                       // goto run_user_task
            ",
            kernel_trapframe_size = const trapframe::KERNEL_TRAPFRAME_SIZE
        )
    }
}

#[naked]
#[unsafe(no_mangle)]
pub extern "C" fn user_restore(context: *mut trapframe::TrapFrame) {
    unsafe{
        naked_asm!(
            "
                addi.d  $sp, $sp, -{kernel_trapframe_size}
                KERNEL_SAVE_REGS

                csrwr  $sp, KSAVE_KSP      // save kernel sp (write actually exchange value)
                move   $sp, $a0            // get the context ptr (from args)
                csrwr  $sp, KSAVE_CTX      // save context ptr

                LOAD_REGS
                ertn
            ",
            kernel_trapframe_size = const trapframe::KERNEL_TRAPFRAME_SIZE,
        )
        
    }
}

/// 1、the first time transform to user mode
/// 2、when user trap in kernel, it will trap into the context of this function
pub fn run_user_task(context: &mut trapframe::TrapFrame) -> EscapeReason {
    user_restore(context);
    // user trap arrive here
    loongarch64_trap_handler(context).into()
}


/// check the privilege of the source and goto suitable save function
#[naked]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn trap_vector_base() {
    unsafe{
        naked_asm!(
            "
            .balign 4096
            csrwr   $sp, KSAVE_USP
            csrrd   $sp, 0x1            //read prmd
            andi    $sp, $sp, 0x3       // check privilege(1:0)
            bnez    $sp, {user_vec}     // goto user trap handler (never return here)

            csrrd   $sp, KSAVE_USP     // restore previous sp (kernel sp)
            addi.d  $sp, $sp, -{trapframe_size}  // allocate space for kernel trap frame
            SAVE_REGS             // actually for caller saved regs

            move    $a0, $sp           // pass error frame to handler
            bl      {trap_handler}

            LOAD_REGS
            ertn
            ",
            trapframe_size = const trapframe::TRAPFRAME_SIZE,
            user_vec = sym user_vec,
            trap_handler = sym loongarch64_trap_handler,
        )
    }
}

/// classify the trap type and handle it
fn loongarch64_trap_handler(tf: &mut trapframe::TrapFrame) -> TrapType {
    let estat = estat::read();
    let trap_type = match estat.cause() {
        Trap::Exception(Exception::Breakpoint) => {
            tf.era += 4;
            TrapType::Breakpoint
        }
        Trap::Exception(Exception::AddressNotAligned) => {
            // error!("address not aligned: {:#x?}", tf);
            //unimplemented!();
            TrapType::Unknown
        }
        Trap::Interrupt(_) => {
            let irq_num: usize = estat.is().trailing_zeros() as usize;
            match irq_num {
                // TIMER_IRQ
                time::TIMER_IRQ => {
                    time::clear_timer();
                    TrapType::Timer
                }
                _ => panic!("unknown interrupt: {}", irq_num),
            }
        }
        Trap::Exception(Exception::Syscall) => TrapType::SysCall,
        Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::PageModifyFault) => {
            TrapType::StorePageFault(badv::read().vaddr())
        }
        Trap::Exception(Exception::PageNonExecutableFault)
        | Trap::Exception(Exception::FetchPageFault) => {
            TrapType::InstructionPageFault(badv::read().vaddr())
        }
        // Load Fault
        Trap::Exception(Exception::LoadPageFault)
        | Trap::Exception(Exception::PageNonReadableFault) => {
            TrapType::LoadPageFault(badv::read().vaddr())
        }
        Trap::MachineError(_) => todo!(),
        Trap::Unknown => todo!(),
        _ => {
            panic!(
                "Unhandled trap {:?} @ {:#x} BADV: {:#x}:\n{:#x?}",
                estat.cause(),
                tf.era,
                badv::read().vaddr(),
                tf
            );
        }
    };

    handler::specific_handler(tf, trap_type, 0);
    trap_type
}

global_asm!{
    r"
        .altmacro
        .equ KSAVE_KSP,  0x30   // kernel level temporal stack pointer register
        .equ KSAVE_CTX,  0x31   // kernel level context register
        .equ KSAVE_USP,  0x32   // user level temporal stack pointer register

        .macro KERNEL_SAVE_REGS     //callee saved
            st.d  $s0, $sp, 0*8
            st.d  $s1, $sp, 1*8
            st.d  $s2, $sp, 2*8
            st.d  $s3, $sp, 3*8
            st.d  $s4, $sp, 4*8
            st.d  $s5, $sp, 5*8
            st.d  $s6, $sp, 6*8
            st.d  $s7, $sp, 7*8
            st.d  $s8, $sp, 8*8
            st.d  $ra, $sp, 10*8
            st.d  $fp, $sp, 11*8
            st.d  $tp, $sp, 12*8
            st.d  $sp, $sp, 9*8
            SAVE_FLOAT_REGS $sp, 13*8
        .endm

        .macro KERNEL_LOAD_REGS
            ld.d  $s0, $sp, 0*8
            ld.d  $s1, $sp, 1*8
            ld.d  $s2, $sp, 2*8
            ld.d  $s3, $sp, 3*8
            ld.d  $s4, $sp, 4*8
            ld.d  $s5, $sp, 5*8
            ld.d  $s6, $sp, 6*8
            ld.d  $s7, $sp, 7*8
            ld.d  $s8, $sp, 8*8
            ld.d  $ra, $sp, 10*8
            ld.d  $fp, $sp, 11*8
            ld.d  $tp, $sp, 12*8
            LOAD_FLOAT_REGS $sp, 13*8
            ld.d  $sp, $sp, 9*8
        .endm

        .macro SAVE_REGS
            st.d    $ra, $sp,  1*8
            st.d    $tp, $sp,  2*8
            st.d    $a0, $sp,  4*8
            st.d    $a1, $sp,  5*8
            st.d    $a2, $sp,  6*8
            st.d    $a3, $sp,  7*8
            st.d    $a4, $sp,  8*8
            st.d    $a5, $sp,  9*8
            st.d    $a6, $sp, 10*8
            st.d    $a7, $sp, 11*8
            st.d    $t0, $sp, 12*8
            st.d    $t1, $sp, 13*8
            st.d    $t2, $sp, 14*8
            st.d    $t3, $sp, 15*8
            st.d    $t4, $sp, 16*8
            st.d    $t5, $sp, 17*8
            st.d    $t6, $sp, 18*8
            st.d    $t7, $sp, 19*8
            st.d    $t8, $sp, 20*8
            st.d    $r21,$sp, 21*8
            st.d    $fp, $sp, 22*8
            st.d    $s0, $sp, 23*8
            st.d    $s1, $sp, 24*8
            st.d    $s2, $sp, 25*8
            st.d    $s3, $sp, 26*8
            st.d    $s4, $sp, 27*8
            st.d    $s5, $sp, 28*8
            st.d    $s6, $sp, 29*8
            st.d    $s7, $sp, 30*8
            st.d    $s8, $sp, 31*8
            st.d    $sp, $sp,  3*8

            csrrd	$t0, 0x1
            st.d	$t0, $sp, 8*32  // prmd
            
            csrrd   $t0, 0x6        
            st.d    $t0, $sp, 8*33  // era

            SAVE_FLOAT_REGS $sp, 34*8
        .endm

        .macro LOAD_REGS
            ld.d    $ra, $sp,  1*8
            ld.d    $tp, $sp,  2*8
            ld.d    $a0, $sp,  4*8
            ld.d    $a1, $sp,  5*8
            ld.d    $a2, $sp,  6*8
            ld.d    $a3, $sp,  7*8
            ld.d    $a4, $sp,  8*8
            ld.d    $a5, $sp,  9*8
            ld.d    $a6, $sp, 10*8
            ld.d    $a7, $sp, 11*8
            ld.d    $t0, $sp, 12*8
            ld.d    $t1, $sp, 13*8
            ld.d    $t2, $sp, 14*8
            ld.d    $t3, $sp, 15*8
            ld.d    $t4, $sp, 16*8
            ld.d    $t5, $sp, 17*8
            ld.d    $t6, $sp, 18*8
            ld.d    $t7, $sp, 19*8
            ld.d    $t8, $sp, 20*8
            ld.d    $r21,$sp, 21*8
            ld.d    $fp, $sp, 22*8
            ld.d    $s0, $sp, 23*8
            ld.d    $s1, $sp, 24*8
            ld.d    $s2, $sp, 25*8
            ld.d    $s3, $sp, 26*8
            ld.d    $s4, $sp, 27*8
            ld.d    $s5, $sp, 28*8
            ld.d    $s6, $sp, 29*8
            ld.d    $s7, $sp ,30*8
            ld.d    $s8, $sp, 31*8

            ld.d    $t0, $sp, 32*8
            csrwr   $t0, 0x1        // Write PRMD(PLV PIE PWE) to prmd

            ld.d    $t0, $sp, 33*8
            csrwr   $t0, 0x6        // Write Exception Address to ERA

            LOAD_FLOAT_REGS $sp, 34*8

            ld.d    $sp, $sp, 3*8
    "
}