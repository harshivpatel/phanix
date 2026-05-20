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
        exit_qemu(QeumuExitCode::Success);
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[repr(u32)]

    pub enum QeumuExitCode {
        Success = 0x10,
        Failed = 0x11,
    }

    pub fn exit_qemu(exit_code: QeumuExitCode) {
        use x86_64::instructions::port::Port;
        unsafe {
            let mut port = Port::new(0xf4);
            port.write(exit_code as u32);
        }
    }