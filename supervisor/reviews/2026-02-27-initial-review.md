# Supervisor Review — Initial Assessment
**Date**: 2026-02-27
**Project**: Phanix OS (x86_64 Kernel in Rust)
**Student**: Harshiv Patel
**Reviewer**: Project Supervisor
**Demo Due**: First week of June 2026

---

## Executive Summary

The project is at the very earliest stage of development. The student has correctly established a freestanding Rust binary environment and produced a thoughtful README and architecture proposal. However, **no functional kernel code has been written** — the current entry point and panic handler both simply loop forever. The proposal and README demonstrate a solid conceptual grasp of the problem domain, which is encouraging. The path from the current state to a demo-worthy kernel in ~14 weeks is achievable but will require consistent, focused weekly progress starting immediately.

**Overall Grade (current state)**: Early Foundation — Conceptually sound, execution not yet started.

---

## 1. Proposal Review

**Full title**: *Phanix: A 64-bit Memory-Safe Operating System and Interactive Shell*
**Student ID**: 3120107, Griffith College Cork, Ireland

The proposal PDF has been fully reviewed. Assessment follows section by section.

---

### 1.1 Introduction & Motivation (Section 1)

**Strengths:**
- The project motivation is well-articulated: bridging the modern software abstraction gap by going bare-metal, demonstrating how Rust prevents low-level vulnerabilities. This is a genuine pedagogical motivation, not a retrofitted justification.
- The name etymology (Phanes, Greek deity of creation — "creating from primeval bare-metal") is a nice touch; it shows intentional thought about the project as a whole rather than just the technical parts.
- The architectural philosophy section is honest: the proposal acknowledges the original microkernel vision was revised to a monolithic design for practicality. This kind of scope self-awareness is exactly what examiners look for.

**Concerns:**
- The motivation section mentions "buffer overflows" as the key vulnerability Rust prevents — this is correct, but undersells it. Rust prevents the *entire class* of memory safety bugs: use-after-free, dangling pointers, data races, out-of-bounds access. The proposal should be bolder here given this is a core justification for the language choice.
- Section 1.4 Technical Specifications still lists `bootimage` as the toolchain. As noted in the codebase review, this is a dated crate. The proposal should be updated to reflect the final toolchain decision before submission.

---

### 1.2 Aims and Objectives (Section 2)

**Technical Objectives are well-chosen:**
All four objectives — VGA/PS/2 HAL, IDT exception handling, heap allocator, async task executor — are the canonical milestones for this class of project. They form a logical dependency chain (you need output before you can debug exceptions, you need exceptions handled before you set up a heap, etc.). The student clearly understands the ordering.

**The `syscheck` walkthrough (Section 2.4) is a strong addition:**
Walking through a specific user-facing command (type `syscheck` → kernel allocates → async task queries page tables → output to VGA) concretises the abstract architecture into something tangible and demonstrable. This is well-done.

**Original Contribution (Section 2.3) — the strongest part of the proposal:**
> *"Phanix will introduce an original contribution by implementing and benchmarking a dual-allocator strategy. The project will feature both a Bump Allocator and a Fixed-Size Block Allocator. I will perform an empirical comparison of these two strategies within the syscheck utility, measuring allocation speed and fragmentation overhead under simulated kernel workloads."*

This genuinely elevates the project above a tutorial replication. Implementing *two* allocators and empirically comparing them within a live kernel context demonstrates both systems programming depth and analytical thinking. This is the correct kind of differentiation for an undergraduate project.

**However, the benchmarking methodology is underspecified — this needs to be addressed:**

1. **Timing mechanism**: In a bare-metal kernel with no OS time services, how will allocation time be measured? The answer is the `RDTSC` (Read Time-Stamp Counter) x86 instruction, which reads the CPU cycle counter. The proposal does not mention this. The student must specify this.

2. **"Fragmentation overhead" for a Bump Allocator** is technically undefined — a bump allocator doesn't fragment; it simply cannot deallocate individual objects. The correct metric is *memory utilisation* (what percentage of the heap is in use vs. wasted alignment padding) rather than fragmentation. The proposal conflates these.

3. **Allocation sizes**: Evaluation criterion 3 says "allocate 1,000 objects using both allocator types" but doesn't specify sizes. A Fixed-Size Block Allocator's performance is entirely dependent on block size matching — this must be specified in the experimental design.

4. **Statistical rigour**: A single run of 1,000 allocations is noisy. The methodology should specify repeated trials and averaging.

**Recommendation**: Before the next supervisory meeting, the student should write a one-page "experimental protocol" for the benchmarking, specifying: timing mechanism (RDTSC), allocation sizes, number of trials, and metrics (cycles per allocation, memory utilisation percentage).

---

### 1.3 Project Plan and Risk Analysis (Section 3)

**Timeline (Section 3.1):**
```
Week 1–2:  Freestanding binary, VGA drivers, CPU exception handling
Week 3:    Contingency (IDT/Triple Fault debugging)
Week 4–5:  Paging + dual heap allocator
Week 6:    Async executor + command shell
Week 7:    Benchmarking and evaluation
Week 8:    Documentation and bug buffer
```

**What's good:**
- Including a dedicated contingency week (Week 3) for IDT/Triple Fault debugging is realistic and shows experience. Triple faults are a near-universal pain point for first-time kernel developers; the student is right to budget time for them.
- Week 8 as a documentation/buffer week is sensible practice.

**Timeline concerns:**

| Timeline Item | Assessment |
|---|---|
| Week 1–2: VGA + exceptions together | Aggressive. GDT + TSS + IDT with correct double-fault handling is a week of work by itself. VGA driver + `println!` macro is another. These should be separated. |
| Week 4–5: Paging + dual allocator | This is the most ambitious block. Physical frame allocator → page table setup → virtual mapping → heap bootstrap → two allocator implementations. This is realistically 3–4 weeks of focused effort for a first-time implementation. |
| Week 6: Async executor + shell | The async executor (implementing `Future`, `Waker`, `Task` types from scratch) is a week alone. Wiring keyboard input as an async stream on top of that is another. Combining both into one week is very tight. |

**Net assessment**: The 8-week timeline is compressed, but with ~14 weeks available before the June demo (first git commit was Feb 25), there is a healthy real-world buffer beyond what the proposal documents. The student should treat the proposal's Week 3 contingency as permanent flexibility rather than a one-time buffer.

**Risk Analysis (Section 3.2):**

| Risk | Proposal Rating | Supervisor Rating | Notes |
|---|---|---|---|
| Timeline Slippage | High | High | Agreed; weekly milestones are the right mitigation |
| Triple Fault Loops | High | High | Agreed; `qemu -d int` is exactly the right debugging tool |
| Complexity Overload | Medium | High | Underrated. The heap + paging phase is the most likely to cause project scope collapse |
| Rust Toolchain Changes | Low | **Medium** | Nightly Rust breaks things regularly; `rust-toolchain.toml` is required, not optional |
| Memory Safety Leaks | Medium | Medium | The mitigation ("avoid unsafe outside HAL") is the right principle but underestimates how pervasive `unsafe` is in kernel code |

**Missing risk**: There is no mention of **QEMU compatibility or hardware emulation bugs**. In rare cases, QEMU behaves differently from real x86 hardware (particularly around APIC, timing, and certain interrupt edge cases). Since this project runs exclusively on QEMU, this is a low-severity but real risk worth acknowledging.

**Missing risk**: **Allocator design regression** — if the Bump Allocator is implemented first and the Fixed-Size Block Allocator introduction breaks the heap, it can be very difficult to debug. The student should use feature flags or runtime switching to test both allocators independently.

---

### 1.4 Evaluation Criteria (Section 4)

The five criteria are:
1. Boot to 64-bit environment
2. Handle a Division-by-Zero exception without crashing
3. Allocate 1,000 objects using both allocator types
4. Document performance differences between allocators
5. Respond to user commands via shell in multitasking executor

**Assessment**: These are specific, measurable, and achievable — exactly what evaluation criteria should be. Criterion 2 (Division-by-Zero, not just "boot") is a smart choice because it demonstrates the IDT is active and the kernel is robust under fault conditions, not just that it printed a startup message. Criterion 4 ties the analytical benchmarking to a deliverable output.

**One suggestion**: Criteria 1–3 are binary (pass/fail) but Criterion 4 (performance documentation) and Criterion 5 (shell commands) are qualitative. The proposal would benefit from specifying: what commands will the shell support? (`help`, `clear`, `syscheck` at minimum?) and what format will the performance documentation take? (a table in the final report? live output from `syscheck`?).

---

### 1.5 System Diagram (Section 5)

```
+---------------------------------------+
|          User Layer (Shell)           |
+---------------------------------------+
|          Async Task Executor          |
+---------------------------------------+
|  Memory Manager  |  Interrupt Manager |
+---------------------------------------+
|         Bare Metal Hardware           |
+---------------------------------------+
```

The layered diagram is correct and clean. The separation of Memory Manager and Interrupt Manager at the same layer level is architecturally accurate (they're peer subsystems within the kernel). The shell sitting above the async executor is also correct — the shell *is* an async task, not a separate layer.

**What the diagram is missing**: The diagram doesn't show where the VGA and PS/2 drivers sit (they should be at the Hardware level, exposed upward through the Interrupt Manager layer), and there's no indication of the physical memory / page table component within Memory Manager. A v2 of this diagram with these details would be stronger for the final report.

---

### 1.6 References (Section 6)

The three core references are the right ones:
- **Intel SDM** — the authoritative hardware reference; must-cite for any x86 OS project
- **Oppermann (2024), Writing an OS in Rust (Edition 2)** — the exact tutorial series the project follows; correctly cited and acknowledged
- **Tanenbaum & Bos, Modern Operating Systems (4th ed.)** — the canonical OS theory textbook; appropriate for the architectural philosophy sections

The APA citations with inline explanations are well-formatted. One note: the Intel SDM citation says "(2026)" — Intel updates this document continuously and doesn't publish it with annual edition years. The correct citation form is typically "Intel Corporation. (n.d.)" or citing the specific volume and document number (e.g., "Volume 3A, Section 6.10").

---

## 2. Codebase Review

### `kernel/src/main.rs` (the entire codebase)

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    loop{}
}
```

**What is correct:**
- `#![no_std]` and `#![no_main]` are the right attributes for a freestanding kernel binary
- `#[panic_handler]` correctly returns `!` (never type) — the kernel can't call `std::panic!` machinery
- `#[unsafe(no_mangle)]` on `_start` is the correct modern Rust syntax for exporting the entry point with a stable ABI name (the older `#[no_mangle]` form now triggers a warning in recent nightly, so this is actually slightly ahead of most tutorials)
- `extern "C"` calling convention on `_start` is correct for the platform ABI

**What needs immediate improvement:**

| Issue | Severity | Notes |
|---|---|---|
| `_start` loops forever without doing anything | Blocker | Should call initialisation functions, not just loop |
| `panic` handler silently loops | Moderate | Should at minimum print to VGA/serial before halting |
| No VGA output | Blocker | Without output, there is no way to verify the kernel is running |
| No `hlt` in idle loops | Minor | `loop { unsafe { core::arch::x86_64::_mm_pause(); } }` or `hlt` is more correct than a busy spin |

**Code quality notes (positive):**
The use of `unsafe(no_mangle)` syntax is correct for Rust 2024 edition. Using edition 2024 in `Cargo.toml` is a forward-looking choice that's fine but means some tutorial code snippets may need minor adjustments.

### `Cargo.toml`

```toml
[package]
name = "phanix"
version = "0.1.0"
edition = "2024"

[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

**Correct choices:**
- `panic = "abort"` in both profiles is necessary — `no_std` binaries cannot unwind
- Edition 2024 is acceptable (requires nightly toolchain for some features)

**Missing:**
- No `[dependencies]` — nothing is added yet; the VGA driver will require at least `volatile` for safe memory-mapped I/O writes
- No `[[bin]]` entry pointing to `kernel/src/main.rs` — this was added in commit `9aae824` but is not in current HEAD (see git issue below)
- No `rust-toolchain.toml` pinning the nightly version. Without this, the build will break when the nightly updates incompatibly.

### `.cargo/config.toml`

```toml
[build]
target = "x86_64-unknown-none"
```

**Issue:** This should be using the custom `x86_64-phanix.json` target that was properly configured in commit `9aae824` (with `build-std`, `disable-redzone`, soft-float, etc.). Using `x86_64-unknown-none` without rebuilding `core` means the student is relying on a precompiled `core` for Linux/macOS, which may not have the correct settings for kernel development.

### Git History Analysis

```
3bdaa82  Refactor README.md for improved formatting       ← HEAD (main)
9aae824  feat: update configuration + VGA diagram          ← has x86_64-phanix.json, updated config.toml
   |
  1249e25  Create README.md (feat/bare-metal-setup)
   |/
5761dc0  feat: initial freestanding setup
12289b9  chore: add .gitignore and remove target files
1794e66  feat: initial freestanding rust binary setup
```

**Git concern — inconsistent HEAD state**: Commit `9aae824` added `x86_64-phanix.json` and properly updated `.cargo/config.toml` to use it with `build-std` settings. However, `git show HEAD:.cargo/config.toml` reveals the current HEAD tree still contains the old configuration. Similarly, `x86_64-phanix.json` does not appear in `git ls-files`. This suggests a rebase or force-push incident that silently dropped file changes while preserving the commit message in the log.

**Action required**: The student must verify their working tree against what they believe is committed. Running `git diff HEAD` and `git ls-files` will reveal the discrepancy. The target JSON file and updated cargo config need to be restored and re-committed.

**Positive git practices observed:**
- Conventional commit messages (`feat:`, `chore:`) — good practice
- Branching attempted (`feat/bare-metal-setup`) — correct workflow
- Descriptive commit messages with bodies — good

**Git practices to improve:**
- Force-push / rebase incidents that corrupt the working state should be avoided
- Recommend using `git push --force-with-lease` instead of `--force` if rebasing is necessary
- The `Cargo.lock` is in `.gitignore` — this is debatable for a binary project. Convention for Rust binaries is to commit `Cargo.lock`. For a kernel binary, committing it is strongly recommended for reproducible builds.

---

## 3. Technical Architecture Assessment

### Strengths of the chosen approach

1. **Rust `no_std` is state-of-the-art for OS development.** The student is using the right tool; this is how production-grade embedded and kernel code is increasingly written.

2. **The VGA text buffer approach (0xb8000) is the canonical starting point** for x86 kernel development. The sequence diagram shows the student understands memory-mapped I/O and volatile writes.

3. **Cooperative async multitasking is a well-scoped concurrency model** for a first OS. It avoids the complexity of preemptive scheduling while still demonstrating genuine concurrency concepts.

4. **Custom LLVM target JSON** is the right approach for a kernel — it gives full control over ABI, linker, and CPU feature flags without relying on system defaults.

### Technical risks

1. **No red zone warning**: The `.cargo/config.toml` currently points to `x86_64-unknown-none`, not the custom JSON. If the custom JSON is not restored, interrupt handlers will silently corrupt the stack by writing below RSP into the red zone. This is a serious, hard-to-debug bug.

2. **SSE/FPU state in interrupts**: The custom target JSON correctly disables SSE (`-sse`). If the student adds SSE-enabled dependencies or uses `f64` operations, the compiler may emit SSE instructions that will cause a Device Not Available fault on interrupt entry. This is worth monitoring.

3. **Page fault before paging is set up**: Common early-stage bug. The bootloader (whichever is chosen) sets up an identity map; the student must understand what is and isn't mapped before they start the page table work.

4. **Double fault and triple fault**: Without a properly configured TSS and double-fault stack, a stack overflow will cause a double fault, which then causes a triple fault, which causes a silent reset in QEMU — and the student will have no idea what happened. The IDT and TSS must be set up before any complex kernel code runs.

---

## 4. Progress vs. Expected Timeline

Given the project submission deadline of the first week of June 2026, and today being 2026-02-27:

| Expected phase at this date | Actual state |
|---|---|
| Freestanding binary environment | ✅ Done |
| Bootable image in QEMU | ❌ Not done |
| VGA driver with `println!` | ❌ Not done |

The student is approximately **1–2 weeks behind** where they should be if following an aggressive but achievable schedule. This is recoverable but requires immediate action.

**The single most important next step** is getting a bootable image that prints "Hello, World" to the QEMU VGA screen. Everything else builds on this. Until this works, the student cannot verify that *any* subsequent code is functioning.

---

## 5. Recommended Priorities (Next 4 Weeks)

### Week 1 (immediate)
1. Fix git state: restore `x86_64-phanix.json` and update `.cargo/config.toml` to use it
2. Add `rust-toolchain.toml` to pin the nightly toolchain version
3. Add `bootloader` as a dependency and configure `bootimage` runner in `.cargo/config.toml`
4. Verify `cargo build` succeeds and `cargo run` launches QEMU

### Week 2
5. Implement VGA text buffer driver with volatile writes
6. Implement `print!` / `println!` macros (global writer, `lazy_static` or `OnceLock`)
7. Move `panic` handler to print the panic info before looping

### Week 3
8. GDT setup (code segment, stack segment, TSS descriptor)
9. TSS setup with separate double-fault stack
10. IDT setup with handlers for: breakpoint (prints "BREAKPOINT"), double fault (prints "DOUBLE FAULT"), page fault (prints "PAGE FAULT")

### Week 4
11. PIC configuration (8259): timer and keyboard interrupts enabled
12. Timer interrupt handler (increment a tick counter, no preemption needed yet)
13. PS/2 keyboard interrupt handler (read scancodes)

By the end of Week 4 (~March 27), the student should have a bootable kernel that can display text, handle CPU exceptions gracefully, and read keyboard input. This is a solid foundation for the memory management and async work in April–May.

---

## 6. What Would Make This Demo Excellent

The minimum viable demo is a kernel that boots, shows text, and doesn't crash on a keyboard keypress. To make the demo genuinely impressive for an undergraduate project:

- **Working heap allocator**: Being able to use `Box<T>`, `Vec<T>` in kernel code and demonstrate dynamic allocation
- **Async task executor**: Two tasks running cooperatively (e.g., a counter printing to screen while keyboard input is handled)
- **A simple shell**: Even `help`, `echo`, `clear` commands would look polished
- **Unit tests**: Demonstrating `cargo test` runs kernel tests via QEMU test harness is a differentiator

---

## 7. Summary Feedback for Student

### Proposal Feedback

**What the proposal gets right:**
- The project motivation and architectural rationale are clear and genuine
- The `syscheck` walkthrough concretises the abstract design into something demonstrable
- The dual-allocator benchmarking contribution is a genuine differentiator — this is the strongest part of the proposal academically
- Acknowledging the original microkernel design and pivoting to monolithic shows realistic self-assessment
- The evaluation criteria are specific and measurable
- The risk table is thoughtful, especially the dedicated Triple Fault contingency week

**What needs to be revised in the proposal (before final submission):**
1. **Benchmarking methodology** needs a one-page experimental protocol: specify RDTSC as the timing mechanism, correct the "fragmentation" metric for a Bump Allocator (it doesn't fragment — measure memory utilisation instead), specify allocation sizes and trial counts
2. **Evaluation criteria** should specify what shell commands will be supported and what format the allocator comparison output takes
3. **Intel SDM citation** year "(2026)" is incorrect — use "(n.d.)" or cite the specific SDM volume/number
4. **`bootimage` crate reference** should be updated to reflect the final toolchain choice before the final report
5. Consider a more detailed architectural diagram for the final report showing driver placement and the page table component

### Codebase Feedback

**What you're doing well:**
- Correct `no_std`/`no_main` setup, properly structured freestanding binary
- The `unsafe(no_mangle)` syntax is the modern correct form for Rust 2024 — you're slightly ahead of most tutorials on this point
- Conventional commit messages (`feat:`, `chore:`) with descriptive bodies
- The VGA sequence diagram shows design-before-code thinking — keep doing this
- The custom LLVM target JSON (in git history) is technically correct in every detail

**What needs immediate action (in priority order):**
1. **Fix git state**: `x86_64-phanix.json` exists in git history but not in your current working tree. Run `git log --all --oneline` to see the inconsistency, recover the file, and re-commit it
2. **Add `rust-toolchain.toml`**: Without pinning your nightly version, your build will break unexpectedly. Do this now, before you add any dependencies
3. **Get a bootable image in QEMU**: Add the `bootloader` dependency and configure the QEMU runner. Until you can see output on screen, you cannot verify any code you write
4. **VGA text driver next**: This is the single most important piece of code for the entire project — without output you are debugging blind

**What to plan for:**
- The heap + paging phase (your Weeks 4–5) is the most likely place to lose time. Start it with no other distractions and budget an extra week if needed
- Your RDTSC benchmarking will require careful setup — the timer counter can be unreliable in QEMU without `rdtsc` serialisation. Research `cpuid` serialisation fencing before you do the benchmarking experiments
- Be conservative with scope in the demo — a kernel that boots, handles a Division-by-Zero gracefully, runs `syscheck` with allocator benchmarks, and responds to keyboard input is an excellent undergraduate project. Don't add features in the final weeks; polish what you have

---

## 8. Overall Proposal Grade

| Category | Assessment |
|---|---|
| Clarity and structure | Good — well organised with ToC, clear sections |
| Technical depth | Good — correct architecture, right reference material |
| Original contribution | Strong — dual allocator benchmarking is genuinely original |
| Timeline realism | Adequate — compressed but real-world buffer exists; contingency weeks are present |
| Risk analysis | Good — key risks identified; two risks underrated (toolchain, complexity) |
| Evaluation criteria | Good — specific and measurable; methodology detail needed for Criteria 3/4 |
| **Overall** | **Solid proposal; above average for undergraduate level. Revise benchmarking methodology before final report.** |

---

*Next review scheduled: ~4 weeks from now (late March 2026). By then, Phases 1 and 2 should be complete: bootable image, VGA driver with `println!`, GDT/IDT with exception handlers, and PIC keyboard interrupt working.*
