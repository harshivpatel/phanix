#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

static PHANIX: &[u8] = b"phanix";

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in PHANIX.iter().enumerate() {
        unsafe {
            let line_offset: isize = 0; 
            
            let char_offset = (i as isize * 2) + line_offset;
            let color_offset = (i as isize * 2 + 1) + line_offset;

            *vga_buffer.offset(char_offset) = byte;
            *vga_buffer.offset(color_offset) = 0x0b;
        }
    }
    loop {}
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }