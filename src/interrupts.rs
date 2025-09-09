use crate::scheduler::TaskContext;
use core::arch::asm;

pub const INTERRUPT_USER_SOFTWARE: usize = 0;
pub const INTERRUPT_SUPERVISOR_SOFTWARE: usize = 1;
pub const INTERRUPT_MACHINE_SOFTWARE: usize = 3;
pub const INTERRUPT_USER_TIMER: usize = 4;
pub const INTERRUPT_SUPERVISOR_TIMER: usize = 5;
pub const INTERRUPT_MACHINE_TIMER: usize = 7;
pub const INTERRUPT_USER_EXTERNAL: usize = 8;
pub const INTERRUPT_SUPERVISOR_EXTERNAL: usize = 9;
pub const INTERRUPT_MACHINE_EXTERNAL: usize = 11;

pub const EXCEPTION_INSTRUCTION_MISALIGNED: usize = 0;
pub const EXCEPTION_INSTRUCTION_ACCESS_FAULT: usize = 1;
pub const EXCEPTION_ILLEGAL_INSTRUCTION: usize = 2;
pub const EXCEPTION_BREAKPOINT: usize = 3;
pub const EXCEPTION_LOAD_MISALIGNED: usize = 4;
pub const EXCEPTION_LOAD_ACCESS_FAULT: usize = 5;
pub const EXCEPTION_STORE_MISALIGNED: usize = 6;
pub const EXCEPTION_STORE_ACCESS_FAULT: usize = 7;
pub const EXCEPTION_ECALL_U: usize = 8;
pub const EXCEPTION_ECALL_S: usize = 9;
pub const EXCEPTION_ECALL_M: usize = 11;
pub const EXCEPTION_INSTRUCTION_PAGE_FAULT: usize = 12;
pub const EXCEPTION_LOAD_PAGE_FAULT: usize = 13;
pub const EXCEPTION_STORE_PAGE_FAULT: usize = 15;

pub struct InterruptManager {
    timer_enabled: bool,
    timer_interval: u64,
}

impl InterruptManager {
    pub const fn new() -> Self {
        Self {
            timer_enabled: false,
            timer_interval: 0,
        }
    }

    pub fn init(&mut self) -> bool {
        crate::print_info!("Initializing interrupt manager...");
        
        if !self.enable_supervisor_interrupts() {
            return false;
        }
        
        crate::print_ok!("Interrupt manager initialized");
        true
    }

    pub fn enable_machine_interrupts(&mut self) -> bool {
        crate::print_info!("Enabling machine interrupts...");
        
        unsafe {
            let mut mstatus: usize;
            asm!("csrr {}, mstatus", out(reg) mstatus);
            crate::print_info!("Current mstatus: {:#x}", mstatus);
            
            mstatus |= 1 << 3;
            asm!("csrw mstatus, {}", in(reg) mstatus);
            crate::print_info!("Set MIE bit in mstatus: {:#x}", mstatus);
            
            let mut mie: usize;
            asm!("csrr {}, mie", out(reg) mie);
            crate::print_info!("Current MIE register: {:#x}", mie);
            
            mie |= 1 << 7;
            asm!("csrw mie, {}", in(reg) mie);
            crate::print_info!("Set MTIE bit in MIE register: {:#x}", mie);
        }
        crate::print_ok!("Machine interrupts enabled");
        true
    }

    pub fn enable_supervisor_interrupts(&mut self) -> bool {
        crate::print_info!("Enabling supervisor interrupts...");
        
        unsafe {
            let mut sstatus: usize;
            asm!("csrr {}, sstatus", out(reg) sstatus);
            crate::print_info!("Current sstatus: {:#x}", sstatus);
            
            sstatus |= 1 << 1;
            asm!("csrw sstatus, {}", in(reg) sstatus);
            crate::print_info!("Set SIE bit in sstatus: {:#x}", sstatus);
            
            let mut sie: usize;
            asm!("csrr {}, sie", out(reg) sie);
            crate::print_info!("Current SIE register: {:#x}", sie);
            
            sie |= 1 << 5;
            asm!("csrw sie, {}", in(reg) sie);
            crate::print_info!("Set STIE bit in SIE register: {:#x}", sie);
        }
        crate::print_ok!("Supervisor interrupts enabled");
        true
    }

    pub fn setup_timer(&mut self, interval_us: u64) -> bool {
        self.timer_interval = interval_us;
        self.timer_enabled = true;
        
        crate::print_info!("Setting up timer with interval: {} us", interval_us);
        
        unsafe {
            let clint_base = 0x2000000 as *mut u64;
            let mtime = core::ptr::read_volatile(clint_base);
            crate::print_info!("Current time from CLINT: {}", mtime);
            
            let mtimecmp = mtime + interval_us;
            core::ptr::write_volatile(clint_base.add(1), mtimecmp);
            crate::print_info!("Set mtimecmp to: {}", mtimecmp);
        }
        
        crate::print_ok!("Timer interrupt configured");
        true
    }

    pub fn handle_timer_interrupt(&mut self) {
        if !self.timer_enabled {
            return;
        }

        const CLINT_BASE: usize = 0x2000000;
        unsafe {
            let clint = CLINT_BASE as *mut u64;
            let mtime = core::ptr::read_volatile(clint);
            let next = mtime.wrapping_add(self.timer_interval);
            core::ptr::write_volatile(clint.add(1), next);
        }
    }

    pub fn handle_external_interrupt(&mut self) {
        crate::println!("External interrupt received");
    }
}

pub struct Task {
    pub ctx: TaskContext,
    pub active: bool,
}

pub static mut INTERRUPT_MANAGER: InterruptManager = InterruptManager::new();

pub fn handle_interrupt(scause: usize, sepc: usize, stval: usize) {
    let interrupt_type = scause & 0x8000000000000000;
    
    if interrupt_type != 0 {
        let interrupt_code = scause & 0x7FFFFFFFFFFFFFFF;
        
        match interrupt_code {
            INTERRUPT_SUPERVISOR_TIMER => {
                unsafe {
                    let manager = &mut INTERRUPT_MANAGER;
                    manager.handle_timer_interrupt();
                }
            }
            INTERRUPT_SUPERVISOR_EXTERNAL => {
                unsafe {
                    let manager = &mut INTERRUPT_MANAGER;
                    manager.handle_external_interrupt();
                }
            }
            _ => {
                crate::println!("Unhandled interrupt: {}", interrupt_code);
            }
        }
    } else {
        crate::println!("Exception occurred: scause={:#x}, sepc={:#x}, stval={:#x}", 
                       scause, sepc, stval);
    }
}

pub fn init_interrupts() -> bool {
    unsafe {
        let manager = &mut INTERRUPT_MANAGER;
        manager.init()
    }
}

pub fn handle_timer_interrupt() {
    unsafe {
        INTERRUPT_MANAGER.handle_timer_interrupt();
    }
}