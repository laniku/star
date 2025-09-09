use core::ptr::{self, NonNull};
use core::alloc::{GlobalAlloc, Layout};

pub const PAGE_SIZE: usize = 4096;
pub const KERNEL_HEAP_SIZE: usize = 1024 * 1024;
pub const MAX_PAGES: usize = 256;
pub const KERNEL_START: usize = 0x80200000;

pub struct PageAllocator {
    bitmap: [u64; MAX_PAGES / 64],
    total_pages: usize,
    free_pages: usize,
}

impl PageAllocator {
    pub const fn new() -> Self {
        Self {
            bitmap: [0; MAX_PAGES / 64],
            total_pages: 0,
            free_pages: 0,
        }
    }

    pub fn init(&mut self, _start_addr: usize, size: usize) -> bool {
        if size < PAGE_SIZE {
            crate::print_fail!("Page allocator: Invalid size {}", size);
            return false;
        }
        
        self.total_pages = size / PAGE_SIZE;
        self.free_pages = self.total_pages;
        
        for i in 0..self.bitmap.len() {
            self.bitmap[i] = 0;
        }
        
        crate::print_ok!("Page allocator: {} pages available", self.free_pages);
        true
    }

    pub fn alloc_page(&mut self) -> Option<usize> {
        for (word_idx, word) in self.bitmap.iter_mut().enumerate() {
            if *word != u64::MAX {
                let bit_idx = word.trailing_ones() as usize;
                if bit_idx < 64 {
                    *word |= 1u64 << bit_idx;
                    self.free_pages -= 1;
                    let page_idx = word_idx * 64 + bit_idx;
                    return Some(page_idx * PAGE_SIZE);
                }
            }
        }
        None
    }

    pub fn dealloc_page(&mut self, addr: usize) {
        let page_idx = addr / PAGE_SIZE;
        let word_idx = page_idx / 64;
        let bit_idx = page_idx % 64;
        
        if word_idx < self.bitmap.len() {
            self.bitmap[word_idx] &= !(1u64 << bit_idx);
            self.free_pages += 1;
        }
    }

    pub fn get_free_pages(&self) -> usize {
        self.free_pages
    }
}

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
        }
    }

    pub fn init(&mut self, start: usize, size: usize) -> bool {
        if size == 0 {
            crate::print_fail!("Heap allocator: Invalid size {}", size);
            return false;
        }
        
        self.heap_start = start;
        self.heap_end = start + size;
        self.next = start;
        crate::print_ok!("Heap allocator: {:#x} - {:#x} ({} bytes)", start, start + size, size);
        true
    }

    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        let align = layout.align();
        let size = layout.size();
        
        let aligned_next = (self.next + align - 1) & !(align - 1);
        
        if aligned_next + size <= self.heap_end {
            self.next = aligned_next + size;
            unsafe {
                Ok(NonNull::new_unchecked(aligned_next as *mut u8))
            }
        } else {
            Err(())
        }
    }

    pub fn get_used_bytes(&self) -> usize {
        self.next - self.heap_start
    }
}

pub static mut PAGE_ALLOCATOR: PageAllocator = PageAllocator::new();
pub static mut HEAP_ALLOCATOR: BumpAllocator = BumpAllocator::new();

pub struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let allocator = &mut HEAP_ALLOCATOR;
        allocator.alloc(layout).map_or(ptr::null_mut(), |ptr| ptr.as_ptr())
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't support deallocation
    }
}

pub fn init_memory() -> bool {
    unsafe {
        extern "C" {
            static __heap_start: u8;
            static __heap_end: u8;
        }
        
        let heap_start = &__heap_start as *const u8 as usize;
        let heap_end = &__heap_end as *const u8 as usize;
        let heap_size = heap_end - heap_start;
        
        crate::print_info!("Heap region: {:#x} - {:#x} ({} bytes)", 
                          heap_start, heap_end, heap_size);
        
        let page_allocator = &mut PAGE_ALLOCATOR;
        if !page_allocator.init(heap_start, heap_size) {
            return false;
        }
        
        let heap_allocator = &mut HEAP_ALLOCATOR;
        if !heap_allocator.init(heap_start, heap_size) {
            return false;
        }
    }
    
    crate::print_ok!("Memory management initialized");
    true
}

pub fn alloc_page() -> Option<usize> {
    unsafe {
        let allocator = &mut PAGE_ALLOCATOR;
        allocator.alloc_page()
    }
}

pub fn dealloc_page(addr: usize) {
    unsafe {
        let allocator = &mut PAGE_ALLOCATOR;
        allocator.dealloc_page(addr)
    }
}

pub fn get_memory_stats() -> (usize, usize, usize) {
    unsafe {
        let page_allocator = &PAGE_ALLOCATOR;
        let heap_allocator = &HEAP_ALLOCATOR;
        (
            page_allocator.get_free_pages(),
            heap_allocator.get_used_bytes(),
            KERNEL_HEAP_SIZE
        )
    }
}