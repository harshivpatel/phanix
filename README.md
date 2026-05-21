# Phanix: A 64-bit Memory-Safe Operating System

Phanix is a 64-bit operating system kernel developed from the ground up using the Rust programming language. The project aims to create a functional software environment from the bare-metal state of hardware, prioritizing memory safety, performance, and system stability.

## Technical Specifications
* **Target Architecture**: The kernel targets the x86_64 architecture, specifically operating in 64-bit Long Mode.
* **Toolchain**: Development utilizes the Rust nightly channel to access experimental features such as `no_std` and the `bootimage` crate for image creation.
* **Memory Management**: The system implements 4-level Paging with Recursive Page Table mapping for address space isolation.
* **Concurrency Model**: A cooperative multitasking model is employed via a specialized Async Task Executor.

## Current Branch Development: Automated Integration Testing Framework
This branch marks the transition from isolated, manual hardware validation to a scalable, automated integration testing architecture. The system has been refactored to decouple the core driver code from the runtime binary target, enabling comprehensive testing matrices inside a headless virtual environment.

### Refactored Crate Structural Boundary
To facilitate external testing, the kernel architecture has been split into a distinct library target and binary target to adhere to Rust's compilation constraints.
* **src/lib.rs (Library Crate)**: Serves as the centralized, reusable core of the operating system. It exposes the VGA buffer abstractions, serial port communication pipelines, custom test runners, and global panic hooks.
* **src/main.rs (Binary Crate)**: Acts as a lightweight executable entry point for the standard kernel bootstrap process, linking directly against the internal library crate.

### Independent Integration Test Suites (tests/)
Standalone integration tests are isolated inside the root level directory to guarantee they execute entirely separate from the main application thread.
* **tests/basic_boot.rs**: Implements a baseline boot validation test package. It re-declares low-level configuration attributes to boot independently on top of the raw bootloader environment and programmatically verifies that basic display streams initialize without crashing.
* **tests/should_panic.rs**: Establishes a negative testing framework. By disabling the default test harness via structural flags, it utilizes a custom loop to verify that internal errors and failing assertions trigger defensive crash states exactly when anticipated.

### Headless Hardware Termination and Exit Port I/O
Automating bare-metal integration testing requires a reliable path to shut down the host emulator upon suite completion.
* **Port-Mapped I/O Communication**: Implemented an automated exit routine using the `x86_64::instructions::port::Port` abstraction. The kernel writes a specific bit pattern to an arbitrary debugging I/O port address at `0xf4`.
* **ISA Debug Exit Mapping**: This write instruction talks to QEMU's emulated hardware interface, causing the virtual machine to shut down immediately. It returns a success status byte of 33 or a failure status byte of 34 directly back to the host machine shell.
* **Exit Status Code Translation**: Configured the build toolchain via `Cargo.toml` to capture the non-zero hardware exit code (33) and map it back to a standard host success flag (0). This prevents automated verification environments from misinterpreting a clean virtual machine shutdown as a runtime error.

### Comprehensive Hardware Verification Test Matrix
The unified test framework executes a 4-part testing matrix to mathematically validate the robustness of the text buffer and communication lines:
* **test_println_simple**: A basic smoke test confirming that macro formatting executes without hanging the instruction pointer.
* **test_println_output**: A data integrity test that executes character-by-character string validation. It performs volatile memory reads directly off the physical hardware video card memory address (`0xb8000`) to confirm written text matches displayed bytes.
* **test_println_many**: A stability stress test that prints 200 consecutive lines to force the row-shifting logic to scroll the screen 175 times back-to-back, confirming that memory bounds do not overflow.
* **test_println_long_line**: A boundary limit test that allocates an array of 80 characters directly onto the stack to perform horizontal wrapping verification. It proves that the 81st printed character automatically triggers a line wrap, resetting the column index back to 0 on the next row without causing memory corruption.

---

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

### 4. Non-Std Heap Allocations in Test Environments
* **The Error**: `error[E0433]: cannot find type String in this scope`.
* **The Context**: Creating a dynamic string via `String::new()` inside the horizontal line wrapping test suite.
* **The Cause**: The kernel operates inside a bare-metal `#![no_std]` scope without a configured memory allocator. Heap-allocated dynamic data structures are entirely unavailable at this stage of initialization.
* **The Resolution**: Redesigned the horizontal test suite to use fixed-size stack arrays. Allocating the data block via `let long_line = ['A'; 80];` lets the test cycle through characters locally on the CPU stack frame, removing the dependency on external heap runtimes.

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
