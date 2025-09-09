use core::arch::asm;

pub struct UserProcess {
    pub pid: usize,
    pub page_table_id: usize,
    pub user_stack: usize,
    pub user_heap: usize,
    pub entry_point: usize,
    pub context: UserContext,
}

pub struct UserContext {
    pub regs: [usize; 32],
    pub pc: usize,
    pub sp: usize,
}

impl UserContext {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: 0,
            sp: 0,
        }
    }
}

impl UserProcess {
    pub fn new(pid: usize, page_table_id: usize, entry_point: usize) -> Self {
        Self {
            pid,
            page_table_id,
            user_stack: 0x7FFFFFFF000,
            user_heap: 0x10000000,
            entry_point,
            context: UserContext::new(),
        }
    }
    
    pub fn setup_user_context(&mut self) {
        self.context.pc = self.entry_point;
        self.context.sp = self.user_stack;
        self.context.regs[2] = self.user_stack;
        self.context.regs[10] = 0;
    }
}

pub fn sys_read_user(fd: usize, _buf: usize, _count: usize) -> usize {
    crate::print_info!("sys_read: fd={}", fd);
    if fd == 0 {
        0
    } else {
        usize::MAX
    }
}

pub fn sys_write_user(fd: usize, _buf: usize, count: usize) -> usize {
    crate::print_info!("sys_write: fd={}, count={}", fd, count);
    if fd == 1 || fd == 2 {
        if let Ok(_data) = copy_from_user(0, count) {
            crate::println!("User wrote {} bytes to fd {}", count, fd);
            count
        } else {
            usize::MAX
        }
    } else {
        usize::MAX
    }
}

pub fn sys_exit_user(status: usize) -> usize {
    crate::print_info!("sys_exit: status={}", status);
    crate::println!("User process exited with status: {}", status);
    loop {
        unsafe { asm!("wfi"); }
    }
}

pub fn sys_brk_user(_addr: usize) -> usize {
    crate::print_info!("sys_brk called");
    0x10000000
}

fn copy_from_user(_user_ptr: usize, _size: usize) -> Result<alloc::vec::Vec<u8>, ()> {
    if _user_ptr < 0x10000000 || _user_ptr > 0x7FFFFFFF {
        return Err(());
    }
    Ok(alloc::vec::Vec::new())
}

pub fn enter_user_mode(process: &UserProcess) {
    unsafe {
        asm!(
            "lw ra, 0({0})",
            "lw sp, 4({0})",
            "lw gp, 8({0})",
            "lw tp, 12({0})",
            "lw t0, 16({0})",
            "lw t1, 20({0})",
            "lw t2, 24({0})",
            "lw s0, 28({0})",
            "lw s1, 32({0})",
            "lw a0, 36({0})",
            "lw a1, 40({0})",
            "lw a2, 44({0})",
            "lw a3, 48({0})",
            "lw a4, 52({0})",
            "lw a5, 56({0})",
            "lw a6, 60({0})",
            "lw a7, 64({0})",
            "lw s2, 68({0})",
            "lw s3, 72({0})",
            "lw s4, 76({0})",
            "lw s5, 80({0})",
            "lw s6, 84({0})",
            "lw s7, 88({0})",
            "lw s8, 92({0})",
            "lw s9, 96({0})",
            "lw s10, 100({0})",
            "lw s11, 104({0})",
            "lw t3, 108({0})",
            "lw t4, 112({0})",
            "lw t5, 116({0})",
            "lw t6, 120({0})",
            "lw t0, 124({0})",
            "csrw sepc, t0",
            in(reg) process.context.regs.as_ptr(),
        );
    }
}

pub fn init_user_mode() -> bool {
    crate::print_info!("Initializing user mode support...");
    if !crate::vm::init_vm() {
        crate::print_fail!("Failed to initialize virtual memory");
        return false;
    }
    crate::print_ok!("User mode support initialized");
    true
}

#[repr(usize)]
enum Syscall {
    Write = 64,
    RamfsList = 1003,
    RamfsRead = 1001,
}

fn syscall(num: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    unsafe {
        let ret: usize;
        core::arch::asm!(
            "mv a7, {0}",
            "mv a0, {1}",
            "mv a1, {2}",
            "mv a2, {3}",
            "ecall",
            "mv {4}, a0",
            in(reg) num, in(reg) arg1, in(reg) arg2, in(reg) arg3, out(reg) ret
        );
        ret
    }
}

fn read_line(buf: &mut [u8]) -> usize {
    let mut i = 0;
    while i < buf.len() {
        let ch = crate::user_loader::getchar();
        
        match ch {
            b'\r' | b'\n' => {
                crate::print::sbi_putchar(b'\n');
                break;
            }
            b'\x08' | b'\x7f' => {
                if i > 0 {
                    i -= 1;
                    crate::print::sbi_putchar(b'\x08');
                    crate::print::sbi_putchar(b' ');
                    crate::print::sbi_putchar(b'\x08');
                }
            }
            b'\x03' => {
                while i > 0 {
                    i -= 1;
                    crate::print::sbi_putchar(b'\x08');
                    crate::print::sbi_putchar(b' ');
                    crate::print::sbi_putchar(b'\x08');
                }
            }
            _ => {
                if ch >= 32 && ch <= 126 {
                    buf[i] = ch;
                    i += 1;
                    crate::print::sbi_putchar(ch);
                }
            }
        }
    }
    i
}

fn handle_command(input: &str) -> bool {
    if input == "ls" {
        let fs = crate::ramfs::ramfs_mut();
        let files = fs.list_files_detailed();
        crate::println!("{:<20} {:<8} {:<12} {}", "NAME", "TYPE", "SIZE (bytes)", "CREATED");
        crate::println!("{}", "-".repeat(50));
        for file in files {
            crate::println!("{:<20} {:<8} {:<12} {}", 
                file.name, 
                file.file_type.to_string(), 
                file.size,
                file.created_at
            );
        }
        crate::println!();
    } else if input.starts_with("cat ") {
        let filename = input.strip_prefix("cat ").unwrap().trim();
        let fs = crate::ramfs::ramfs_mut();
        if let Some(data) = fs.read_file(filename) {
            if let Ok(s) = core::str::from_utf8(data) {
                crate::println!("{}", s);
            } else {
                crate::println!("(binary data)");
            }
        } else {
            crate::println!("File not found: {}", filename);
        }
        crate::println!();
    } else if input.starts_with("echo ") {
        let text = input.strip_prefix("echo ").unwrap();
        crate::println!("{}", text);
        crate::println!();
    } else if input.starts_with("info ") {
        let filename = input.strip_prefix("info ").unwrap().trim();
        let fs = crate::ramfs::ramfs_mut();
        if let Some(file) = fs.get_file_info(filename) {
            crate::println!("File Information:");
            crate::println!("  Name: {}", file.name);
            crate::println!("  Type: {}", file.file_type.to_string());
            crate::println!("  Size: {} bytes", file.size);
            crate::println!("  Created: {}", file.created_at);
        } else {
            crate::println!("File not found: {}", filename);
        }
        crate::println!();
    } else if input == "exit" {
        crate::println!("Bye!");
        return true;
    } else if !input.is_empty() {
        crate::println!("Unknown command: {}", input);
        crate::println!();
    }
    false
}

pub fn launch_shell() {
    crate::println!("Welcome to the S.T.A.R. shell!");

    loop {
        crate::print!("> ");
        let mut buf = [0u8; 64];
        let len = read_line(&mut buf);
        let input = core::str::from_utf8(&buf[..len]).unwrap_or("").trim();
        
        if handle_command(input) {
            break;
        }
    }
}