#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(phanix::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use phanix::memory::active_level_4_table;
use x86_64::VirtAddr;
use phanix::println;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use phanix::memory::active_level_4_table;
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    phanix::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 Entry {}, {:?}", i, entry);
        }
    }

    #[cfg(test)]
    test_main();

    println!("It did not crash");

    phanix::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    phanix::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    phanix::test_panic_handler(info)
}