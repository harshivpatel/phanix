#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(phanix::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[unsafe(no_mangle)] // Tells the compiler to not change name of the function during compilation
pub extern "C" fn _start() -> ! { // Forces compiler to use the standard C calling convention for passing parameters
    test_main();

    loop {}
}


fn test_runner(tests: &[&dyn Fn()]) {
    unimplemented!();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    phanix::test_panic_handler(info);
}


use phanix::println;

#[test_case]
fn test_println() {
    println!("test_println output");
}