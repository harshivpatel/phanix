use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use x86_64::structures::gdt::SegmentSelector;

// Label our emergency stack position as table slot 0
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    // Create a permanent, read only toolkit structure in memory
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        
        // Point table slot 0 to our custom emergency memory block
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // Allocate a clean storage container of 20,480 bytes
            const STACK_SIZE: usize =  4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // Get the memory location where our container begins
            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            
            // Calculate the top address because stacks grow downward
            let stack_end = stack_start + STACK_SIZE;
            
            // Save the top address into the slot
            stack_end
        };
        tss
    };
}

lazy_static! {
    // Create a permanent, read-only hardware registration table in memory
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        // Returns a tuple with all its setup work and packs gdt & selectors together
        (gdt, Selectors { code_selector, tss_selector })
    };
}


// The container holding our distinct hardware index keys
struct Selectors {
    code_selector: SegmentSelector, // Key pointing to our Kernel Code rules
    tss_selector: SegmentSelector, // Key pointing to our TSS Emergency Stack
}

// Tells the physical CPU chip to drop old settings and use our new configuration
pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};

    // Load the new master GDT notebook structure into the CPU
    GDT.0.load();
    unsafe {
        // Place the code key in the CPU pocket to activate kernel execution permissions
        CS::set_reg(GDT.1.code_selector);
        // Place the TSS key in the CPU pocket to put the emergency stack on standby
        load_tss(GDT.1.tss_selector);
    }
}