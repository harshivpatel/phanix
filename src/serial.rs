use uart_16550::{Config, Uart16550Tty, backend::PioBackend};
use spin::Mutex;
use lazy_static::lazy_static;

// Normal static variables in Rust must be initialized.
// Here initializing a hardware port requires runtime instructions, so we wrap this in a lazy_static Macro
lazy_static! {
    pub static ref SERIAL1: Mutex<Uart16550Tty<PioBackend>> = Mutex::new(unsafe {
        // Creates an instance of Uart16550Tty driver stuct and initializes UART hardware interface
        Uart16550Tty::new_port(0x3F8, Config::default())
        .expect("failed to initialize UART")
    });
}

// Hidden tella cargo doc to not include this function in documentation
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed")
    })
   
}

// Prints to the host though the serial interface
// Makes macro availabe globally
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

// Prints to the host through the serial interface, appending a newline
#[macro_export]
macro_rules! serial_println {
    // If nothing is passed, just move down one line
    () =>($crate::serial_print!("\n"));
    // If one single string is passed, append \n onto the end at compile time
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    // If a string is passed with a variables like {}, it appends newline and passes passes extra tokens
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}