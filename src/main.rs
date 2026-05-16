    #![no_std]
    #![no_main]

    mod vga_buffer;

    use core::panic::PanicInfo;

    static PHANIX: &[u8] = b"phanix";

    #[unsafe(no_mangle)]
    pub extern "C" fn _start() -> ! {
        println!("Hello World{}", "!");
        panic!("Testing the custom panic handler!");
        loop {}
    }    

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        println!("{}", info);
        loop {}
    }

