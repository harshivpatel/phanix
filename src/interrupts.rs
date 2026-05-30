use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{gdt, print, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;

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

        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        
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

extern "x86-interrupt" fn timer_interrupt_handler(
_stack_frame: InterruptStackFrame) {
    print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}


#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}

pub const PIC_1_OFFSET: u8 = 32;

// Slave PIC hardware interrupts map back-to-back starting at slot 40
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// Thread-safe wrapper preventing simultaneous raw hardware port access
pub static PICS: spin::Mutex<ChainedPics> = 
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET, // Registers system clock heartbeat line explicitly to vector 32
}
impl InterruptIndex {
    // Converts the named variant into its raw u8 index for PIC confirmation commands
    fn as_u8(self) -> u8 {
        self as u8
    }
    // Converts the index to a pointer-sized integer required for IDT array indexing
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}