# Phanix: A 64-bit Memory-Safe Operating System

Phanix is a 64-bit operating system kernel developed from the ground up using the Rust programming language. The project aims to create a functional software environment from the bare-metal state of hardware, prioritizing memory safety, performance, and system stability.

## Technical Specifications
* **Target Architecture**: The kernel targets the x86_64 architecture, specifically operating in 64-bit Long Mode.
* **Toolchain**: Development utilizes the Rust nightly channel to access experimental features such as `no_std` and the `bootimage` crate for image creation.
* **Memory Management**: The system implements 4-level Paging with Complete Physical Memory Mapping for address space isolation.
* **Concurrency Model**: A cooperative multitasking model is employed via a specialized Async Task Executor.

---

## Module 1: VGA Text Buffer Driver & Formatting Subsystem
This subsystem marks the transition from static hardware writes to a dynamic, thread-safe terminal interface.

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

---

## Module 2: Automated Integration Testing Framework
This subsystem introduces a scalable, automated integration testing architecture to validate hardware states headlessly.

### Headless Hardware Termination and Exit Port I/O
Automating bare-metal integration testing requires a reliable path to shut down the host emulator upon suite completion.
* **Port-Mapped I/O Communication**: Implemented an automated exit routine using the `x86_64::instructions::port::Port` abstraction. The kernel writes a specific bit pattern to an arbitrary debugging I/O port address at `0xf4`.
* **ISA Debug Exit Mapping**: This write instruction talks to QEMU's emulated hardware interface, causing the virtual machine to shut down immediately. It returns a success status byte of 33 or a failure status byte of 34 directly back to the host machine shell.
* **Exit Status Code Translation**: Configured the build toolchain via `Cargo.toml` to capture the non-zero hardware exit code (33) and map it back to a standard host success flag (0).

---

## Module 3: Interrupt Handling Infrastructure
This subsystem transitions the kernel from a synchronous execution loop to a reactive, asynchronous runtime architecture. The codebase implements low-level CPU exception vectors, remaps the legacy Programmable Interrupt Controller (PIC) chips, configures a Global Descriptor Table (GDT) with an emergency Task State Segment (TSS) stack, and introduces drivers for asynchronous system timers and keyboard matrices.

### Verification Matrix Output
The following capture illustrates the successfully initialized asynchronous system runtime executing inside the QEMU emulator environment:

![Phanix Interrupt Execution Layout](docs/interrupts_success.png)

### Dual-Layer Core Initialization Flow
To establish a stable environment for hardware event tracking, execution setup is split across two explicit boundaries:
* **src/gdt.rs (Global Descriptor Table)**: Configures segment boundaries and sets up a Task State Segment (TSS). The TSS registers an independent, 20 KB emergency stack index dedicated entirely to catching Double Faults. This guarantees that if the main execution path suffers a stack overflow, the CPU can switch over to a clean memory space rather than triggering an unrecoverable triple fault.
* **src/interrupts.rs (Interrupt Descriptor Table)**: Configures and loads the 256-slot IDT switchboard. It maps CPU exception vectors (such as Breakpoints and Double Faults) and hardware interrupt vectors to explicit handler functions using the specialized `"x86-interrupt"` calling convention.

### Programmable Interrupt Controller (PIC) Remapping
The motherboard uses two cascaded Intel 8259 PIC chips (Master and Slave) to manage 15 hardware interrupt lines. By default, these chips map events to vectors 0–15, which directly overlaps with internal CPU exception gates.
* **Vector Offset Shift**: Remapped the Master PIC to start at vector offset 32 (0x20) and the Slave PIC to start at 40 (0x28). This cleanly chains hardware signals right after the CPU's internal reserve layout.
* **Thread-Safe Mutex Handling**: Wrapped the raw PIC ports inside a `spin::Mutex<ChainedPics>` static global wrapper to enforce mutual exclusion and prevent raw port contention during device updates.

### Async Drivers and End of Interrupt (EOI) Signaling
The system implements handlers for the two primary motherboard interrupt channels:
* **The PIT System Timer (IRQ 0 / Vector 32)**: Configured the Programmable Interval Timer to act as the kernel heartbeat. The timer handler captures async ticks and outputs periodic status marks across the display.
* **The Keyboard Controller (IRQ 1 / Vector 33)**: Configured a driver for the Intel 8042 keyboard controller. The handler drains the physical I/O register port address `0x60` on every event. This clears the controller's internal 1-byte data buffer, unlocking its signaling lines so subsequent keypresses can fire. It utilizes the `pc-keyboard` crate state machine to decode raw make/break scancodes into printable Unicode characters based on a US 104-key layout.
* **Explicit EOI Handshakes**: Hardware lines remain locked by the PIC until a confirmation handshake is delivered. The handlers issue an explicit `notify_end_of_interrupt` command to the command ports right before dropping their resource locks to keep lines open for incoming signals.

---

## Technical Hurdles and Debugging

### 1. Silent Buffer Overwrites (The Scrolling Bug)
* **The Cause**: The initial implementation lacked a check for the row boundary. The cursor would continue incrementing the memory address beyond the mapped VGA space, potentially corrupting other memory regions.
* **The Resolution**: Implemented the `new_line` function. This function utilizes a nested loop to shift all characters up by one row and clears the 24th row for new input, providing a standard terminal "scrolling" feel.

### 2. Static Initialization & Mutable State
* **The Cause**: Rust statics are initialized at compile-time. Our `Writer` requires a raw pointer cast and calculation that the compiler cannot guarantee as "constant" at that stage.
* **The Resolution**: Integrated the `lazy_static` crate to defer initialization until the first time the `WRITER` is accessed at runtime, wrapped in a `Spinlock` to allow safe mutability.

### 3. UTF-8 vs. VGA ASCII Constraints
* **The Cause**: The VGA hardware only supports a specific 8-bit character set (Code Page 437). UTF-8 characters can be multiple bytes long, which the hardware interprets as separate, unrelated symbols.
* **The Resolution**: Updated the `write_string` logic with a match arm to filter for the printable ASCII range (`0x20..=0x7e`). Any character outside this range is automatically replaced with a fallback "block" character (`■` / `0xfe`).

### 4. Non-Std Heap Allocations in Test Environments
* **The Cause**: The kernel operates inside a bare-metal `#![no_std]` scope without a configured memory allocator. Heap-allocated dynamic data structures are entirely unavailable at this stage of initialization.
* **The Resolution**: Redesigned the horizontal test suite to use fixed-size stack arrays. Allocating the data block via `let long_line = ['A'; 80];` lets the test cycle through characters locally on the CPU stack frame, removing the dependency on external heap runtimes.

### 5. Invisible Panic Output (The Red-on-Black Bug)
* **The Error**: The kernel would boot into a completely blank, silent screen rather than displaying initialization logs.
* **The Context**: Intentionally triggering a stack overflow to verify the double fault exception framework.
* **The Cause**: The core `WRITER` foreground color was configured to use `Color::Red`. When the stack overflow successfully triggered a Double Fault, the handler invoked a panic. Because the default background color of the VGA card is `Color::Black`, the red text blended into the background canvas, rendering the exception details invisible.
* **The Resolution**: Updated the diagnostic environment to omit the intentional stack crash during normal boot loops and verified text outputs. Added explicit palette overrides inside high-priority tracking blocks to ensure exception details print clearly.

### 6. Vertical Column Cascades (The println Scroll Bug)
* **The Error**: When the timer interrupt began firing, the entire screen text violently scrolled upwards, leaving a single vertical column of dots on the left margin.
* **The Context**: Initializing the system heartbeat ticker loop.
* **The Cause**: The handler utilized the `println!` macro, which appends a trailing newline (`\n`). Because the timer fires multiple times per second, it continuously triggered the VGA buffer's row-shifting logic, scrolling all text off the top edge of the screen.
* **The Resolution**: Swapped out the macro inside the timer handler to use `print!` and changed the character stream to write raw bytes horizontally across the active line without advancing rows.

### 7. Circular Dependency Lockup (The Deadlock Race Condition)
* **The Error**: The kernel would freeze instantly and stop responding to keyboard inputs after printing a few characters.
* **The Context**: Simultaneous printing commands executing from both the main application thread and asynchronous hardware interrupts.
* **The Cause**: The main thread called `println!` and locked the global `WRITER` spinlock. Mid-operation, a timer interrupt fired and paused the thread. The timer handler then called `print!`, attempting to lock the exact same `WRITER`. The handler spun infinitely waiting for the lock to open, while the main thread remained frozen waiting for the interrupt to exit.
* **The Resolution**: Integrated an interrupt-nesting rule within the architecture. Implemented the `x86_64::instructions::interrupts::without_interrupts` helper block inside critical execution sections and testing frameworks. This turns off the CPU's interrupt listening pin during a lock window, ensuring operations complete before an interrupt handler can execute.

### 8. Single Keypress Lock (The Keyboard Buffer Stash)
* **The Error**: The keyboard handler correctly caught the very first keypress, but would never print another character again on subsequent typing.
* **The Context**: Testing initial keypress routines with basic print marks.
* **The Cause**: The initial implementation sent an EOI signal to the PIC but forgot to poll the device register data. The keyboard controller holds the generated scancode byte inside its 1-byte hardware buffer and refuses to generate any subsequent interrupts until that byte is read.
* **The Resolution**: Imported the `x86_64::instructions::port::Port` utility into the handler loop to explicitly read from port `0x60`. Draining the hardware port flushes the chip and resets its internal state machine for the next input.

---

## Building and Execution
The kernel is configured to build for the custom x86_64-phanix target. 

```bash
# Compile the core kernel library and binary packages
cargo build

# Execute the integrated automated testing suite across all test targets
cargo test

# Build a bootable disk image and launch the operating system inside QEMU
cargo run
