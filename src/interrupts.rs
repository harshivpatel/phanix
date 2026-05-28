use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{gdt, println};
use lazy_static::lazy_static;


// A safe, permanent, read-only global variable for our table
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        // Create an empty 256-slot interrupt switchboard
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // Return the configured table to the global variable
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        
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

// Special hardware function that automatically runs when a double fault happens
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
    {
        // Freeze the computer instantly and print out the snapshot to the screen
        panic!("EXCEPTION: DOUBLE FAULT \n{:#?}", stack_frame);
    }


#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}