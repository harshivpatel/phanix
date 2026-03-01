# Phanix OS — Supervisor Context File
**Full title**: Phanix: A 64-bit Memory-Safe Operating System and Interactive Shell
**Student**: Harshiv Bharat Patel (hp200378@gmail.com, ID: 3120107)
**Institution**: Griffith College Cork, Ireland
**Demo deadline**: First week of June 2026
**First supervisor review**: 2026-02-27
**Last updated**: 2026-03-01

---

## Project Goal

Build a bootable x86_64 OS kernel in Rust (`no_std`) from bare metal, featuring:
- VGA text output + PS/2 keyboard input
- CPU exception handling via a configured IDT
- **Dual heap allocator**: Bump Allocator AND Fixed-Size Block Allocator, with empirical benchmarking comparison (this is the original academic contribution)
- Cooperative async task executor
- Interactive CLI shell with a `syscheck` command that queries the page table state and benchmarks the active allocator

The name derives from *Phanes*, the Greek primordial deity of creation — reflecting the project's aim of creating a runtime from bare-metal hardware.

---

## Original Academic Contribution

The standout element of the proposal: implement **both** a Bump Allocator and a Fixed-Size Block Allocator, then run an empirical comparison inside the live kernel via `syscheck`. This distinguishes the project from a straight tutorial follow-along.

### `syscheck` user flow (from proposal §2.4)
1. User types `syscheck` into the shell
2. Kernel allocates memory to store a temporary report string
3. An async task queries the page table state and benchmarks the currently active heap allocator
4. Task outputs system health + allocator performance metrics to the VGA buffer

### Benchmarking methodology — what is specified vs. what is missing

| Item | Proposal says | Needs clarifying |
|---|---|---|
| Timing mechanism | Not named | Must use **RDTSC** (x86 cycle counter) — the only option on bare metal |
| Metric for Bump Allocator | "fragmentation overhead" | **Wrong** — a bump allocator doesn't fragment; measure **memory utilisation** (bytes requested vs. bytes consumed incl. alignment padding) |
| Metric for Fixed-Size Block | "allocation speed" | Correct; also measure utilisation efficiency |
| Allocation sizes | Not specified | Must be specified — Fixed-Size Block performance is entirely block-size-dependent |
| Trial count | "1,000 objects" | Should be repeated (e.g., 5 × 1,000) and averaged; single run is noisy |
| RDTSC serialisation | Not mentioned | `cpuid` fencing needed before RDTSC for reliable timing in QEMU |

**Action item for student**: Write a one-page experimental protocol before starting the benchmarking phase.

---

## Evaluation Criteria (from proposal §4)

These are the student's own stated criteria — use these at the demo:

1. Boot successfully into a 64-bit environment
2. Handle a Division-by-Zero exception **without crashing** (IDT active and working)
3. Successfully allocate 1,000 objects using **both** allocator types
4. Document performance differences between the two allocators
5. Respond to user commands via the shell in the multitasking executor

---

## Supervisor Machine Dev Environment (configured 2026-03-01)

| Tool | Version | Location |
|---|---|---|
| Rust nightly | 1.95.0-nightly (3a70d0349 2026-02-27) | `~/.cargo/bin/` |
| rustup | 1.28.2 | `~/.cargo/bin/` |
| cargo-bootimage | 0.10.4 | `~/.cargo/bin/bootimage.exe` |
| QEMU | 10.2.0 | `C:\Program Files\qemu\` (in user PATH) |
| MSVC Build Tools | 2019 v14.29 | Pre-existing |

**Build commands** (from `F:/Harshiv/phanix/`, any terminal with PATH refreshed):
```bash
cargo build              # compile kernel binary only
cargo bootimage          # compile + stitch into bootable disk image
cargo run                # compile + disk image + launch QEMU
```

**Current observable behaviour**: `cargo run` opens a QEMU window with a blank black screen. The bootloader runs, enters 64-bit Long Mode, jumps to `_start()`, which spins in `loop {}`. Kernel is alive but silent — no output until VGA driver is written.

---

## Git State (as of 2026-03-01)

### Branches
| Branch | HEAD commit | Purpose |
|---|---|---|
| `main` | `3bdaa82` | Student's working branch |
| `origin/feat/bare-metal-setup` | `3bdaa82` | Student's feature branch (merged into main) |
| `supervisor` | `8fd57ef` | Supervisor environment fixes — pushed 2026-03-01 |

### What is on the `supervisor` branch (vs `main`)
Single commit `8fd57ef` — *"chore: fix build environment and restore bare-metal target configuration"*

| File | Change |
|---|---|
| `x86_64-phanix.json` | **Restored** — was in git history (`9aae824`) but missing from HEAD due to rebase incident |
| `rust-toolchain.toml` | **New** — pins `nightly-2026-02-27` with `rust-src` + `llvm-tools-preview` |
| `.cargo/config.toml` | **Updated** — custom JSON target, `build-std`, `json-target-spec`, `bootimage runner` |
| `Cargo.toml` | **Updated** — `[[bin]]` entry, `bootloader = "0.9"`, QEMU run/test args |
| `Cargo.lock` | **Committed** — was gitignored; added for reproducible builds |
| `.gitignore` | **Cleaned** — removed duplicate line, added `supervisor/` and `.claude/` |
| `SUPERVISOR_NOTES.md` | **New** — student-facing explanation of every change with rationale |

### What the student needs to do
```bash
git fetch origin
git checkout supervisor
rustup toolchain install nightly-2026-02-27
cargo install bootimage
cargo run
```
The `SUPERVISOR_NOTES.md` at the project root explains every file change in detail.

### Build quirks resolved during setup (for future reference)
Three bugs had to be fixed vs. the original target JSON commit to work on `nightly-2026-02-27`:
1. `target-pointer-width` / `target-c-int-width` must be integers not strings
2. `+soft-float` in `features` requires `"rustc-abi": "x86-softfloat"` on nightly ≥ 1.81
3. `[target.*.runner]` TOML header syntax was malformed — correct form is `[target.*]` with `runner =` inline

---

## Current Codebase State (2026-03-01)

| Component | Status | Notes |
|---|---|---|
| `no_std` freestanding binary | ✅ Done | Correct `#![no_std]`, `#![no_main]` |
| `_start` entry point | ✅ Done | Bare `loop {}` — kernel reaches here via bootloader |
| Panic handler | ⚠️ Partial | Silent `loop {}` — must print before halting once VGA exists |
| `x86_64-phanix.json` target spec | ✅ Fixed | Restored on `supervisor` branch |
| `.cargo/config.toml` | ✅ Fixed | Correct target, `build-std`, runner configured |
| `rust-toolchain.toml` | ✅ Fixed | Pinned to `nightly-2026-02-27` |
| `bootloader` dependency + `cargo run` | ✅ Fixed | `bootimage 0.10.4` + `bootloader 0.9.34` working |
| Bootable disk image | ✅ Working | `target/x86_64-phanix/debug/bootimage-phanix.bin` (53KB) |
| VGA text driver | ❌ Not started | **Next immediate task** |
| `print!` / `println!` macros | ❌ Not started | Depends on VGA driver |
| GDT / TSS | ❌ Not started | |
| IDT / exception handling | ❌ Not started | |
| Serial UART output | ❌ Not started | |
| PIC + hardware interrupts | ❌ Not started | |
| PS/2 keyboard driver | ❌ Not started | |
| Physical frame allocator | ❌ Not started | |
| Virtual memory / paging | ❌ Not started | |
| Heap allocator (Bump) | ❌ Not started | |
| Heap allocator (Fixed-Size Block) | ❌ Not started | |
| Async task executor | ❌ Not started | |
| CLI shell + `syscheck` | ❌ Not started | |

### File structure (current, `supervisor` branch)
```
phanix/
├── .cargo/config.toml        # Custom JSON target, build-std, bootimage runner
├── Cargo.toml                # [[bin]], bootloader dep, QEMU metadata
├── Cargo.lock                # Committed (bootloader 0.9.34 locked)
├── rust-toolchain.toml       # nightly-2026-02-27
├── x86_64-phanix.json        # Custom LLVM bare-metal target spec
├── SUPERVISOR_NOTES.md       # Student-facing explanation of all changes
├── kernel/src/main.rs        # 14 lines: _start{loop{}} + panic{loop{}}
├── docs/
│   └── Phanix_Proposal_v2.pdf
├── supervisor/               # Gitignored — supervisor working files
│   ├── context.md            # This file
│   └── reviews/
│       └── 2026-02-27-initial-review.md
└── README.md
```

---

## Planned Architecture (from proposal)

- **Target**: x86_64 Long Mode
- **Toolchain**: Rust nightly, `no_std`, `bootimage 0.10` / `bootloader 0.9`
- **Memory**: 4-level paging; proposal says Recursive Page Table, `OffsetPageTable` is simpler and equally valid for this scope
- **Concurrency**: Cooperative multitasking, single-threaded async/await task executor
- **HAL**: VGA text buffer (0xb8000), PS/2 keyboard (interrupt-driven, async stream)
- **Kernel layers**: Bare Metal → [Memory Manager | Interrupt Manager] → Async Task Executor → Shell

### Custom LLVM target JSON (final verified version)
```json
{
    "llvm-target": "x86_64-unknown-none",
    "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
    "arch": "x86_64",
    "target-endian": "little",
    "target-pointer-width": 64,
    "target-c-int-width": 32,
    "os": "none",
    "executables": true,
    "linker-flavor": "ld.lld",
    "linker": "rust-lld",
    "panic-strategy": "abort",
    "disable-redzone": true,
    "features": "-mmx,-sse,+soft-float",
    "rustc-abi": "x86-softfloat"
}
```

---

## Dependencies to Add (in order, as student progresses)

Already in `Cargo.toml`:
```toml
bootloader = "0.9"   # provides bootloader; required by bootimage
```

To be added per phase:
```toml
# Phase 1 — VGA driver
volatile = "0.2"
lazy_static = { version = "1.0", features = ["spin_no_std"] }
spin = "0.5"

# Phase 2 — CPU exceptions + interrupts
x86_64 = "0.15"
uart_16550 = "0.2"
pic8259 = "0.10"
pc-keyboard = "0.7"

# Phase 3 — heap allocator scaffold (or implement from scratch)
linked_list_allocator = "0.9"
```

---

## Student's Proposed Timeline (from proposal §3.1)

```
Week 1–2:  Freestanding binary, VGA drivers, basic CPU exception handling
Week 3:    Contingency — IDT / Triple Fault debugging
Week 4–5:  Paging + dual heap allocator (Bump + Fixed-Size Block)
Week 6:    Async executor + command shell integration
Week 7:    Comparative benchmarking + evaluation (syscheck)
Week 8:    Final report documentation + bug buffer
```

First commit: Feb 25, 2026. ~14 real weeks to June demo vs. 8 planned — buffer exists but must not be squandered. As of Mar 1 the student is at the end of Week 1 per their own plan; VGA driver is the immediate deliverable.

---

## Supervisor's Recommended Pacing

### Phase 1 — Bootable & Visible (by ~Mar 13)
- [x] Fix git state: restore `x86_64-phanix.json`, update `.cargo/config.toml` *(done on `supervisor` branch)*
- [x] Create `rust-toolchain.toml` *(done)*
- [x] Add `bootloader`/`bootimage` dependency + QEMU runner *(done)*
- [x] Verify `cargo run` launches QEMU with a bootable image *(verified — blank screen, kernel running)*
- [ ] VGA text buffer driver: volatile writes to 0xb8000, colour codes, newline/scroll
- [ ] Global `WRITER` via `lazy_static` + `spin::Mutex`
- [ ] `print!` / `println!` macros; panic handler prints before halting

### Phase 2 — Stable CPU Environment (by ~Mar 27)
- [ ] GDT (code, data, TSS descriptors)
- [ ] TSS with dedicated double-fault stack (IST entry)
- [ ] IDT: breakpoint, double fault, page fault handlers — print and halt cleanly
- [ ] Serial UART output (`uart_16550`) for headless QEMU test logging
- [ ] PIC 8259 configuration; timer + keyboard interrupt skeletons
- [ ] PS/2 keyboard scancode reader via interrupt handler

### Phase 3 — Memory Management (by ~Apr 24)
- [ ] Physical frame allocator (use bootloader memory map)
- [ ] 4-level page table mapping (`OffsetPageTable` recommended)
- [ ] Map additional pages; virtual → physical address translation
- [ ] **Bump Allocator** implementation + `#[global_allocator]` registration
- [ ] Validate: `Box<T>`, `Vec<T>`, `String` work in kernel code
- [ ] **Fixed-Size Block Allocator** implementation
- [ ] Runtime or compile-time switching between allocators for benchmarking

### Phase 4 — Async Executor & Shell (by ~May 15)
- [ ] `Task` type wrapping `Pin<Box<dyn Future>>`
- [ ] Executor: task queue + `Waker` implementation
- [ ] Keyboard async stream: interrupt → scancode queue → async reader
- [ ] CLI loop: read line, tokenise, dispatch commands
- [ ] `syscheck` command: page table query + RDTSC allocator benchmark

### Phase 5 — Polish & Demo Prep (by ~June 6)
- [ ] RDTSC with `cpuid` serialisation; formatted comparison output
- [ ] `cargo test` via QEMU serial harness
- [ ] Demo script + build/run instructions in README

---

## Issues to Watch

| Issue | Priority | Status |
|---|---|---|
| Broken git state (x86_64-phanix.json) | ~~Immediate~~ | ✅ Fixed on `supervisor` branch |
| No bootable image | ~~Immediate~~ | ✅ Fixed — `cargo run` works |
| `rust-toolchain.toml` missing | ~~Immediate~~ | ✅ Fixed on `supervisor` branch |
| `Cargo.lock` gitignored | ~~Medium~~ | ✅ Fixed — now committed |
| Paging + dual allocator complexity | **High** | Phase 3 most likely to slip |
| RDTSC benchmarking methodology | **Medium** | Student needs experimental protocol before Phase 4 |
| `bootimage` vs `bootloader 0.11` | Medium | Committed to `bootimage 0.10` / `bootloader 0.9` — do not switch |
| SSE in dependencies | Low | Target spec disables it; watch for crates that emit SSE |
| Intel SDM citation "(2026)" | Low | Fix to "(n.d.)" before final report |

---

## Demo MVP (maps to student's own evaluation criteria)

1. **Boots** — QEMU shows text on screen *(currently: black screen — VGA driver needed)*
2. **Handles Division-by-Zero** — prints error instead of triple-faulting
3. **1,000 allocations with both allocators** — live `syscheck` output
4. **Performance comparison documented** — cycles-per-allocation table
5. **Shell accepts commands** — `syscheck`, `help`, `clear` at minimum

---

## Reference Material

- **Primary tutorial**: https://os.phil-opp.com — "Writing an OS in Rust" Ed. 2 — every phase maps to a chapter
- **x86_64 crate**: https://docs.rs/x86_64/latest/x86_64/
- **OSDev Wiki**: https://wiki.osdev.org — VGA, PIC 8259, PS/2, IDT, GDT hardware specs
- **Intel SDM**: https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html — Vol 3A for interrupts/paging
- **Tanenbaum & Bos**: Modern Operating Systems 4th ed. — architectural theory reference
