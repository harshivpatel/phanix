# Phanix: A 64-bit Memory-Safe Operating System

Phanix is a 64-bit operating system kernel developed from the ground up using the Rust programming language. The project aims to create a functional software environment from the bare-metal state of hardware, prioritizing memory safety, performance, and system stability.

## Technical Specifications
* **Target Architecture**: The kernel targets the x86_64 architecture, specifically operating in 64-bit Long Mode.
* **Toolchain**: Development utilizes the Rust nightly channel to access experimental features such as `no_std` and the `bootimage` crate for image creation.
* **Memory Management**: The system implements 4-level Paging with Recursive Page Table mapping for address space isolation.
* **Concurrency Model**: A cooperative multitasking model is employed via a specialized Async Task Executor.

## Current Branch Development: VGA Text Buffer Driver & Formatting Subsystem
This branch marks the transition from static hardware writes to a dynamic, thread-safe terminal interface. The following technical implementations have been completed to establish a robust output subsystem.

### Hardware Verification & Driver Output
The VGA driver implementation was verified by executing the kernel within the QEMU emulator. The following image demonstrates the successful initialization of the `WRITER` global static, the functional `println!` macro, and the automated scrolling logic.

<p align="center">
  <img src="docs/vga-driver-result.png" alt="Phanix OS VGA Output" width="700">
  <br>
  <b>Figure 1:</b> <i>Phanix Kernel successfully rendering text and handling a test panic at 0xb8000.</i>
</p>

### High-Level VGA Abstraction
A complete driver for the VGA text buffer (located at `0xb8000`) has been implemented to manage screen output.
* **Encapsulation**: Developed a `Writer` struct to manage the cursor position, color attributes, and a reference to the VGA hardware buffer.
* **Volatile Memory Safety**: Utilized the `volatile` crate to wrap memory-mapped I/O operations, ensuring the Rust compiler does not optimize away "apparent" dead writes to the hardware.
* **Color Management**: Implemented a `ColorCode` system using bit-shifting logic to pack 4-bit foreground and background colors into a single `u8` attribute byte.

### Vertical Scrolling & Buffer Management
To handle continuous output, a vertical scrolling mechanism was developed.
* **Row Shifting**: Implemented a `new_line` method that performs a manual block transfer of `ScreenChar` data from row `n` to row `n-1`.
* **Safe Overflows**: Added logic to clear the bottom row and reset the column position when the buffer limit (25 rows x 80 columns) is exceeded.

### Standard Library Integration (no_std)
The driver was integrated into the Rust ecosystem to provide a familiar developer experience.
* **Trait Implementation**: Implemented `core::fmt::Write` for the `Writer` struct, enabling support for Rust’s core formatting engine.
* **Global Interface**: Created a global `WRITER` instance using `lazy_static` and a **Spinlock (Mutex)** to ensure thread-safe access from any part of the kernel.
* **Custom Macros**: Developed `print!` and `println!` macros that hook into the global `_print` function, allowing for standard string interpolation.

### Diagnostic Panic Handler
The system's error reporting was upgraded from a silent hang to a visual "Kernel Oops" system.
* **Panic Redirection**: The `#[panic_handler]` now utilizes the custom `println!` macro to output `PanicInfo` (message, file, and line number) directly to the screen upon failure.

## Technical Hurdles and Debugging

### 1. Silent Buffer Overwrites (The Scrolling Bug)
* **The Error**: Text would simply disappear or overwrite the same line once the 25th row was reached.
* **The Context**: Testing long strings that exceeded the vertical height of the VGA buffer.
* **The Cause**: The initial implementation lacked a check for the row boundary. The cursor would continue incrementing the memory address beyond the mapped VGA space, potentially corrupting other memory regions.
* **The Resolution**: Implemented the `new_line` function. This function utilizes a nested loop to shift all characters up by one row and clears the 24th row for new input, providing a standard terminal "scrolling" feel.

### 2. Static Initialization & Mutable State
* **The Error**: `error[E0015]: calls in statics are limited to constant functions`.
* **The Context**: Attempting to create a `static WRITER` directly in the global scope.
* **The Cause**: Rust statics are initialized at compile-time. Our `Writer` requires a raw pointer cast and calculation that the compiler cannot guarantee as "constant" at that stage.
* **The Resolution**: Integrated the `lazy_static` crate to defer initialization until the first time the `WRITER` is accessed at runtime, wrapped in a `Spinlock` to allow safe mutability.

### 3. UTF-8 vs. VGA ASCII Constraints
* **The Error**: Printing special characters (like `ə`) resulted in garbled symbols or visual noise.
* **The Context**: Passing standard Rust `&str` (which is UTF-8) to the `write_string` method.
* **The Cause**: The VGA hardware only supports a specific 8-bit character set (Code Page 437). UTF-8 characters can be multiple bytes long, which the hardware interprets as separate, unrelated symbols.
* **The Resolution**: Updated the `write_string` logic with a match arm to filter for the printable ASCII range (`0x20..=0x7e`). Any character outside this range is automatically replaced with a fallback "block" character (`■` / `0xfe`).

## Building and Execution
The kernel is configured to build for the custom x86_64-phanix target. 

```bash
# Compile the kernel and rebuild core library
cargo build

# Create bootable image and execute via QEMU
cargo run
