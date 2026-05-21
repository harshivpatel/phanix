    #![no_std]
    #![no_main]
    #![feature(custom_test_frameworks)]
    #![test_runner(crate::test_runner)]
    #![reexport_test_harness_main = "test_main"]


    mod vga_buffer;

    mod serial;

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
        assert_eq!(1, 1);
    }

    // Any type that implements this trait must provide a "run" function
    pub trait Testable {
        fn run(&self) -> ();
    }
    // A blanket implementation that automatically applies the Testable trait 
    // to ALL functions in the project that accept 0 argument.
    impl<T> Testable for T
    where
        T: Fn(),
        {
            fn run(&self) {
                // Extract and print the full compiler symbol path name of the function over serial
                serial_print!("{}...\t", core::any::type_name::<T>());

                // Execute the actual function body itself
                self();

                // Print the success status
                serial_println!("[ok]");
            }
        }

    #[cfg(not(test))]
    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        println!("{}", info);
        loop {}
    }

    // Panic handler in test mode
    #[cfg(test)]
    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        serial_println!("[failed]\n");
        serial_println!("Error: {}", info);
        exit_qemu(QemuExitCode::Failed);
        loop {}
    }

    #[cfg(test)]
    pub fn test_runner(tests: &[&dyn Testable]) {
        serial_println!("Running {} tests ", tests.len());
        for test in tests {
            test.run();
        }
        exit_qemu(QemuExitCode::Success);
    }
    #[test_case]
    fn trivial_assertions() {
        serial_print!("trivial assertions... ");
        assert_eq!(1, 1);
        serial_println!("[ok]");
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[repr(u32)]

    pub enum QemuExitCode {
        Success = 0x10,
        Failed = 0x11,
    }

    pub fn exit_qemu(exit_code: QemuExitCode) {
        use x86_64::instructions::port::Port;
        unsafe {
            let mut port = Port::new(0xf4);
            port.write(exit_code as u32);
        }
    }