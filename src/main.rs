#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

mod print;
mod trap;
mod memory;
mod interrupts;
mod syscall;
mod scheduler;
mod vm;
mod user;
mod user_loader;
mod ramfs;

use core::arch::asm;
use core::alloc::{Layout, GlobalAlloc};

#[global_allocator]
static ALLOCATOR: memory::KernelAllocator = memory::KernelAllocator;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
pub extern "C" fn boot() -> ! {
    unsafe {
        asm!(
            "la sp, __stack_top",
            "j {main}",
            main = sym main,
            options(noreturn)
        );
    }
}

unsafe extern "C" {
    static mut __bss_start: u8;
    static mut __bss_end: u8;
    static __heap_start: u8;
    static __heap_end: u8;
}

fn init_bss() {
    unsafe {
        let bss_start = &raw mut __bss_start;
        let bss_size = (&raw mut __bss_end as usize) - (&raw mut __bss_start as usize);
        core::ptr::write_bytes(bss_start, 0, bss_size);
    }
}

fn init_trap_handler() {
    let trap_addr = trap::trap_handler as usize & !1;
    unsafe {
        asm!("csrw stvec, {}", in(reg) trap_addr);
    }
    crate::print_ok!("Trap handler set up at: {:#x}", trap_addr);
}

fn init_kernel_systems() {
    if !memory::init_memory() {
        crate::print_fail!("Memory management initialization failed");
        panic!("Memory management initialization failed");
    }
    
    if !interrupts::init_interrupts() {
        crate::print_fail!("Interrupt system initialization failed");
        panic!("Interrupt system initialization failed");
    }
    
    if !scheduler::init_scheduler() {
        crate::print_fail!("Scheduler initialization failed");
        panic!("Scheduler initialization failed");
    }
    
    if !user::init_user_mode() {
        crate::print_fail!("User mode initialization failed");
        panic!("User mode initialization failed");
    }
    
    if !ramfs::init_ramfs() {
        crate::print_fail!("RAMFS initialization failed");
        panic!("RAMFS initialization failed");
    }
}

fn setup_sample_files() {
    let fs = crate::ramfs::ramfs_mut();
    fs.create_file("hello.txt", b"Hello, World!\nThis is a text file.\n");
    fs.create_file("readme.md", b"# S.T.A.R. Kernel\n\nA simple RISC-V kernel implementation.\n");
    fs.create_file("config.bin", b"\x00\x01\x02\x03\xFF\xFE\xFD\xFC");
    fs.create_file("kernel.elf", b"ELF\x7F\x45\x4C\x46\x02\x01\x01\x00");
}

fn main() -> ! {
    init_bss();
    init_trap_handler();
    
    println!("S.T.A.R. booting...");
    init_kernel_systems();
    
    crate::print_ok!("Kernel initialization complete!");
    setup_sample_files();
    
    crate::user::launch_shell();
    loop {}
}

fn test_memory_management() {
    println!("Testing memory management...");
    
    if let Some(page_addr) = memory::alloc_page() {
        println!("Allocated page at: {:#x}", page_addr);
        memory::dealloc_page(page_addr);
        println!("Deallocated page");
    }
    
    let layout = Layout::from_size_align(1024, 4).unwrap();
    let ptr = unsafe { ALLOCATOR.alloc(layout) };
    if !ptr.is_null() {
        println!("Allocated 1KB on heap at: {:#x}", ptr as usize);
    }
    
    let mut vec = alloc::vec::Vec::new();
    vec.push(42);
    vec.push(1337);
    println!("Vec allocated: {:?}", vec);
    
    let (free_pages, used_heap, total_heap) = memory::get_memory_stats();
    println!("Memory stats - Free pages: {}, Heap used: {}/{} bytes", 
             free_pages, used_heap, total_heap);
}

fn test_timer_interrupts() {
    println!("Testing timer interrupt setup...");
    
    unsafe {
        let manager = &mut interrupts::INTERRUPT_MANAGER;
        if manager.setup_timer(1000000) {
            println!("Timer interrupt setup successful");
        } else {
            println!("Timer interrupt setup failed");
        }
    }
}

use core::panic::PanicInfo;

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    println!("panic: {}", info);
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}