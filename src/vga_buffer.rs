#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]

pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[repr(transparent)]
#[allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

struct ColorCode(u8);
impl ColorCode {
    fn new(foreground: Color , background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

use volatile::Volatile;
#[repr(transparent)] // enusres that Buffer has exact same memory layout as the array it has
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}


// writing to screen
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer // 'static makes reference valid for the whole runtime of program
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }
    fn new_line(&mut self) {
        // Iterate from the second row (index 1) to the end
        for row in 1..BUFFER_HEIGHT {
            for col in 0.. BUFFER_WIDTH {
                // Read character from current row and write it to the row above
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row-1][col].write(character);
            }
        }
        // After shifting up, clear the last row and reset the cursor to start
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    // Overwrites everything by a space character
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _=> self.write_byte(0xfe), 
            }
        }
    }

}


use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// Static Writer, so that other modules can use it as an interface

use lazy_static::lazy_static;
use spin::Mutex;
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Red, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}
// Copying the print! and println! macros but we modify _print
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

// This test just prints to VGA Buffer. If it finishes without panicking that means println did not panic
#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}


// Enusres that no panic occurs even if the lines are printed and they are shfited off the screen
#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a sinlge line";
    
    // Print string to VGA buffer; trailing newline (\n) shifts this line up by 1 row
    println!("{}", s);
    
    // Iterate through characters tracking both index (i) and character value (c)
    for (i, c) in s.chars().enumerate() {
        // Read raw byte directly from hardware memory (Row 23) via volatile read
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        
        // Assert that the printed character matches the physical character in video memory
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}
#[test_case]
fn test_println_long_line() {
    // We create a static buffer of 80 'A' characters directly on the stack
    let mut long_line = ['A'; 80];
    
    // Print the 80 'A's as a string slice, then immediately print 'B' to force the wrap
    for c in long_line.iter() {
        print!("{}", c);
    }
    println!("B"); // The 'B' plus trailing newline (\n) triggers the shifts

    // Validate the first row chunk (The 80 'A's) now sitting at BUFFER_HEIGHT - 3
    let writer = WRITER.lock();
    for i in 0..80 {
        let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 3][i].read();
        assert_eq!(char::from(screen_char.ascii_character), 'A');
    }

    // Validate the wrapped character (The single 'B') sitting at BUFFER_HEIGHT - 2
    let wrapped_char = writer.buffer.chars[BUFFER_HEIGHT - 2][0].read();
    assert_eq!(char::from(wrapped_char.ascii_character), 'B');
}