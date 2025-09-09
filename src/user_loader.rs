use core::arch::asm;

pub static USER_PROG: [u32; 2] = [
    0x00000013, // nop
    0x0000006f, // j 0 (infinite loop)
];

pub static USER_PROG2: [u32; 2] = [
    0x00000013, // nop
    0x0000006f, // j 0 (infinite loop)
];

pub const USER_PROG_BASE: usize = 0x8040_0000;
pub const USER_PROG2_BASE: usize = 0x8040_1000;
pub const USER_STACK_BASE: usize = 0x8050_0000;
pub const USER_STACK_SIZE: usize = 0x1000;

pub fn load_user_programs() -> bool {
    unsafe {
        let code_addr = USER_PROG_BASE as *mut u8;
        core::ptr::copy_nonoverlapping(
            USER_PROG.as_ptr() as *const u8,
            code_addr,
            USER_PROG.len() * core::mem::size_of::<u32>(),
        );
        let code_addr2 = USER_PROG2_BASE as *mut u8;
        core::ptr::copy_nonoverlapping(
            USER_PROG2.as_ptr() as *const u8,
            code_addr2,
            USER_PROG2.len() * core::mem::size_of::<u32>(),
        );
        let stack_addr = USER_STACK_BASE as *mut u8;
        core::ptr::write_bytes(stack_addr, 0, USER_STACK_SIZE);
    }
    true
}

pub fn run_user_program() -> ! {
    unsafe {
        let entry = USER_PROG_BASE;
        let stack = USER_STACK_BASE + USER_STACK_SIZE;

        asm!(
            "mv sp, {stack}",
            "csrw sepc, {entry}",
            "li t0, 0",
            "csrw sstatus, t0",
            "sret",
            stack = in(reg) stack,
            entry = in(reg) entry,
            options(noreturn)
        );
    }
}

pub fn sbi_getchar() -> i32 {
    crate::print::sbi_getchar()
}

pub fn getchar() -> u8 {
    loop {
        let result = sbi_getchar();
        if result >= 0 {
            return result as u8;
        }
        unsafe {
            core::arch::asm!("nop");
        }
    }
}