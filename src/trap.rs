use crate::interrupts::*;
use core::arch::asm;

macro_rules! read_csr {
    ($csr:expr) => {{
        let mut value: usize;
        unsafe {
            ::core::arch::asm!(concat!("csrr {}, ", $csr), out(reg) value);
        }
        value
    }};
}

#[no_mangle]
pub extern "C" fn trap_handler() {
    let mut regs: [usize; 32] = [0; 32];
    unsafe {
        asm!(
            "sd x0,  0({0})",
            "sd x1,  8({0})",
            "sd x2,  16({0})",
            "sd x3,  24({0})",
            "sd x4,  32({0})",
            "sd x5,  40({0})",
            "sd x6,  48({0})",
            "sd x7,  56({0})",
            "sd x8,  64({0})",
            "sd x9,  72({0})",
            "sd x10, 80({0})",
            "sd x11, 88({0})",
            "sd x12, 96({0})",
            "sd x13, 104({0})",
            "sd x14, 112({0})",
            "sd x15, 120({0})",
            "sd x16, 128({0})",
            "sd x17, 136({0})",
            "sd x18, 144({0})",
            "sd x19, 152({0})",
            "sd x20, 160({0})",
            "sd x21, 168({0})",
            "sd x22, 176({0})",
            "sd x23, 184({0})",
            "sd x24, 192({0})",
            "sd x25, 200({0})",
            "sd x26, 208({0})",
            "sd x27, 216({0})",
            "sd x28, 224({0})",
            "sd x29, 232({0})",
            "sd x30, 240({0})",
            "sd x31, 248({0})",
            in(reg) regs.as_mut_ptr(),
            options(nostack)
        );
    }

    let scause = read_csr!("scause");
    let sepc = read_csr!("sepc");
    let sstatus = read_csr!("sstatus");
    let _stval = read_csr!("stval");

    let is_interrupt = (scause & 0x8000_0000_0000_0000u128 as usize) != 0;
    let code = scause & 0xff;

    if is_interrupt {
        match code {
            INTERRUPT_SUPERVISOR_TIMER => {
                unsafe {
                    let cur = crate::scheduler::CURRENT_TASK;
                    let tasks = &mut crate::scheduler::TASKS;
                    crate::scheduler::save_context(&mut tasks[cur].ctx, &regs, sepc, regs[2], sstatus);
                }

                crate::interrupts::handle_timer_interrupt();
                let next = crate::scheduler::next_task();
                crate::scheduler::switch_to_task(next);
            }
            _ => {
                crate::println!("Unhandled interrupt code: {}", code);
                unsafe {
                    let cur = crate::scheduler::CURRENT_TASK;
                    let tasks = &mut crate::scheduler::TASKS;
                    crate::scheduler::save_context(&mut tasks[cur].ctx, &regs, sepc, regs[2], sstatus);
                }
                let next = crate::scheduler::next_task();
                crate::scheduler::switch_to_task(next);
            }
        }
    } else {
        let exception_code = scause & 0xff;
        if exception_code == crate::interrupts::EXCEPTION_ECALL_U {
            let syscall_num = regs[17];
            let arg0 = regs[10];
            let arg1 = regs[11];
            let arg2 = regs[12];
            let ret = crate::syscall::handle_syscall(syscall_num, arg0, arg1, arg2);
            regs[10] = ret;
            let new_sepc = sepc + 4;
            unsafe {
                let cur = crate::scheduler::CURRENT_TASK;
                let tasks = &mut crate::scheduler::TASKS;
                crate::scheduler::save_context(&mut tasks[cur].ctx, &regs, new_sepc, regs[2], sstatus);
            }
            let next = crate::scheduler::next_task();
            crate::scheduler::switch_to_task(next);
        } else {
            unsafe {
                let cur = crate::scheduler::CURRENT_TASK;
                let tasks = &mut crate::scheduler::TASKS;
                crate::scheduler::save_context(&mut tasks[cur].ctx, &regs, sepc, regs[2], sstatus);
            }
            crate::println!("Exception (scause = {:#x}), dropping to kernel", scause);
            let next = crate::scheduler::next_task();
            crate::scheduler::switch_to_task(next);
        }
    }
}