use core::time::Duration;

use crate::arch::{context::Context, handler::TrapType, trapframe};

use super::addr::VirtAddr;

// 1.中断控制
pub trait InterruptController {
    fn is_interrupt_enabled() -> bool;
    fn disable_interrupt();
    fn enable_interrupt();
    fn enable_timer_interrupt();
    fn disable_timer_interrupt();
    fn enable_external_interrupt();
    fn set_trap_handler();
}

// 2.时钟和定时器相关接口
pub trait Timer {
    fn get_time() -> usize;
    fn get_time_ms() -> usize;
    fn get_time_sec() -> usize;
    fn get_time_us() -> usize;
    fn get_time_duration() -> Duration;
    fn set_next_time_irq();
    fn set_timer_irq(times: usize);
    fn init_timer();
    fn enable_timer();
    fn disable_timer();
    fn clear_timer();
}

//3.调度接口（任务上下文切换）
pub trait TaskSwitch {
    //使用无栈协程进行调度似乎并没有用到
    fn context_switch(from: *mut Context, to: *const Context);
    fn context_switch_pt(from: *mut Context, to: *const Context, pt_token: usize);
}

// 4.trap处理接口
pub trait TrapOps {
    fn init();
    fn set_kernel_trap();
    fn set_user_trap();
    fn kernel_trap_handler();
    fn trap_handler(tf: &mut trapframe::TrapFrame) -> TrapType;
    fn trap_return(tf: &mut trapframe::TrapFrame);
}

//5.内存管理
//页表相关操作
//TLB相关操作
pub trait TLBOperation {
    fn flush_vaddr(vaddr: VirtAddr);
    fn flush_all();
}
// VirtAddr,PhysAddr,VirtPageNum,PhysPageNum,PageTable,PTEFlags
//内存的分配与释放操作
// heap frame

// 6.boot相关
pub trait Boot {
    fn clear_bss();
    fn print_banner();
}

//7. 设备相关的接口
