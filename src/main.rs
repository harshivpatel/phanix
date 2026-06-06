#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(phanix::test_runner)]
#![reexport_test_harness_main = "test_main"]

use phanix::task::executor::Executor;
use core::{ops::Add, panic::PanicInfo};
use bootloader::{BootInfo, entry_point};
use phanix::memory::BootInfoFrameAllocator;
use x86_64::{VirtAddr, structures::paging::PageTable};
use x86_64::{structures::paging::Page};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use phanix::allocator;
use phanix::task::{shell};
use phanix::println;
use phanix::task::{Task, simple_executor::SimpleExecutor};
use phanix::task::keyboard;

extern crate alloc;

// Register the custom kernel entry point macro
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use phanix::memory;
    use x86_64::{structures::paging::Translate, VirtAddr};

    // Print welcome logging messages and initialize the GDT and IDT structures
    println!("Hello World{}", "!");
    phanix::init();

    // Calculate the virtual mapping highway base address provided by the bootloader
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    
    // Initialize the page mapping tracking utility and physical frame allocator
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    
    // Map virtual pages to physical memory fields to activate heap storage structures
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // Allocate a test value on the heap using a boxed raw pointer wrapper
    let heap_value = Box::new(41);
    //println!("heap_value at {:p}", heap_value);

    // Create a dynamic array and grow it to verify heap reallocation workflows
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    //println!("vec at {:p}", vec.as_slice());

    // Initialize an reference counted wrapper to verify data lifetime tracking
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    //println!("current reference count is {} now", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    //println!("reference count is {} now", Rc::strong_count(&cloned_reference));


    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(shell::run_shell()));
    executor.run();

    // Run the automated integration test suite if a test flag is active
    #[cfg(test)]
    test_main();

    // Log the successful initialization sequence and park the CPU core safely
    println!("It did not crash");
    phanix::hlt_loop();
}

// Intercept standard runtime panics during regular supervisor execution loops
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    phanix::hlt_loop();
}

// Intercept execution panic states when running the headless testing framework
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    phanix::test_panic_handler(info)
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async_number: {}", number);
}