use crate::println;
use crate::utils::OnceCell;
use fdt::Fdt;

static CPU_ID: usize = 0;

static MY_FDT: &[u8] = include_bytes!("../dtb/test.dtb");

static CPU_NUM: OnceCell<usize> = OnceCell::new();

pub fn parse_dtb_info() {
    let fdt = Fdt::new(MY_FDT).unwrap();

    println!(
        "This is a devicetree representation of a {}",
        fdt.root().model()
    );
    println!(
        "...which is compatible with at least: {}",
        fdt.root().compatible().first()
    );

    println!("Platform Hart Count: {}", fdt.cpus().count());

    fdt.memory().regions().for_each(|mm| {
        println!(
            "Memory Region: {:#X} - {:#X}",
            mm.starting_address as usize,
            (mm.starting_address as usize) + mm.size.unwrap_or(0)
        );
    });

    println!("Boot Args: {:?}", fdt.chosen().bootargs().unwrap_or(""));

    CPU_NUM.init(fdt.cpus().count());
}
