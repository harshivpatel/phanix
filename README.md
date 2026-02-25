Phanix: A 64-bit Memory-Safe Operating System
Phanix is a 64-bit operating system kernel developed from the ground up using the Rust programming language. The project aims to create a functional software environment from the bare-metal state of hardware, prioritizing memory safety, performance, and system stability.

Architectural Design: Modular Monolithic Kernel
Phanix follows a monolithic architecture where core system services—such as memory management, hardware drivers, and task scheduling—operate within the same kernel space. This design is chosen for its efficiency in execution and direct access to hardware, while leveraging Rust's ownership model to maintain the memory safety typically absent in traditional C-based monolithic kernels.

Technical Specifications
Target Architecture: x86_64 (64-bit Long Mode).

Toolchain: Rust nightly (no_std) and the bootimage crate.

Memory Management: 4-level Paging with Recursive Page Table mapping.

Concurrency Model: Cooperative multitasking via an Async Task Executor.

Project Objectives
The primary aim is to develop a bootable kernel capable of managing hardware resources independently of an existing operating system. Key technical objectives include:

Hardware Abstraction Layer (HAL): Implementation of VGA drivers for visual output and PS/2 keyboard drivers for user input.

Interrupt Handling: Configuration of an Interrupt Descriptor Table (IDT) to safely handle CPU exceptions and hardware signals.

Memory Architecture: Implementation of a custom heap allocator for dynamic memory management.

Execution Environment: A cooperative task executor to manage a command-line interface (CLI) and background system checks.

Current Development Status
The project is currently in the initial phase of development.

Environment: Established a freestanding Rust binary environment targeting x86_64-unknown-none.

Entry Point: Implemented a custom _start entry point and a non-returning panic handler.

Next Milestone: Implementation of the VGA Text Buffer driver for system logging.