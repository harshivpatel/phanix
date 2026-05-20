    #![no_std]
    #![no_main]
    #![feature(custom_test_frameworks)]
    #![test_runner(crate::test_runner)]
    #![reexport_test_harness_main = "test_main"]


    mod vga_buffer;

    use core::panic::PanicInfo;

    static PHANIX: &[u8] = b"phanix";

    #[unsafe(no_mangle)]
    pub extern "C" fn _start() -> ! {
        println!("Hello World{}", "!");
        #[cfg(test)]
        test_main();
        loop {}
    }

    #[test_case]
    fn trivial_assertions() {
        print!("trivial assertions... ");
        assert_eq!(1, 1);
        println!("[ok")
    }

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        println!("{}", info);
        loop {}
    }


    #[cfg(test)]
    pub fn test_runner(tests: &[&dyn Fn()]) {
        println!("Running {} tests ", tests.len());
        for test in tests {
            test();
        }
    }