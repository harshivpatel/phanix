#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use phanix::{QemuExitCode, exit_qemu, serial_print, serial_println};
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    //Set up our new GDT rules and our 20KB emergency backup stack
    phanix::gdt::init();
    
    // Load our custom test alarm table (defined below) into the CPU
    init_test_idt();

    // Deliberately break the system by blowing up the memory stack
    stack_overflow();

    // If the computer somehow reaches this line, our test failed
    panic!("Execution continued after stack overflow");
}

// A broken function that calls itself endlessly to force a stack overflow
#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // Inside here, it calls itself over and over
    volatile::Volatile::new(0).read(); // Prevents the compiler from optimizing this out
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    phanix::test_panic_handler(info);
}

// Build a custom test alarm directory just for this file
lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            // Tie the Double Fault alarm to our custom test handler function
            idt.double_fault
            .set_handler_fn(test_double_fault_handler)
            // Tell the CPU to use our emergency backup stack space (Slot 0)
            .set_stack_index(phanix::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

// Simple function to push our test alarm directory into the CPU chip
pub fn init_test_idt() {
    TEST_IDT.load();
}

// The safe emergency bunker code that runs when the double fault hits
extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    // If the CPU successfully lands here, our backup stack successfully saved the system!
    serial_println!("[ok]");
    
    // Tell the QEMU emulator that the test passed cleanly so it can close
    exit_qemu(QemuExitCode::Success);
    
    loop {}
}