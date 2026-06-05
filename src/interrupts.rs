use x86_64::instructions::port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;

// Explicitly pull in your custom GDT and VGA printing macros from the crate root
use crate::{gdt, print, println};
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
        
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
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

// extern "x86-interrupt" fn keyboard_interrupt_handler(
//     _stack_frame: InterruptStackFrame
// ) {
//     use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
//     use spin::Mutex;
//     use x86_64::instructions::port::Port;

//     // Thread-safe state container preserving modifier states across async interrupts
//     static KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
//         Mutex::new(Keyboard::new(
//             ScancodeSet1::new(),
//             layouts::Us104Key,
//             HandleControl::Ignore,
//         ));

//     let mut keyboard = KEYBOARD.lock();
//     let mut port = Port::new(0x60);

//     // Read from port 0x60 to drain the hardware buffer and clear the line
//     let scancode: u8 = unsafe { port.read() };

//     // Pass the raw byte to the state machine for processing
//     if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
//         // Unify character generation and handle active shift states
//         if let Some(key) = keyboard.process_keyevent(key_event) {
//             match key {
//                 DecodedKey::Unicode(character) => print!("{}", character),
//                 DecodedKey::RawKey(key) => print!("{:?}", key),
//             }
//         }
//     }

//     unsafe {
//         // Send end of interrupt confirmation to clear the master PIC line
//         PICS.lock()
//             .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
//     }
// }

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

use x86_64::structures::idt::PageFaultErrorCode;
use crate::hlt_loop;

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    
    println!("EXCEPTION: PAGE FAULT");
    print!("Accessed Address: {:?}", Cr2::read());
    print!("Error Code: {:?}", error_code);
    print!("{:#?}", stack_frame);
    hlt_loop();
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
    Keyboard,
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