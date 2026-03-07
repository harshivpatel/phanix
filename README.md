# Phanix: A 64-bit Memory-Safe Operating System

Phanix is a 64-bit operating system kernel developed from the ground up using the Rust programming language. The project aims to create a functional software environment from the bare-metal state of hardware, prioritizing memory safety, performance, and system stability.

## Technical Specifications
* **Target Architecture**: The kernel targets the x86_64 architecture, specifically operating in 64-bit Long Mode.
* **Toolchain**: Development utilizes the Rust nightly channel to access experimental features such as no_std and the bootimage crate for image creation.
* **Memory Management**: The system implements 4-level Paging with Recursive Page Table mapping for address space isolation.
* **Concurrency Model**: A cooperative multitasking model is employed via a specialized Async Task Executor.


## Current Branch Development: Bare-Metal Infrastructure and Verification
This branch focuses exclusively on the transition from a hosted freestanding binary to a verifiable bare-metal execution environment. The following technical implementations have been completed within this scope.

### Custom Target Configuration
A specialized target specification file has been developed to define the hardware constraints of the Phanix environment.
* **Target Triple**: The system targets x86_64-unknown-none to eliminate dependencies on existing operating system ABIs.
* **Linker Strategy**: The configuration utilizes the LLD linker with the rust-lld driver and the ld.lld flavor for cross-platform compatibility.
* **Panic Behavior**: The panic strategy is set to abort to comply with the lack of stack unwinding support in a bare-metal context.
* **Instruction Set Restrictions**: MMX and SSE features are disabled, and soft-float is enabled to prevent the use of large SIMD registers that complicate interrupt handling.
* **Red Zone Disable**: The x86_64 red zone optimization is disabled to ensure that hardware interrupts do not overwrite data on the kernel stack.

### Kernel Entry and Freestanding Execution
The kernel entry logic has been established to allow the CPU to begin execution without a standard library.
* **Entry Point**: A custom _start function has been implemented using the C calling convention and the no_mangle attribute to serve as the default linker entry point.
* **Panic Handling**: A freestanding panic handler has been implemented that enters an infinite loop, preventing undefined behavior upon system failure.

### Hardware Verification via VGA Buffer
To verify successful booting and memory mapping, a minimal hardware handshake was implemented.
* **Visual Output**: The system successfully renders the string "phanix" in light cyan text.

![Phanix Victory Screenshot](docs/qemu_entry.png)

## Technical Hurdles and Debugging

This section documents the primary obstacles encountered during the infrastructure setup and the technical reasoning behind their resolutions.

### 1. Segmentation Fault on Host Execution
* **The Error**: `Segmentation fault (core dumped)` after running `cargo run`.
* **The Context**: Attempting to execute the kernel binary directly in the Linux terminal.
* **The Cause**: The binary is compiled for a freestanding `x86_64-phanix` target. It lacks a standard `main` function and the C runtime required by Linux. When executed as a native process, the CPU triggers a fault because it cannot find the expected OS entry points.
* **The Resolution**: Configured `.cargo/config.toml` with a custom runner: `runner = "bootimage runner"`. This forces Cargo to wrap the binary in a bootloader and execute it via QEMU instead of the host OS.



### 2. Bootloader Mapping Conflict (Red Screen Panic)
* **The Error**: QEMU display turned red with the message `panicked at src/page_table.rs: PageAlreadyMapped`.
* **The Context**: The `bootloader` crate was attempting to transition the CPU into 64-bit Long Mode.
* **The Cause**: A bug in the initial `bootloader v0.9.0` dependency caused a collision between the kernel's memory segments and the recursive page table entries. The bootloader tried to map a memory page that was already occupied.
* **The Resolution**: Updated `Cargo.toml` to `bootloader = "0.9.23"`. Additionally, performed a `cargo clean` to ensure all previous memory artifacts were purged from the build cache.


### 3. VGA Buffer Headless/Blind Output
* **The Error**: QEMU launched but displayed a permanent black screen, even though code was technically correct.
* **The Context**: Implementing the first hardware write to `0xb8000`.
* **The Cause**: The QEMU configuration in `test-args` included `-display none` or an incomplete `-display` flag, which suppressed the graphical window. Simultaneously, the `line_offset` was set to the bottom of the screen where it was obscured by BIOS status text.
* **The Resolution**: Removed conflicting display flags from `Cargo.toml` and adjusted the `_start` logic to write to the top-left of the buffer (index 0) to ensure immediate visibility.

## Building and Execution
The kernel is configured to build for the custom x86_64-phanix target. The build-std feature is utilized in the local Cargo configuration to recompile the core and compiler_builtins crates for the specific hardware target.

```bash
# Compile the kernel and rebuild core library
cargo build

# Create bootable image and execute via QEMU
cargo run
