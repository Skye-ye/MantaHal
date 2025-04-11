/// QEMU Loongarch64 Virt Machine:
/// https://github.com/qemu/qemu/blob/master/include/hw/loongarch/virt.h
use crate::macro_utils::register_mut_const;
pub const QEMU_DTB_ADDR: usize = 0x100000;

pub const MAX_HARTS: usize = 2;

register_mut_const!(pub CLOCK_FREQ, usize, 10000000);