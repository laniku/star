use core::arch::asm;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    Exited,
    Waiting,
}

#[derive(Clone)]
pub struct Task {
    pub ctx: TaskContext,
    pub active: bool,
    pub pid: usize,
    pub ppid: usize,
    pub state: TaskState,
}

#[derive(Clone)]
pub struct TaskContext {
    pub regs: [usize; 32],
    pub pc: usize,
    pub sp: usize,
    pub sstatus: usize,
    pub mode: u8,
}

pub const MAX_TASKS: usize = 4;

pub static mut TASKS: [Task; MAX_TASKS] = [
    Task { ctx: TaskContext { regs: [0; 32], pc: 0, sp: 0, sstatus: 0, mode: 0 }, active: false, pid: 0, ppid: 0, state: TaskState::Exited },
    Task { ctx: TaskContext { regs: [0; 32], pc: 0, sp: 0, sstatus: 0, mode: 0 }, active: false, pid: 1, ppid: 0, state: TaskState::Exited },
    Task { ctx: TaskContext { regs: [0; 32], pc: 0, sp: 0, sstatus: 0, mode: 0 }, active: false, pid: 2, ppid: 0, state: TaskState::Exited },
    Task { ctx: TaskContext { regs: [0; 32], pc: 0, sp: 0, sstatus: 0, mode: 0 }, active: false, pid: 3, ppid: 0, state: TaskState::Exited },
];

pub static mut CURRENT_TASK: usize = 0;

pub fn init_scheduler() -> bool {
    unsafe {
        TASKS[0] = Task {
            ctx: TaskContext { regs: [0; 32], pc: 0, sp: 0, sstatus: 0, mode: 0 },
            active: true,
            pid: 0,
            ppid: 0,
            state: TaskState::Ready,
        };
        TASKS[1] = Task {
            ctx: TaskContext {
                regs: [0; 32],
                pc: crate::user_loader::USER_PROG_BASE,
                sp: crate::user_loader::USER_STACK_BASE + crate::user_loader::USER_STACK_SIZE,
                sstatus: 0,
                mode: 1,
            },
            active: true,
            pid: 1,
            ppid: 0,
            state: TaskState::Ready,
        };
    }
    true
}

pub fn save_context(dst: &mut TaskContext, regs: &[usize; 32], pc: usize, sp: usize, sstatus: usize) {
    dst.regs.copy_from_slice(regs);
    dst.pc = pc;
    dst.sp = sp;
    dst.sstatus = sstatus;
}

pub fn next_task() -> usize {
    unsafe {
        let mut next = (CURRENT_TASK + 1) % MAX_TASKS;
        for _ in 0..MAX_TASKS {
            if TASKS[next].active && TASKS[next].state == TaskState::Ready {
                return next;
            }
            next = (next + 1) % MAX_TASKS;
        }
        0
    }
}

pub fn switch_to_task(next_id: usize) -> ! {
    unsafe {
        CURRENT_TASK = next_id;
        let task = &TASKS[next_id];
        let rptr = task.ctx.regs.as_ptr();
        let pc = task.ctx.pc;
        let sstatus = task.ctx.sstatus;

        asm!(
            "mv t0, {rptr}",
            "ld x1,  8(t0)",
            "ld sp,  16(t0)",
            "ld x3,  24(t0)",
            "ld x4,  32(t0)",
            "ld x5,  40(t0)",
            "ld x6,  48(t0)",
            "ld x7,  56(t0)",
            "ld x8,  64(t0)",
            "ld x9,  72(t0)",
            "ld x10, 80(t0)",
            "ld x11, 88(t0)",
            "ld x12, 96(t0)",
            "ld x13, 104(t0)",
            "ld x14, 112(t0)",
            "ld x15, 120(t0)",
            "ld x16, 128(t0)",
            "ld x17, 136(t0)",
            "ld x18, 144(t0)",
            "ld x19, 152(t0)",
            "ld x20, 160(t0)",
            "ld x21, 168(t0)",
            "ld x22, 176(t0)",
            "ld x23, 184(t0)",
            "ld x24, 192(t0)",
            "ld x25, 200(t0)",
            "ld x26, 208(t0)",
            "ld x27, 216(t0)",
            "ld x28, 224(t0)",
            "ld x29, 232(t0)",
            "ld x30, 240(t0)",
            "ld x31, 248(t0)",
            "csrw sepc, {pc}",
            "csrw sstatus, {sstatus}",
            "sret",
            rptr = in(reg) rptr,
            pc = in(reg) pc,
            sstatus = in(reg) sstatus,
            options(noreturn)
        );
    }
}