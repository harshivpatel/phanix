# Supervisor Notes — Environment Setup & Build System
**Branch**: `supervisor`
**Date**: 2026-03-01
**From**: Project Supervisor

---

These notes explain every change made to the build configuration on this branch. None of the changes touch your kernel code (`kernel/src/main.rs`). They fix the toolchain and build pipeline so the project actually compiles, produces a bootable disk image, and runs in QEMU.

Read through this before merging into `main`. You should understand every line before continuing development.

---

## What was broken and why

Your `main` branch had four problems that would have prevented any further development:

1. **`.cargo/config.toml` pointed at `x86_64-unknown-none`** — this is a generic bare-metal target that uses a precompiled `core` library built for Linux. It doesn't have the correct settings for kernel development (wrong red zone behaviour, wrong ABI).

2. **`x86_64-phanix.json` was missing from the working tree** — you created it correctly in commit `9aae824` with all the right kernel settings, but a subsequent rebase/force-push dropped it from `HEAD` while leaving it visible in `git log`. Your working tree was silently using the wrong target.

3. **`Cargo.toml` had no `[[bin]]` entry** — without this, Cargo couldn't find `kernel/src/main.rs` as the source file, because Cargo's default is to look for `src/main.rs` at the package root.

4. **No nightly version was pinned** — Rust nightly releases daily and regularly introduces breaking changes to unstable features like `build-std`. Without pinning, your build will break unexpectedly the next time nightly updates.

---

## File-by-file explanation

### `rust-toolchain.toml` *(new file)*

```toml
[toolchain]
channel = "nightly-2026-02-27"
components = ["rust-src", "llvm-tools-preview"]
```

**What it does**: Tells Rustup which exact nightly to use for this project. Anyone who clones the repo gets the same compiler version automatically — no manual `rustup override set` needed.

**Why nightly**: The `build-std` feature (rebuilding `core` from source) is unstable and requires nightly. The `json-target-spec` feature (custom target JSON files) is also unstable.

**The two components**:
- `rust-src` — the source code of `core` and `compiler_builtins`. Without this, `build-std` has nothing to compile.
- `llvm-tools-preview` — provides `llvm-objcopy` and `llvm-objdump`, which `bootimage` uses to strip and inspect the kernel binary.

**Action required from you**: Run `rustup toolchain install nightly-2026-02-27` once. After that, `cargo build` picks it up automatically via this file.

---

### `x86_64-phanix.json` *(restored)*

This is the custom LLVM target specification. You wrote it correctly — it was just missing from `HEAD`. Here is what each field does:

```json
"llvm-target": "x86_64-unknown-none"
```
The base LLVM triple. `none` as the OS means no operating system — bare metal.

```json
"linker-flavor": "ld.lld",
"linker": "rust-lld"
```
Use Rust's bundled LLD linker instead of the system linker (MSVC on Windows, GNU ld on Linux). This makes the build portable and avoids system linker quirks.

```json
"panic-strategy": "abort"
```
On a panic, immediately abort rather than unwind the stack. Unwinding requires OS infrastructure (`libunwind`) that doesn't exist yet.

```json
"disable-redzone": true
```
**This is critical for interrupt safety.** The "red zone" is a 128-byte area below the stack pointer that the System V ABI allows functions to use without adjusting `rsp`. On a normal OS, this is fine. In a kernel, a hardware interrupt can fire *at any point* — including in the middle of a function using the red zone. The CPU will push the interrupt frame into the red zone, corrupting whatever data was there. Disabling the red zone prevents this class of silent memory corruption.

```json
"features": "-mmx,-sse,+soft-float",
"rustc-abi": "x86-softfloat"
```
**Three related settings.** The CPU does not save and restore SSE/MMX registers when handling interrupts (that's the OS's job, and your OS doesn't do it yet). If the compiler emits SSE instructions anywhere in kernel code, an interrupt between two SSE operations corrupts the registers and produces silent data corruption that is nearly impossible to debug.

- `-mmx` — disable MMX instructions
- `-sse` — disable SSE/SSE2/SSE3/AVX instructions
- `+soft-float` — any floating point operations must use software emulation (never SSE)
- `"rustc-abi": "x86-softfloat"` — required by recent nightly (1.81+) to pair with `+soft-float`; tells the Rust compiler the calling convention matches

In practice: don't use `f32`/`f64` in kernel code at all. These flags are a safety net.

---

### `.cargo/config.toml` *(updated)*

```toml
[build]
target = "x86_64-phanix.json"
```
Points the build at your custom target JSON. Was previously `x86_64-unknown-none`.

```toml
[unstable]
build-std = ["core", "compiler_builtins"]
build-std-features = ["compiler-builtins-mem"]
json-target-spec = true
```
- `build-std` — recompiles `core` and `compiler_builtins` from source specifically for your custom target. This is what gives you the correct bare-metal `core` rather than a precompiled one.
- `compiler-builtins-mem` — provides implementations of `memcpy`, `memset`, `memcmp` as compiler intrinsics. Without this, the linker will complain about undefined references to these functions when `core` uses them internally.
- `json-target-spec` — required unstable flag to allow `.json` custom target specs.

```toml
[target.'cfg(target_arch = "x86_64")']
runner = "bootimage runner"
```
When you run `cargo run`, Cargo doesn't know how to execute a bare-metal kernel binary. This line tells it to use `bootimage runner` as a wrapper, which:
1. Calls `bootimage` to stitch your kernel binary and the bootloader into a bootable disk image
2. Launches QEMU with that disk image

---

### `Cargo.toml` *(updated)*

```toml
[[bin]]
name = "phanix"
path = "kernel/src/main.rs"
```
Without this, Cargo looks for `src/main.rs` at the project root. This tells it the actual location.

```toml
[dependencies]
bootloader = "0.9"
```
The `bootimage` tool wraps `bootloader 0.9.x`. This crate provides the 16→32→64-bit mode transition, sets up an initial page table, and calls your `_start` function with a `BootInfo` struct containing the memory map. You don't call it directly yet, but it must be a declared dependency for `bootimage` to find and compile it.

```toml
[package.metadata.bootimage]
run-args = ["-serial", "stdio", "-no-reboot", "-no-shutdown"]
test-args = [
    "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
    "-serial", "stdio",
    "-display", "none"
]
test-success-exit-code = 33
```
QEMU arguments passed automatically when running:

- `-serial stdio` — wires the VM's serial port (COM1) to your terminal. Once you implement a UART driver, `println!` output via serial will appear here. More reliable than VGA for early debugging.
- `-no-reboot` — QEMU stays open after a triple fault instead of silently rebooting. Without this, a crash looks identical to a reboot and you lose the state.
- `-no-shutdown` — QEMU stays open if the kernel executes a shutdown instruction.
- The `test-args` configure `cargo test` to use a special QEMU exit device (lets the kernel signal pass/fail to the test runner) and run headless (no window, faster).

---

### `.gitignore` *(cleaned up)*

- Removed the duplicate `target/` line
- Added `supervisor/` and `.claude/` — supervisor working files that are not part of your submission

---

## How to build and run

```bash
# First time only — install the pinned toolchain
rustup toolchain install nightly-2026-02-27

# Install bootimage (if not already done)
cargo install bootimage

# Build the kernel binary
cargo build

# Build a bootable disk image
cargo bootimage

# Build and launch in QEMU (does both steps above automatically)
cargo run
```

When you run `cargo run`, a QEMU window will open. Right now it shows a **blank black screen** — this is correct. The bootloader runs, enters 64-bit Long Mode, and jumps to `_start()`, which spins in `loop {}`. The kernel is running. It just has nothing to say yet.

Close QEMU: close the window, or press `Ctrl+Alt+G` to release the mouse, then `Ctrl+C` in the terminal.

---

## What to do next

The environment is now fully working. Your next task is the VGA text buffer driver. This is the single most important piece of code in the project — without output, you cannot verify that any subsequent code (exceptions, heap, async) is functioning.

**Immediate priorities:**
1. Implement the VGA text buffer driver — write a character to `0xb8000` and see it appear on screen
2. Implement `print!` / `println!` macros backed by the VGA driver
3. Update the `panic!` handler to print the panic message before halting — right now panics are silent

The Phil Oppermann blog series chapters you need next:
- [VGA Text Mode](https://os.phil-opp.com/vga-text-mode/)
- [Testing](https://os.phil-opp.com/testing/) — set this up early, not at the end

Good luck — the hard part of the environment is done.
