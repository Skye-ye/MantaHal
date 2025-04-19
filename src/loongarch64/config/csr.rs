pub const CRMD: usize = 0x0; // Current Mode Information
pub const PRMD: usize = 0x1; // Pre-exception Mode Information
pub const EUEN: usize = 0x2; // Extended Component Unit Enable
pub const ASID: usize = 0x18; // Address Space IDentifier 
pub const PGDL: usize = 0x19; // Page Global Directory base address for Lower half address space
pub const PGDH: usize = 0x1A; // Page Global Directory base address for Higher half address space
pub const PDG: usize = 0x1B; // Page Global Directory base address 
pub const PWCL: usize = 0x1C; // Page Walk Controller for Lower half address space
pub const PWCH: usize = 0x1D; // Page Walk Controller for Higher half address space
pub const STLBPS: usize = 0x1E; // STLB Page Size
pub const CPUID: usize = 0x20; // CPU Identity
pub const TLBRENTRY: usize = 0x88; // TLB Refill exception ENTRY address 
pub const TLBREHI: usize = 0x8E; // TLB Refill exception Entry HIgh-order bits

pub const DMW0: usize = 0x180; // Direct Mapping Configuration Window 0
pub const DMW1: usize = 0x181; // Direct Mapping Configuration Window 1
pub const DMW2: usize = 0x182; // Direct Mapping Configuration Window 2
pub const DMW3: usize = 0x183; // Direct Mapping Configuration Window 3
