pub mod addr;
pub mod pagetable;
pub mod tlb;

use loongArch64::register::tlbrentry;

use super::config::{
    csr::{PWCH, PWCL},
    mm::{
        DIR_1_SHIFT, DIR_2_SHIFT, DIR_3_SHIFT, DIR_4_SHIFT, PAGE_SHIFT, PAGE_TABLE_LEVELS,
        PTE_SIZE_BITS, PWCL_PTE_WIDTH,
    },
};
use crate::write_csr_loong;

pub fn mm_init(tlbrentry: usize) {
    tlb::tlb_init();
    setup_ptwalker();
    tlb::set_tlb_refill(tlbrentry);
}

/// Setup multi-level page table walker (pwcl, pwch, pgdh, pgdl)
fn setup_ptwalker() {
    // TODO: set pgdl and pgdh
    let mut dir4_i: usize = 0;
    let mut dir4_w: usize = 0;
    let mut dir3_i: usize = 0;
    let mut dir3_w: usize = 0;
    let mut dir2_i: usize = 0;
    let mut dir2_w: usize = 0;
    let mut dir1_i: usize = 0;
    let mut dir1_w: usize = 0;

    // config fields according to page table levels
    if PAGE_TABLE_LEVELS > 4 {
        dir4_i = DIR_4_SHIFT;
        dir4_w = PAGE_SHIFT - PTE_SIZE_BITS;
    }
    if PAGE_TABLE_LEVELS > 3 {
        dir3_i = DIR_3_SHIFT;
        dir3_w = PAGE_SHIFT - PTE_SIZE_BITS;
    }
    if PAGE_TABLE_LEVELS > 2 {
        dir2_i = DIR_2_SHIFT;
        dir2_w = PAGE_SHIFT - PTE_SIZE_BITS;
    }
    if PAGE_TABLE_LEVELS > 1 {
        dir1_i = DIR_1_SHIFT;
        dir1_w = PAGE_SHIFT - PTE_SIZE_BITS;
    }
    let pte_i = PAGE_SHIFT;
    let pte_w = PAGE_SHIFT - PTE_SIZE_BITS;

    let pwcl = pte_i
        | (pte_w << 5)
        | (dir1_i << 10)
        | (dir1_w << 15)
        | (dir2_i << 20)
        | (dir2_w << 25)
        | (PWCL_PTE_WIDTH << 30);

    let pwch = dir3_i | (dir3_w << 6) | (dir4_i << 12) | (dir4_w << 18);

    // write to csr
    write_csr_loong!(PWCL, pwcl);
    write_csr_loong!(PWCH, pwch);
}
