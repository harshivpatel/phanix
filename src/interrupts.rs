use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use lazy_static::lazy_static;


// Create a safe, permanent, read-only global variable for our table
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        // Create an empty 256-slot interrupt switchboard
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // Return the configured table to the global variable
        idt
    };
}

pub fn init_idt() {
    // Tell the physical CPU chip where to find our table in memory
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(
    // Receives the hardware snapshot saved by the CPU
    stack_frame: InterruptStackFrame
) {
    // Print out the entire snapshot
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}