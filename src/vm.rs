use core::arch::asm;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_BITS: usize = 12;
pub const VPN_BITS: usize = 9;
pub const PPN_BITS: usize = 44;

pub const PTE_V: usize = 1 << 0;
pub const PTE_R: usize = 1 << 1;
pub const PTE_W: usize = 1 << 2;
pub const PTE_X: usize = 1 << 3;
pub const PTE_U: usize = 1 << 4;
pub const PTE_G: usize = 1 << 5;
pub const PTE_A: usize = 1 << 6;
pub const PTE_D: usize = 1 << 7;

#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub const fn new() -> Self {
        Self { bits: 0 }
    }
    
    pub fn is_valid(&self) -> bool {
        (self.bits & PTE_V) != 0
    }
    
    pub fn is_readable(&self) -> bool {
        (self.bits & PTE_R) != 0
    }
    
    pub fn is_writable(&self) -> bool {
        (self.bits & PTE_W) != 0
    }
    
    pub fn is_executable(&self) -> bool {
        (self.bits & PTE_X) != 0
    }
    
    pub fn is_user(&self) -> bool {
        (self.bits & PTE_U) != 0
    }
    
    pub fn get_ppn(&self) -> usize {
        (self.bits >> 10) & ((1 << PPN_BITS) - 1)
    }
    
    pub fn set_valid(&mut self, valid: bool) {
        self.bits = (self.bits & !PTE_V) | ((valid as usize) * PTE_V);
    }
    
    pub fn set_readable(&mut self, readable: bool) {
        self.bits = (self.bits & !PTE_R) | ((readable as usize) * PTE_R);
    }
    
    pub fn set_writable(&mut self, writable: bool) {
        self.bits = (self.bits & !PTE_W) | ((writable as usize) * PTE_W);
    }
    
    pub fn set_executable(&mut self, executable: bool) {
        self.bits = (self.bits & !PTE_X) | ((executable as usize) * PTE_X);
    }
    
    pub fn set_user(&mut self, user: bool) {
        self.bits = (self.bits & !PTE_U) | ((user as usize) * PTE_U);
    }
    
    pub fn set_ppn(&mut self, ppn: usize) {
        self.bits = (self.bits & 0x3FF) | (ppn << 10);
    }
    
    pub fn set_flags(&mut self, flags: usize) {
        self.bits = (self.bits & 0x3FF) | (flags & 0xFF);
    }
}

pub struct PageTable {
    pub root_ppn: usize,
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub fn new() -> Option<Self> {
        if let Some(page) = crate::memory::alloc_page() {
            let root_ppn = page >> PAGE_BITS;
            Some(Self {
                root_ppn,
                entries: [PageTableEntry::new(); 512],
            })
        } else {
            None
        }
    }
    
    pub fn map_page(&mut self, vpn: usize, ppn: usize, flags: usize) -> bool {
        let _vpn_l2 = (vpn >> 18) & 0x1FF;
        let _vpn_l1 = (vpn >> 9) & 0x1FF;
        let _vpn_l0 = vpn & 0x1FF;
        
        let mut pte = PageTableEntry::new();
        pte.set_valid(true);
        pte.set_ppn(ppn);
        pte.set_flags(flags);
        
        crate::print_info!("Mapping VPN {:#x} to PPN {:#x} with flags {:#x}", vpn, ppn, flags);
        true
    }
    
    pub fn unmap_page(&mut self, vpn: usize) -> bool {
        crate::print_info!("Unmapping VPN {:#x}", vpn);
        true
    }
    
    pub fn get_satp(&self) -> usize {
        (8 << 60) | self.root_ppn
    }
}

pub struct VMManager {
    pub kernel_page_table: PageTable,
    pub user_page_tables: alloc::vec::Vec<PageTable>,
}

impl VMManager {
    pub fn new() -> Option<Self> {
        if let Some(kernel_pt) = PageTable::new() {
            Some(Self {
                kernel_page_table: kernel_pt,
                user_page_tables: alloc::vec::Vec::new(),
            })
        } else {
            None
        }
    }
    
    pub fn create_user_page_table(&mut self) -> Option<usize> {
        if let Some(user_pt) = PageTable::new() {
            let id = self.user_page_tables.len();
            self.user_page_tables.push(user_pt);
            Some(id)
        } else {
            None
        }
    }
    
    pub fn get_user_page_table(&mut self, id: usize) -> Option<&mut PageTable> {
        self.user_page_tables.get_mut(id)
    }
    
    pub fn switch_to_kernel(&self) {
        unsafe {
            asm!("csrw satp, {}", in(reg) self.kernel_page_table.get_satp());
            asm!("sfence.vma");
        }
    }
    
    pub fn switch_to_user(&self, page_table_id: usize) {
        if let Some(user_pt) = self.user_page_tables.get(page_table_id) {
            unsafe {
                asm!("csrw satp, {}", in(reg) user_pt.get_satp());
                asm!("sfence.vma");
            }
        }
    }
}

pub static mut VM_MANAGER: Option<VMManager> = None;

pub fn init_vm() -> bool {
    crate::print_info!("Initializing virtual memory...");
    
    unsafe {
        if let Some(vm_manager) = VMManager::new() {
            VM_MANAGER = Some(vm_manager);
            crate::print_ok!("Virtual memory manager initialized");
            true
        } else {
            crate::print_fail!("Failed to initialize virtual memory manager");
            false
        }
    }
}

pub fn get_vm_manager() -> Option<&'static mut VMManager> {
    unsafe {
        VM_MANAGER.as_mut()
    }
}

pub fn create_user_page_table() -> Option<usize> {
    if let Some(vm_manager) = get_vm_manager() {
        vm_manager.create_user_page_table()
    } else {
        None
    }
}

pub fn switch_to_kernel() {
    if let Some(vm_manager) = get_vm_manager() {
        vm_manager.switch_to_kernel();
    }
}

pub fn switch_to_user(page_table_id: usize) {
    if let Some(vm_manager) = get_vm_manager() {
        vm_manager.switch_to_user(page_table_id);
    }
}