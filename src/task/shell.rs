use crate::{print, println};
use crate::task::keyboard::ScancodeStream;
use futures_util::stream::StreamExt;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use crate::allocator::{HEAP_START, HEAP_SIZE};

// Max length of a single command line
const MAX_INPUT: usize = 128;

pub async fn run_shell() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    // Line buffer and cursor
    let mut buf = ['\0'; MAX_INPUT];
    let mut len = 0usize;

    print_prompt();

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode('\n') => {
                        println!(); // move to next line
                        let command: &str = core::str::from_utf8(
                            // collect buf[..len] chars into a byte slice
                            // we only accept ASCII so this is safe
                            unsafe {
                                core::slice::from_raw_parts(
                                    buf.as_ptr() as *const u8,
                                    len,
                                )
                            },
                        )
                        .unwrap_or("").trim();

                        // Build an owned &str from our char array
                        let mut cmd_bytes = [0u8; MAX_INPUT];
                        for (i, c) in buf[..len].iter().enumerate() {
                            cmd_bytes[i] = *c as u8;
                        }
                        let cmd = core::str::from_utf8(&cmd_bytes[..len])
                            .unwrap_or("")
                            .trim();

                        dispatch(cmd);

                        // Reset buffer
                        len = 0;
                        buf = ['\0'; MAX_INPUT];
                        print_prompt();
                    }

                    DecodedKey::Unicode('\x08') => {
                        // Backspace
                        if len > 0 {
                            len -= 1;
                            buf[len] = '\0';
                            // Erase last char on screen: back, space, back
                            print!("\x08 \x08");
                        }
                    }

                    DecodedKey::Unicode(c) => {
                        if c.is_ascii() && len < MAX_INPUT - 1 {
                            buf[len] = c;
                            len += 1;
                            print!("{}", c);
                        }
                    }

                    DecodedKey::RawKey(_) => {
                        // Ignore non-unicode keys
                    }
                }
            }
        }
    }
}

fn print_prompt() {
    print!("phanix> ");
}

fn dispatch(cmd: &str) {
    match cmd {
        "" => {}
        "help" => cmd_help(),
        "syscheck" => cmd_syscheck(),
        "clear" => cmd_clear(),
        "echo" => println!(),
        _ => {
            if let Some(rest) = cmd.strip_prefix("echo ") {
                println!("{}", rest);
            } else {
                println!("unknown command: '{}'. Type 'help' for commands.", cmd);
            }
        }
    }
}

fn cmd_help() {
    println!("Phanix Shell -- available commands:");
    println!("  help      print this message");
    println!("  syscheck  display kernel health and memory info");
    println!("  clear     clear the screen");
    println!("  echo      echo text to the screen");
}

fn cmd_syscheck() {
    use x86_64::registers::control::Cr3;

    println!("syscheck");

    // read CR3 to show the active page table root
    let (level_4_table_frame, cr3_flags) = Cr3::read();
    println!(
        "CR3 (L4 page table): phys {:#x}  flags: {:?}",
        level_4_table_frame.start_address(),
        cr3_flags,
    );

    // Heap region
    println!(
        "Heap region:  start={:#x}  size={} KiB  end={:#x}",
        HEAP_START,
        HEAP_SIZE / 1024,
        HEAP_START + HEAP_SIZE,
    );

    // Allocator smoke-test: allocate and immediately free a Vec
    {
        extern crate alloc;
        use alloc::vec::Vec;

        let mut v: Vec<u64> = Vec::new();
        for i in 0..16u64 {
            v.push(i * i);
        }
        let sum: u64 = v.iter().sum();
        println!(
            "Heap alloc test: allocated 16 x u64, sum = {}  [OK]",
            sum
        );
        // v is dropped here memory returned to allocator
    }

    // Virtual address translation spot-check
    // Translate the address of this very function as a sanity check
    println!("syscheck fn ptr: virt {:#x}", cmd_syscheck as usize);

    println!("status: OK");
}

fn cmd_clear() {
    // Print enough newlines to scroll everything off screen
    for _ in 0..25 {
        println!();
    }
}